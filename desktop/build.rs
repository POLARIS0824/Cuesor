use std::{path::Path, process::Command};

fn main() {
    if std::env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("gnu") {
        // Rust's bundled MinGW contains fewer import libraries than a complete GCC install.
        if let Ok(output) = Command::new("gcc")
            .arg("-print-file-name=libktmw32.a")
            .output()
        {
            let library = String::from_utf8_lossy(&output.stdout).trim().to_owned();
            if Path::new(&library).is_file() {
                println!(
                    "cargo:rustc-link-search=native={}",
                    Path::new(&library).parent().unwrap().display()
                );
            }
        }
    }
    println!("cargo:rerun-if-changed=assets/app.rc");
    println!("cargo:rerun-if-changed=assets/app.ico");
    // gpui-pre-platform supplies the process manifest (Common Controls v6,
    // asInvoker and PerMonitorV2). Embedding a second one breaks GNU .rsrc merging.
    embed_resource::compile("assets/app.rc", embed_resource::NONE)
        .manifest_required()
        .expect("embed desktop icon");
}
