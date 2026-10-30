use std::{
    collections::{HashMap, HashSet},
    mem::size_of,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use windows::{
    core::PCWSTR,
    Win32::{
        Foundation::{GetLastError, ERROR_ALREADY_EXISTS, WAIT_OBJECT_0, WAIT_TIMEOUT},
        System::{
            Console::{
                FreeConsole, GetConsoleProcessList, GetConsoleWindow, SetConsoleCtrlHandler,
                CTRL_BREAK_EVENT, CTRL_C_EVENT,
            },
            Diagnostics::ToolHelp::{
                CreateToolhelp32Snapshot, Process32FirstW, Process32NextW, PROCESSENTRY32W,
                TH32CS_SNAPPROCESS,
            },
            Threading::{
                CreateEventW, CreateMutexW, GetProcessId, OpenEventW, WaitForMultipleObjects,
                WaitForSingleObject, SYNCHRONIZATION_SYNCHRONIZE,
            },
        },
    },
};

use crate::{
    model::{ColorState, ImeMode},
    platform::{self, wide, AppResult, OwnedHandle},
    process,
    settings::{self, Settings},
};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConsoleSession {
    pub shell_pid: u32,
    pub created: u64,
    pub shell: String,
    pub console: usize,
    pub terminal: usize,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct RuntimeStatus {
    pub controller_pid: u32,
    pub sessions: Vec<ConsoleSession>,
    pub worker_pids: Vec<u32>,
    pub errors: Vec<String>,
    #[serde(default)]
    pub elevated_terminal_pids: Vec<u32>,
    #[serde(default)]
    pub obsidian_enabled: bool,
    #[serde(default)]
    pub obsidian_clients: usize,
}

#[derive(Debug, Default, Serialize)]
struct DiscoveryReport {
    sessions: Vec<ConsoleSession>,
    elevated_terminal_pids: Vec<u32>,
}

fn earliest_client(
    clients: impl Iterator<Item = (u64, u32)>,
    discoverer_pid: u32,
) -> Option<(u64, u32)> {
    // AttachConsole adds this persistent discoverer to the client list too.
    // A shell opened after the app would otherwise inherit the discoverer's lifetime.
    clients.filter(|(_, pid)| *pid != discoverer_pid).min()
}

/// The discoverer is a separate, hidden process: AttachConsole never changes GUI or shell handles.
pub fn discover() -> AppResult<Vec<ConsoleSession>> {
    Ok(discover_report()?.sessions)
}

fn discover_report() -> AppResult<DiscoveryReport> {
    let snapshot = OwnedHandle(unsafe { CreateToolhelp32Snapshot(TH32CS_SNAPPROCESS, 0)? });
    let mut entry = PROCESSENTRY32W {
        dwSize: size_of::<PROCESSENTRY32W>() as u32,
        ..Default::default()
    };
    let mut processes = HashMap::new();
    let mut next = unsafe { Process32FirstW(snapshot.0, &mut entry) };
    while next.is_ok() {
        let end = entry
            .szExeFile
            .iter()
            .position(|&c| c == 0)
            .unwrap_or(entry.szExeFile.len());
        processes.insert(
            entry.th32ProcessID,
            (
                entry.th32ParentProcessID,
                String::from_utf16_lossy(&entry.szExeFile[..end]),
            ),
        );
        next = unsafe { Process32NextW(snapshot.0, &mut entry) };
    }
    // AttachConsole failures were previously discarded, leaving an administrator
    // terminal indistinguishable from no terminal. Report the permission boundary.
    let elevated = platform::process_elevated(std::process::id())?;
    let mut elevated_terminal_pids = Vec::new();
    if !elevated {
        for (&pid, (_, name)) in &processes {
            if name.eq_ignore_ascii_case("WindowsTerminal.exe")
                && platform::process_elevated(pid).unwrap_or(false)
            {
                elevated_terminal_pids.push(pid);
            }
        }
    }
    elevated_terminal_pids.sort_unstable();
    let mut candidates: Vec<u32> = processes
        .iter()
        .filter_map(|(&pid, (parent, name))| {
            if pid == std::process::id()
                || matches!(
                    name.to_ascii_lowercase().as_str(),
                    "cuesor-engine.exe"
                        | "cuesor.exe"
                        | "openconsole.exe"
                        | "conhost.exe"
                        | "windowsterminal.exe"
                )
            {
                return None;
            }
            let known_shell = matches!(
                name.to_ascii_lowercase().as_str(),
                "nu.exe" | "pwsh.exe" | "powershell.exe" | "cmd.exe" | "bash.exe" | "wsl.exe"
            );
            let mut ancestor = *parent;
            let mut in_terminal = false;
            for _ in 0..24 {
                let Some((previous, executable)) = processes.get(&ancestor) else {
                    break;
                };
                if executable.eq_ignore_ascii_case("WindowsTerminal.exe") {
                    in_terminal = true;
                    break;
                }
                if *previous == ancestor {
                    break;
                }
                ancestor = *previous;
            }
            (known_shell || in_terminal).then_some(pid)
        })
        .collect();
    candidates.sort_unstable();
    let mut consoles = HashSet::new();
    let mut sessions = Vec::new();
    for pid in candidates {
        if platform::attach(pid).is_err() {
            continue;
        }
        let console = unsafe { GetConsoleWindow() }.0 as usize;
        let host = platform::terminal_host();
        if !consoles.insert(console) || !platform::is_terminal(host) {
            continue;
        }
        let mut clients = vec![0u32; 64];
        let count = unsafe { GetConsoleProcessList(&mut clients) } as usize;
        if count > clients.len() {
            clients.resize(count, 0);
            unsafe {
                GetConsoleProcessList(&mut clients);
            }
        }
        // Prefer the earliest client, so a transient command cannot become the root lifetime.
        let root = earliest_client(
            clients
                .into_iter()
                .filter(|pid| processes.contains_key(pid))
                .filter_map(|pid| {
                    let process = process::shell_process(pid).ok()?;
                    let created = process::created_at(process.0).ok()?;
                    Some((created, pid))
                }),
            std::process::id(),
        );
        if let Some((created, shell_pid)) = root {
            sessions.push(ConsoleSession {
                shell_pid,
                created,
                shell: processes[&shell_pid].1.clone(),
                console,
                terminal: host.0 as usize,
            });
        }
    }
    unsafe {
        let _ = FreeConsole();
    }
    sessions.sort_by_key(|s| s.shell_pid);
    Ok(DiscoveryReport {
        sessions,
        elevated_terminal_pids,
    })
}

struct Worker {
    session: ConsoleSession,
    process: OwnedHandle,
}

fn ready_name(console: usize, created: u64) -> AppResult<Vec<u16>> {
    Ok(wide(format!(
        "Local\\Cuesor-auto-{}-{console}-{created}-ready",
        process::identity()?
    )))
}

fn is_ready(session: &ConsoleSession) -> bool {
    let Ok(name) = ready_name(session.console, session.created) else {
        return false;
    };
    unsafe { OpenEventW(SYNCHRONIZATION_SYNCHRONIZE, false, PCWSTR(name.as_ptr())) }
        .map(|h| {
            drop(OwnedHandle(h));
            true
        })
        .unwrap_or(false)
}

pub fn start_controller(controller_pid: u32, data_dir: &Path) -> AppResult<OwnedHandle> {
    let owner = process::shell_process(controller_pid)?;
    let executable = std::env::current_exe()?.with_file_name("cuesor-engine.exe");
    if !executable.is_file() {
        return Err(crate::i18n::messages::engine_missing(crate::i18n::Locale::current()).into());
    }
    let args = vec![
        executable.into_os_string(),
        "_agent".into(),
        controller_pid.to_string().into(),
        process::created_at(owner.0)?.to_string().into(),
        data_dir.as_os_str().to_owned(),
    ];
    process::spawn_hidden(&args)
}

fn run(controller_pid: u32, created: u64, directory: &Path) -> AppResult<()> {
    let controller = checked_process(controller_pid, created)?;
    let mut workers: Vec<Worker> = Vec::new();
    let mut bridge: Option<crate::bridge::BridgeServer> = None;
    while unsafe { WaitForSingleObject(controller.0, 0) } == WAIT_TIMEOUT {
        workers
            .retain(|worker| unsafe { WaitForSingleObject(worker.process.0, 0) } == WAIT_TIMEOUT);
        let mut errors = Vec::new();
        let mut elevated_terminal_pids = Vec::new();

        let options = Settings::load(directory).unwrap_or_default();
        if options.obsidian_enabled && bridge.is_none() {
            match crate::bridge::BridgeServer::start_with_dir(crate::bridge::DEFAULT_BRIDGE_PORT, Some(directory.to_path_buf())) {
                Ok(server) => bridge = Some(server),
                Err(error) => errors.push(crate::i18n::messages::obsidian_service_failed(crate::i18n::Locale::current(), &error)),
            }
        } else if !options.obsidian_enabled && bridge.is_some() {
            if let Some(mut b) = bridge.take() {
                b.stop();
            }
        }
        let obsidian_clients = bridge.as_ref().map(|b| b.client_count()).unwrap_or(0);

        match discover_report() {
            Ok(report) => {
                elevated_terminal_pids = report.elevated_terminal_pids;
                for discovered in report.sessions {
                    if workers.iter().any(|worker| {
                        worker.session.console == discovered.console
                            && worker.session.created == discovered.created
                    }) {
                        continue;
                    }
                    let args = vec![
                        std::env::current_exe()?.into_os_string(),
                        "_auto-worker".into(),
                        controller_pid.to_string().into(),
                        created.to_string().into(),
                        directory.as_os_str().to_owned(),
                        discovered.shell_pid.to_string().into(),
                        discovered.created.to_string().into(),
                        discovered.console.to_string().into(),
                    ];
                    match process::spawn_hidden(&args) {
                        Ok(process) => workers.push(Worker {
                            session: discovered,
                            process,
                        }),
                        Err(error) => errors.push(error.to_string()),
                    }
                }
            }
            Err(error) => errors.push(error.to_string()),
        }
        if let Ok(bytes) = std::fs::read(directory.join("last-worker-error.json")) {
            if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) {
                if value["controller_pid"].as_u64() == Some(controller_pid as u64) {
                    if let Some(error) = value["error"].as_str() {
                        errors.push(error.to_owned());
                    }
                }
            }
        }
        let active: Vec<_> = workers.iter().filter(|w| is_ready(&w.session)).collect();
        let status = RuntimeStatus {
            controller_pid,
            sessions: active.iter().map(|w| w.session.clone()).collect(),
            worker_pids: active
                .iter()
                .map(|w| unsafe { GetProcessId(w.process.0) })
                .collect(),
            errors,
            elevated_terminal_pids,
            obsidian_enabled: options.obsidian_enabled,
            obsidian_clients,
        };
        settings::write_json(&directory.join("runtime.json"), &status)?;
        if unsafe { WaitForSingleObject(controller.0, 2000) } == WAIT_OBJECT_0 {
            break;
        }
    }
    if let Some(mut b) = bridge.take() {
        b.stop();
    }
    // Workers watch the controller too. Wait for their graceful color restoration.
    for worker in workers {
        unsafe {
            WaitForSingleObject(worker.process.0, 1500);
        }
    }
    Ok(())
}

fn checked_process(pid: u32, created: u64) -> AppResult<OwnedHandle> {
    let process = process::shell_process(pid)?;
    if process::created_at(process.0)? != created {
        return Err("process identity changed".into());
    }
    Ok(process)
}

unsafe extern "system" fn control(kind: u32) -> windows::Win32::Foundation::BOOL {
    (kind == CTRL_C_EVENT || kind == CTRL_BREAK_EVENT).into()
}

fn worker(
    controller_pid: u32,
    created: u64,
    directory: &Path,
    shell_pid: u32,
    shell_created: u64,
    console: usize,
) -> AppResult<()> {
    let controller = checked_process(controller_pid, created)?;
    let shell = checked_process(shell_pid, shell_created)?;
    let mutex_name = wide(format!(
        "Local\\Cuesor-auto-{}-{console}-{shell_created}",
        process::identity()?
    ));
    let mutex = OwnedHandle(unsafe { CreateMutexW(None, false, PCWSTR(mutex_name.as_ptr()))? });
    if unsafe { GetLastError() } == ERROR_ALREADY_EXISTS {
        return Ok(());
    }
    platform::attach(shell_pid)?;
    if unsafe { GetConsoleWindow() }.0 as usize != console
        || !platform::is_terminal(platform::terminal_host())
    {
        return Err("console identity changed".into());
    }
    unsafe {
        SetConsoleCtrlHandler(Some(control), true)?;
    }
    let Some(_console_lease) = process::console_lease()? else {
        // Retry discovery after the current console owner exits.
        return Ok(());
    };
    let mut options = Settings::load(directory)?;
    let ready = ready_name(console, shell_created)?;
    let _ready = OwnedHandle(unsafe { CreateEventW(None, true, false, PCWSTR(ready.as_ptr()))? });
    let mut state = ColorState::default();
    let mut force_reset = !options.enabled;
    let mut force_color = false;
    let mut reload = Instant::now();
    let handles = [controller.0, shell.0];
    loop {
        if reload.elapsed() >= Duration::from_millis(500) {
            if let Ok(next) = Settings::load(directory) {
                if next != options {
                    force_reset = !next.enabled;
                    force_color = true;
                    options = next;
                }
            }
            reload = Instant::now();
        }
        let host = platform::terminal_host();
        let sample = platform::sample(host);
        let mode = if options.enabled {
            sample.mode
        } else {
            ImeMode::Unknown
        };
        if force_reset {
            if platform::write_sequence(crate::model::RESET_CURSOR).is_ok() {
                state.applied(ImeMode::Unknown, host.0 as usize);
                force_reset = false;
                force_color = false;
            }
        } else if let Some(mode) = if force_color && mode != ImeMode::Unknown {
            Some(mode)
        } else {
            state.pending(mode, host.0 as usize)
        } {
            if platform::write_sequence(&options.colors()?.sequence(mode)).is_ok() {
                state.applied(mode, host.0 as usize);
                force_color = false;
            }
        }
        let active = sample.focus.is_some_and(|f| f.foreground == host);
        let wait =
            unsafe { WaitForMultipleObjects(&handles, false, if active { 100 } else { 500 }) };
        if wait != WAIT_TIMEOUT {
            break;
        }
    }
    if state.is_colored() {
        let _ = platform::write_sequence(crate::model::RESET_CURSOR);
    }
    drop(mutex);
    Ok(())
}

pub fn internal_command(args: Vec<String>) -> Option<AppResult<()>> {
    match args.first().map(String::as_str) {
        Some("discover") => Some((|| {
            if args.len() != 2 {
                return Err("discover requires exactly one JSON report path".into());
            }
            let report = args.get(1).ok_or("discover requires a JSON report path")?;
            settings::write_json(Path::new(report), &discover()?)?;
            Ok(())
        })()),
        Some("diagnose") => Some((|| {
            if args.len() != 2 {
                return Err("diagnose requires exactly one JSON report path".into());
            }
            let report = args.get(1).ok_or("diagnose requires a JSON report path")?;
            settings::write_json(Path::new(report), &discover_report()?)?;
            Ok(())
        })()),
        Some("_agent") | Some("_auto-worker") => Some((|| {
            let is_worker = args[0] == "_auto-worker";
            if args.len() != if is_worker { 7 } else { 4 } {
                return Err("invalid controller arguments".into());
            }
            let pid = args[1].parse()?;
            let created = args[2].parse()?;
            let directory = PathBuf::from(&args[3]);
            if is_worker {
                let result = worker(
                    pid,
                    created,
                    &directory,
                    args[4].parse()?,
                    args[5].parse()?,
                    args[6].parse()?,
                );
                if let Err(error) = &result {
                    let _ = settings::write_json(
                        &directory.join("last-worker-error.json"),
                        &serde_json::json!({"controller_pid":pid,"worker_pid":std::process::id(),"shell_pid":args[4],"error":error.to_string()}),
                    );
                }
                result
            } else {
                run(pid, created, &directory)
            }
        })()),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn newly_opened_shell_does_not_bind_to_the_older_discovery_process() {
        assert_eq!(
            earliest_client([(100, 7), (200, 42)].into_iter(), 7),
            Some((200, 42))
        );
        assert_eq!(earliest_client([(100, 7)].into_iter(), 7), None);
    }
}
