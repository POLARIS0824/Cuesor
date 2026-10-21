use std::ffi::OsString;
use std::fs::File;
use std::io::Write;
use std::mem::size_of;
use std::os::windows::ffi::OsStrExt;
use std::time::{Duration, Instant};

use windows::core::{PCWSTR, PWSTR};
use windows::Win32::Foundation::{
    GetLastError, BOOL, ERROR_ALREADY_EXISTS, ERROR_FILE_NOT_FOUND, FILETIME, HANDLE,
    WAIT_OBJECT_0, WAIT_TIMEOUT,
};
use windows::Win32::Security::{
    GetLengthSid, GetTokenInformation, TokenIntegrityLevel, TokenUser, TOKEN_INFORMATION_CLASS,
    TOKEN_MANDATORY_LABEL, TOKEN_QUERY, TOKEN_USER,
};
use windows::Win32::System::Console::{SetConsoleCtrlHandler, CTRL_BREAK_EVENT, CTRL_C_EVENT};
use windows::Win32::System::Threading::{
    CreateEventW, CreateMutexW, CreateProcessW, GetCurrentProcess, GetProcessTimes, OpenEventW,
    OpenMutexW, OpenProcess, OpenProcessToken, SetEvent, WaitForMultipleObjects,
    WaitForSingleObject, CREATE_NO_WINDOW, EVENT_MODIFY_STATE, PROCESS_INFORMATION,
    PROCESS_QUERY_LIMITED_INFORMATION, PROCESS_SYNCHRONIZE, STARTUPINFOW,
    SYNCHRONIZATION_SYNCHRONIZE,
};

use crate::cli::{session_id, Start};
use crate::model::ColorState;
use crate::platform::{self, wide, AppResult, OwnedHandle};

struct Names {
    mutex: Vec<u16>,
    stop: Vec<u16>,
}

impl Names {
    fn new(session: &str) -> AppResult<Self> {
        let key = format!(
            "Local\\TerminalIME-v1-{}-{}",
            identity()?,
            session_id(session)?
        );
        Ok(Self {
            mutex: wide(&format!("{key}-worker")),
            stop: wide(&format!("{key}-stop")),
        })
    }
}

fn token_information(token: HANDLE, kind: TOKEN_INFORMATION_CLASS) -> AppResult<Vec<usize>> {
    let mut size = 0;
    unsafe {
        let _ = GetTokenInformation(token, kind, None, 0, &mut size);
    }
    if size == 0 {
        return Err("could not determine token information size".into());
    }
    let mut buffer = vec![0usize; (size as usize).div_ceil(size_of::<usize>())];
    unsafe {
        GetTokenInformation(
            token,
            kind,
            Some(buffer.as_mut_ptr().cast()),
            size,
            &mut size,
        )?;
    }
    Ok(buffer)
}

fn identity() -> AppResult<String> {
    let mut raw = HANDLE::default();
    unsafe {
        OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut raw)?;
    }
    let token = OwnedHandle(raw);
    let user = token_information(token.0, TokenUser)?;
    let integrity = token_information(token.0, TokenIntegrityLevel)?;
    let mut key = String::new();
    unsafe {
        let user_sid = (*(user.as_ptr().cast::<TOKEN_USER>())).User.Sid;
        let level_sid = (*(integrity.as_ptr().cast::<TOKEN_MANDATORY_LABEL>()))
            .Label
            .Sid;
        for sid in [user_sid, level_sid] {
            for byte in std::slice::from_raw_parts(sid.0.cast::<u8>(), GetLengthSid(sid) as usize) {
                use std::fmt::Write;
                write!(key, "{byte:02x}")?;
            }
            key.push('-');
        }
    }
    Ok(key)
}

pub fn current_session() -> AppResult<String> {
    Ok(session_id(
        &std::env::var("WT_SESSION").unwrap_or_default(),
    )?)
}

fn open_mutex(names: &Names) -> AppResult<Option<OwnedHandle>> {
    match unsafe {
        OpenMutexW(
            SYNCHRONIZATION_SYNCHRONIZE,
            false,
            PCWSTR(names.mutex.as_ptr()),
        )
    } {
        Ok(handle) => Ok(Some(OwnedHandle(handle))),
        Err(error)
            if error.code() == windows::core::HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) =>
        {
            Ok(None)
        }
        Err(error) => Err(error.into()),
    }
}

fn shell_process(pid: u32) -> AppResult<OwnedHandle> {
    Ok(OwnedHandle(unsafe {
        OpenProcess(
            PROCESS_SYNCHRONIZE | PROCESS_QUERY_LIMITED_INFORMATION,
            false,
            pid,
        )?
    }))
}

fn created_at(process: HANDLE) -> AppResult<u64> {
    let mut created = FILETIME::default();
    let mut exited = FILETIME::default();
    let mut kernel = FILETIME::default();
    let mut user = FILETIME::default();
    unsafe {
        GetProcessTimes(process, &mut created, &mut exited, &mut kernel, &mut user)?;
    }
    Ok((created.dwHighDateTime as u64) << 32 | created.dwLowDateTime as u64)
}

pub fn start(options: Start) -> AppResult<()> {
    let session = current_session()?;
    let names = Names::new(&session)?;
    if open_mutex(&names)?.is_some() {
        return Ok(());
    }
    let shell = shell_process(options.shell_pid)?;
    let created = created_at(shell.0)?;
    let executable = std::env::current_exe()?;
    let mut arguments = vec![executable.clone().into_os_string()];
    arguments.extend(
        [
            "_worker",
            "--shell-pid",
            &options.shell_pid.to_string(),
            "--session",
            &session,
            "--created",
            &created.to_string(),
            "--cn-color",
            &format!("{:06X}", options.colors.chinese.0),
            "--en-color",
            &format!("{:06X}", options.colors.latin.0),
        ]
        .into_iter()
        .map(OsString::from),
    );
    if let Some(path) = options.log {
        arguments.push("--log".into());
        arguments.push(path.into_os_string());
    }
    let executable: Vec<u16> = executable
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect();
    let mut command = command_line(&arguments);
    let startup = STARTUPINFOW {
        cb: size_of::<STARTUPINFOW>() as u32,
        ..Default::default()
    };
    let mut process = PROCESS_INFORMATION::default();
    // Do not inherit ANY handles. Inheriting Nu's pipe handles makes `complete`
    // wait for the long-lived worker even after this short-lived launcher exits.
    unsafe {
        CreateProcessW(
            PCWSTR(executable.as_ptr()),
            PWSTR(command.as_mut_ptr()),
            None,
            None,
            false,
            CREATE_NO_WINDOW,
            None,
            None,
            &startup,
            &mut process,
        )?;
    }
    let _process = OwnedHandle(process.hProcess);
    let _thread = OwnedHandle(process.hThread);
    Ok(())
}

fn command_line(arguments: &[OsString]) -> Vec<u16> {
    let mut output = Vec::new();
    for (index, argument) in arguments.iter().enumerate() {
        if index != 0 {
            output.push(b' ' as u16);
        }
        output.push(b'"' as u16);
        let mut slashes = 0;
        for unit in argument.encode_wide() {
            if unit == b'\\' as u16 {
                slashes += 1;
                continue;
            }
            let count = if unit == b'"' as u16 {
                slashes * 2 + 1
            } else {
                slashes
            };
            output.extend(std::iter::repeat_n(b'\\' as u16, count));
            output.push(unit);
            slashes = 0;
        }
        output.extend(std::iter::repeat_n(b'\\' as u16, slashes * 2));
        output.push(b'"' as u16);
    }
    output.push(0);
    output
}

pub fn status() -> AppResult<bool> {
    let names = Names::new(&current_session()?)?;
    match unsafe {
        OpenEventW(
            SYNCHRONIZATION_SYNCHRONIZE,
            false,
            PCWSTR(names.stop.as_ptr()),
        )
    } {
        Ok(handle) => {
            let event = OwnedHandle(handle);
            Ok(unsafe { WaitForSingleObject(event.0, 0) } == WAIT_TIMEOUT)
        }
        Err(error)
            if error.code() == windows::core::HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) =>
        {
            Ok(false)
        }
        Err(error) => Err(error.into()),
    }
}

pub fn stop() -> AppResult<()> {
    let names = Names::new(&current_session()?)?;
    // A very recent start may still be between mutex acquisition and event creation.
    let deadline = Instant::now() + Duration::from_secs(1);
    loop {
        match unsafe { OpenEventW(EVENT_MODIFY_STATE, false, PCWSTR(names.stop.as_ptr())) } {
            Ok(handle) => {
                let event = OwnedHandle(handle);
                unsafe {
                    SetEvent(event.0)?;
                }
                drop(event);
                loop {
                    if open_mutex(&names)?.is_none() {
                        return Ok(());
                    }
                    if Instant::now() >= deadline {
                        return Err("worker did not stop within one second".into());
                    }
                    std::thread::sleep(Duration::from_millis(10));
                }
            }
            Err(error)
                if error.code() == windows::core::HRESULT::from_win32(ERROR_FILE_NOT_FOUND.0) =>
            {
                if open_mutex(&names)?.is_none() || Instant::now() >= deadline {
                    return Ok(());
                }
                std::thread::sleep(Duration::from_millis(10));
            }
            Err(error) => return Err(error.into()),
        }
    }
}

unsafe extern "system" fn control(kind: u32) -> BOOL {
    BOOL::from(kind == CTRL_C_EVENT || kind == CTRL_BREAK_EVENT)
}

pub fn worker(options: Start, session: String, created: u64) -> AppResult<()> {
    let path = options.log.clone();
    let result = run_worker(options, session, created);
    if let (Err(error), Some(path)) = (&result, path) {
        if let Ok(mut file) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
        {
            let _ = writeln!(
                file,
                "{{\"worker_error\":{}}}",
                crate::json_string(&error.to_string())
            );
        }
    }
    result
}

fn run_worker(options: Start, session: String, created: u64) -> AppResult<()> {
    let names = Names::new(&session)?;
    let raw = unsafe { CreateMutexW(None, false, PCWSTR(names.mutex.as_ptr()))? };
    let existed = unsafe { GetLastError() } == ERROR_ALREADY_EXISTS;
    let _singleton = OwnedHandle(raw);
    if existed {
        return Ok(());
    }

    let mut log = options.log.as_ref().map(File::create).transpose()?;
    let shell = shell_process(options.shell_pid)?;
    if created_at(shell.0)? != created {
        return Err("shell PID was reused before worker initialization".into());
    }
    platform::attach(options.shell_pid)?;
    unsafe {
        SetConsoleCtrlHandler(Some(control), true)?;
    }
    if !platform::is_terminal(platform::terminal_host()) {
        return Err("attached console has no recognized Windows Terminal host".into());
    }
    let stop =
        OwnedHandle(unsafe { CreateEventW(None, true, false, PCWSTR(names.stop.as_ptr()))? });
    let handles = [stop.0, shell.0];
    let started = Instant::now();
    record(
        &mut log,
        format!(
            "{{\"started\":true,\"worker_pid\":{},\"shell_pid\":{},\"session\":{}}}",
            std::process::id(),
            options.shell_pid,
            crate::json_string(&session)
        ),
    )?;
    let mut state = ColorState::default();
    let mut last_observation = None;
    let mut retry_at = Instant::now();
    let mut last_error = String::new();

    let result = (|| -> AppResult<()> {
        loop {
            let host = platform::terminal_host();
            let sample = platform::sample(Some(host));
            let foreground = sample.focus.is_some_and(|f| f.foreground == host);
            let observed = (sample.mode, host.0 as usize);
            if last_observation != Some(observed) {
                record(
                    &mut log,
                    format!(
                        "{{\"elapsed_ms\":{},\"observed\":\"{}\",\"host\":{}}}",
                        started.elapsed().as_millis(),
                        sample.mode,
                        host.0 as usize
                    ),
                )?;
                last_observation = Some(observed);
            }
            if let Some(mode) = state.pending(sample.mode, host.0 as usize) {
                if Instant::now() >= retry_at {
                    match platform::write_sequence(&options.colors.sequence(mode)) {
                        Ok(()) => {
                            state.applied(mode, host.0 as usize);
                            last_error.clear();
                            record(
                                &mut log,
                                format!(
                                    "{{\"elapsed_ms\":{},\"written\":\"{}\"}}",
                                    started.elapsed().as_millis(),
                                    mode
                                ),
                            )?;
                        }
                        Err(error) => {
                            let message = error.to_string();
                            if message != last_error {
                                record(
                                    &mut log,
                                    format!(
                                        "{{\"elapsed_ms\":{},\"write_error\":{}}}",
                                        started.elapsed().as_millis(),
                                        crate::json_string(&message)
                                    ),
                                )?;
                                last_error = message;
                            }
                            retry_at = Instant::now() + Duration::from_secs(1);
                        }
                    }
                }
            }
            match unsafe {
                WaitForMultipleObjects(&handles, false, if foreground { 100 } else { 500 })
            } {
                WAIT_TIMEOUT => (),
                value if value == WAIT_OBJECT_0 || value.0 == WAIT_OBJECT_0.0 + 1 => break,
                _ => return Err(windows::core::Error::from_win32().into()),
            }
        }
        Ok(())
    })();
    if state.is_colored() {
        let _ = platform::write_sequence(crate::model::RESET_CURSOR);
    }
    record(
        &mut log,
        format!(
            "{{\"elapsed_ms\":{},\"stopped\":true}}",
            started.elapsed().as_millis()
        ),
    )?;
    result
}

fn record(log: &mut Option<File>, line: String) -> AppResult<()> {
    if let Some(file) = log {
        writeln!(file, "{line}")?;
        file.flush()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn native_spawn_preserves_spaces_and_trailing_backslashes() {
        let actual = command_line(&[
            "D:\\工具 路径\\app.exe".into(),
            "".into(),
            "D:\\logs folder\\".into(),
            "a\"b".into(),
        ]);
        assert_eq!(
            String::from_utf16(&actual[..actual.len() - 1]).unwrap(),
            "\"D:\\工具 路径\\app.exe\" \"\" \"D:\\logs folder\\\\\" \"a\\\"b\""
        );
    }
}
