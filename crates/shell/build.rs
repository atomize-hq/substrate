fn main() {
    // Capture the Rust compiler version at build time
    let version = rustc_version::version()
        .map(|v| v.to_string())
        .unwrap_or_else(|_| "unknown".to_string());

    println!("cargo:rustc-env=SHIM_RUSTC_VERSION={version}");
}
