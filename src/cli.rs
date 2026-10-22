use std::path::PathBuf;

use crate::model::{Color, Colors, ImeMode};

pub const HELP: &str = "Terminal IME — Windows Terminal native engine

For the tray and settings app, open TerminalIME.exe beside this executable.

  start --shell-pid PID [--cn-color #RRGGBB] [--en-color #RRGGBB] [--log FILE]
  stop
  status
  reset
  discover REPORT.json
  diagnose REPORT.json
  probe [--watch SECONDS] [--report FILE] [--color cn|en|reset] [--follow]

start returns after creating the worker; status reports whether it initialized.
Colors represent the active input mode of this WT window, shared by its panes.
Probe logs contain window/IME metadata only. --follow applies observed colors.
";

#[derive(Debug)]
pub enum Command {
    Start(Start),
    Worker {
        options: Start,
        session: String,
        created: u64,
    },
    Stop,
    Status,
    Reset,
    Probe(Probe),
    Help,
    Version,
}

#[derive(Clone, Debug)]
pub struct Start {
    pub shell_pid: u32,
    pub colors: Colors,
    pub log: Option<PathBuf>,
}

#[derive(Debug, Default)]
pub struct Probe {
    pub watch: u64,
    pub report: Option<PathBuf>,
    pub color: Option<ImeMode>,
    pub follow: bool,
}

pub fn parse(args: impl IntoIterator<Item = String>) -> Result<Command, String> {
    let mut args = args.into_iter();
    let Some(command) = args.next() else {
        return Ok(Command::Help);
    };
    let rest: Vec<String> = args.collect();
    if rest.iter().any(|s| s == "--help" || s == "-h") {
        return Ok(Command::Help);
    }
    match command.as_str() {
        "start" | "_worker" => {
            let mut shell_pid = None;
            let mut colors = Colors::default();
            let mut log = None;
            let mut session = None;
            let mut created = None;
            let mut seen = std::collections::HashSet::new();
            let mut iter = rest.into_iter();
            while let Some(key) = iter.next() {
                if !seen.insert(key.clone()) {
                    return Err(format!("duplicate option {key}"));
                }
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {key}"))?;
                match key.as_str() {
                    "--shell-pid" => {
                        shell_pid = Some(value.parse::<u32>().map_err(|_| "invalid shell PID")?)
                    }
                    "--cn-color" => colors.chinese = Color::parse(&value)?,
                    "--en-color" => colors.latin = Color::parse(&value)?,
                    "--log" => log = Some(PathBuf::from(value)),
                    "--session" if command == "_worker" => session = Some(value),
                    "--created" if command == "_worker" => {
                        created = Some(value.parse::<u64>().map_err(|_| "invalid creation time")?)
                    }
                    _ => return Err(format!("unknown {command} option {key}")),
                }
            }
            let shell_pid = shell_pid
                .filter(|pid| *pid != 0)
                .ok_or("--shell-pid requires a nonzero PID")?;
            let options = Start {
                shell_pid,
                colors,
                log,
            };
            if command == "start" {
                Ok(Command::Start(options))
            } else {
                Ok(Command::Worker {
                    options,
                    session: session.ok_or("missing --session")?,
                    created: created.ok_or("missing --created")?,
                })
            }
        }
        "probe" => {
            let mut options = Probe::default();
            let mut seen = std::collections::HashSet::new();
            let mut iter = rest.into_iter();
            while let Some(key) = iter.next() {
                if !seen.insert(key.clone()) {
                    return Err(format!("duplicate option {key}"));
                }
                if key == "--follow" {
                    options.follow = true;
                    continue;
                }
                let value = iter
                    .next()
                    .ok_or_else(|| format!("missing value for {key}"))?;
                match key.as_str() {
                    "--watch" => {
                        options.watch = value.parse::<u64>().map_err(|_| "invalid duration")?;
                        if options.watch > 3600 {
                            return Err("--watch must be at most 3600 seconds".into());
                        }
                    }
                    "--report" => options.report = Some(value.into()),
                    "--color" => {
                        options.color = Some(match value.as_str() {
                            "cn" => ImeMode::Chinese,
                            "en" => ImeMode::Latin,
                            "reset" => ImeMode::Unknown,
                            _ => return Err("--color must be cn, en, or reset".into()),
                        })
                    }
                    _ => return Err(format!("unknown probe option {key}")),
                }
            }
            if options.follow && options.color.is_some() {
                return Err("--follow and --color are mutually exclusive".into());
            }
            Ok(Command::Probe(options))
        }
        _ if !rest.is_empty() => Err(format!("unexpected arguments for {command}")),
        "stop" => Ok(Command::Stop),
        "status" => Ok(Command::Status),
        "reset" => Ok(Command::Reset),
        "help" | "--help" | "-h" => Ok(Command::Help),
        "--version" | "-V" => Ok(Command::Version),
        _ => Err(format!("unknown command {command:?}; use --help")),
    }
}

pub fn session_id(value: &str) -> Result<String, String> {
    let text = value
        .strip_prefix('{')
        .and_then(|v| v.strip_suffix('}'))
        .unwrap_or(value);
    if text.len() != 36
        || !text.bytes().enumerate().all(|(i, c)| {
            if [8, 13, 18, 23].contains(&i) {
                c == b'-'
            } else {
                c.is_ascii_hexdigit()
            }
        })
    {
        return Err(
            "WT_SESSION is missing or is not a GUID; start this from Windows Terminal".into(),
        );
    }
    Ok(text.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;
    fn command(text: &[&str]) -> Result<Command, String> {
        parse(text.iter().map(|s| s.to_string()))
    }

    #[test]
    fn rejects_invalid_or_ambiguous_launch_arguments() {
        for args in [
            vec!["start"],
            vec!["start", "--shell-pid", "0"],
            vec!["start", "--shell-pid", "12", "--shell-pid", "13"],
            vec!["stop", "--follow"],
            vec!["probe", "--follow", "--color", "cn"],
            vec!["probe", "--watch", "3601"],
        ] {
            assert!(command(&args).is_err(), "{args:?}");
        }
        assert!(matches!(
            command(&["start", "--shell-pid", "12", "--cn-color", "#ff7800"]).unwrap(),
            Command::Start(_)
        ));
        assert!(matches!(
            command(&["probe", "--follow", "--watch", "20"]).unwrap(),
            Command::Probe(_)
        ));
    }

    #[test]
    fn session_ids_cannot_escape_the_named_object_namespace() {
        assert!(session_id("x\\Global\\anything").is_err());
        assert!(session_id("00000000-0000-0000-0000-00000000000/").is_err());
        assert_eq!(
            session_id("{01234567-89AB-CDEF-0123-456789ABCDEF}").unwrap(),
            "01234567-89ab-cdef-0123-456789abcdef"
        );
    }
}
