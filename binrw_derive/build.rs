use std::env;
use std::process::Command;
use std::str;

fn main() {
    if is_nightly().unwrap_or(false) {
        println!("cargo:rustc-cfg=nightly");
    }
}

fn is_nightly() -> Option<bool> {
    let rustc = env::var_os("RUSTC")?;
    let output = Command::new(rustc).arg("--version").output().ok()?;
    let version = str::from_utf8(&output.stdout).ok()?;
    let nightly = version.contains("nightly") || version.contains("dev");

    Some(nightly)
}
