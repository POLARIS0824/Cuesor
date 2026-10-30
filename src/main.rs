fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args == ["--version"] {
        println!("cuesor-engine {}", env!("CARGO_PKG_VERSION"));
        return;
    }
    let result = cuesor::agent::internal_command(args).unwrap_or_else(|| {
        Err(cuesor::i18n::messages::engine_helper_cli(cuesor::i18n::Locale::current()).into())
    });
    if let Err(error) = result {
        eprintln!("cuesor-engine: {error}");
        std::process::exit(1);
    }
}
