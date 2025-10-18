#[cfg(target_os = "windows")]
mod bridge;
#[cfg(target_os = "windows")]
mod config;
#[cfg(target_os = "windows")]
mod logging;
#[cfg(target_os = "windows")]
mod pipe;
#[cfg(target_os = "windows")]
mod tcp;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "windows")]
mod wsl;

#[cfg(target_os = "windows")]
fn main() {
    if let Err(err) = windows::run() {
        eprintln!("substrate-forwarder failed: {err}");
        std::process::exit(1);
    }
}

#[cfg(not(target_os = "windows"))]
fn main() {
    eprintln!("substrate-forwarder is only supported on Windows hosts.");
}
