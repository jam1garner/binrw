fn main() {
    println!("cargo:rustc-check-cfg=cfg(coverage)");
    println!("cargo:rustc-check-cfg=cfg(coverage_nightly)");
    println!("cargo:rustc-check-cfg=cfg(nightly)");
    if is_nightly().unwrap_or(false) {
        println!("cargo:rustc-cfg=nightly");
    }
}

fn is_nightly() -> Option<bool> {
    let rustc = std::env::var_os("RUSTC")?;
    let output = std::process::Command::new(rustc)
        .arg("--version")
        .output()
        .ok()?;
    let version = core::str::from_utf8(&output.stdout).ok()?;
    let nightly = version.contains("nightly") || version.contains("dev");

    Some(nightly)
}
