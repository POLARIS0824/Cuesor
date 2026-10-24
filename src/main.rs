use cuesor::{agent, cli, model, platform, session};

use cli::{Command, Probe};
use model::Colors;
use platform::AppResult;
use std::io::Write;
use std::time::{Duration, Instant};

fn main() {
    if let Err(error) = execute() {
        if !std::env::args().nth(1).is_some_and(|arg| arg == "_worker") {
            eprintln!("cuesor-engine: {error}");
        }
        std::process::exit(1);
    }
}

fn execute() -> AppResult<()> {
    if let Some(result) = agent::internal_command(std::env::args().skip(1).collect()) {
        return result;
    }
    match cli::parse(std::env::args().skip(1))? {
        Command::Start(options) => session::start(options),
        Command::Worker {
            options,
            session,
            created,
        } => session::worker(options, session, created),
        Command::Stop => session::stop(),
        Command::Status => {
            println!("{{\"running\":{}}}", session::status()?);
            Ok(())
        }
        Command::Probe(options) => probe(options),
        Command::Reset => platform::write_sequence(model::RESET_CURSOR),
        Command::Version => {
            println!("cuesor-engine {}", env!("CARGO_PKG_VERSION"));
            Ok(())
        }
        Command::Help => {
            println!("{}", cli::HELP);
            Ok(())
        }
    }
}

fn probe(options: Probe) -> AppResult<()> {
    let Probe {
        watch,
        report,
        mut color,
        follow,
    } = options;
    let mut writer: Box<dyn Write> = match report {
        Some(path) => Box::new(std::fs::File::create(path)?),
        None => Box::new(std::io::stdout()),
    };
    let started = Instant::now();
    let mut previous = String::new();
    let mut applied = None;
    loop {
        let host = platform::terminal_host();
        let sample = platform::sample(follow.then_some(host));
        let foreground = sample.focus.map(|s| s.foreground.0 as usize).unwrap_or(0);
        let focused = sample.focus.map(|s| s.focused.0 as usize).unwrap_or(0);
        let layout = sample.focus.map(|s| s.layout).unwrap_or(0);
        let vt = platform::output_mode().ok();
        let fields = format!("\"mode\":\"{}\",\"stable\":{},\"host\":{},\"host_class\":{},\"foreground\":{},\"focused\":{},\"layout\":{},\"ime_hwnd\":{},\"open\":{},\"conversion\":{},\"output_mode\":{}", sample.mode, sample.stable, host.0 as usize, json_string(&platform::class_name(host)), foreground, focused, layout, sample.ime_hwnd.0 as usize, json_option(sample.open), json_option(sample.conversion), json_option(vt.map(|v| v as usize)));
        if fields != previous {
            writeln!(
                writer,
                "{{\"elapsed_ms\":{},{} }}",
                started.elapsed().as_millis(),
                fields
            )?;
            writer.flush()?;
            previous = fields;
        }
        if let Some(mode) = color.take() {
            platform::write_sequence(&Colors::default().sequence(mode))?;
        }
        if follow && applied != Some(sample.mode) {
            platform::write_sequence(&Colors::default().sequence(sample.mode))?;
            writeln!(
                writer,
                "{{\"elapsed_ms\":{},\"color_written\":\"{}\"}}",
                started.elapsed().as_millis(),
                sample.mode
            )?;
            writer.flush()?;
            applied = Some(sample.mode);
        }
        if started.elapsed() >= Duration::from_secs(watch) {
            break;
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    if follow {
        platform::write_sequence(model::RESET_CURSOR)?;
    }
    Ok(())
}

fn json_option(value: Option<usize>) -> String {
    value
        .map(|v| v.to_string())
        .unwrap_or_else(|| "null".to_owned())
}

fn json_string(value: &str) -> String {
    let mut out = String::from("\"");
    for c in value.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c < ' ' => {
                use std::fmt::Write;
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}
