// Give each GNU import-library invocation its own temporary prefix, in the
// compiler's writable output directory rather than a shared dependency directory.
use std::{ffi::OsString, path::PathBuf, process::Command};

fn main() {
    let mut arguments: Vec<OsString> = std::env::args_os().skip(1).collect();
    if let (Some(output), Some(prefix)) = (
        arguments.iter().position(|arg| arg == "-l"),
        arguments.iter().position(|arg| arg == "--temp-prefix"),
    ) {
        if let Some(path) = arguments.get(output + 1) {
            let mut temporary = PathBuf::from(path);
            temporary.set_file_name(format!("cuesor-dlltool-{}-", std::process::id()));
            if prefix + 1 < arguments.len() {
                arguments[prefix + 1] = temporary.into_os_string();
            }
        }
    }
    let executable =
        std::env::var_os("CUESOR_DLLTOOL").expect("real dlltool path is required");
    let status = Command::new(executable)
        .args(arguments)
        .status()
        .expect("could not start GNU dlltool");
    std::process::exit(status.code().unwrap_or(1));
}
