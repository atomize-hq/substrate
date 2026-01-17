use std::path::PathBuf;
use std::process::Command;

fn main() {
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }

    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").unwrap());
    let driver_path = manifest_dir
        .join("scripts")
        .join("dev")
        .join("substrate_shell_driver");

    let source_path = out_dir.join("substrate_shell_driver_windows.rs");
    let source = r#"
use std::ffi::{OsStr, OsString};
use std::path::{Path, PathBuf};
use std::process::{Command, ExitCode};

fn usage() {
    println!(
        "substrate_shell_driver - host-only wrapper around the substrate CLI\n\n\
Usage:\n  substrate_shell_driver [--bin <path>] [--help]\n  substrate_shell_driver [substrate args...]\n\n\
Options:\n  --bin <path>  Explicit substrate binary to run (env SUBSTRATE_BIN works too)\n  --help        Show this message\n\n\
By default the driver uses SUBSTRATE_BIN if set, otherwise it searches <repo>\\target\\debug\\substrate.exe\n\
and target\\release\\substrate.exe. The wrapper injects --no-world by default (unless --world/--no-world\n\
is already provided) to prevent world provisioning during tests and developer scripts."
    );
}

fn repo_root_from_exe(exe: &Path) -> Option<PathBuf> {
    let dev = exe.parent()?;
    let scripts = dev.parent()?;
    scripts.parent().map(|p| p.to_path_buf())
}

fn substrate_bin(repo_root: &Path, override_bin: Option<OsString>) -> Option<PathBuf> {
    if let Some(bin) = override_bin {
        return Some(PathBuf::from(bin));
    }

    if let Some(bin) = std::env::var_os("SUBSTRATE_BIN") {
        if !bin.is_empty() {
            return Some(PathBuf::from(bin));
        }
    }

    let debug = repo_root.join("target").join("debug").join("substrate.exe");
    if debug.is_file() {
        return Some(debug);
    }
    let release = repo_root.join("target").join("release").join("substrate.exe");
    if release.is_file() {
        return Some(release);
    }

    None
}

fn main() -> ExitCode {
    let mut args = std::env::args_os().skip(1);
    let mut bin_override: Option<OsString> = None;
    let mut passthrough: Vec<OsString> = Vec::new();

    while let Some(arg) = args.next() {
        match arg.to_string_lossy().as_ref() {
            "--bin" => {
                let Some(value) = args.next() else {
                    eprintln!("error: --bin requires a path");
                    return ExitCode::from(2);
                };
                bin_override = Some(value);
            }
            "--help" => {
                usage();
                return ExitCode::SUCCESS;
            }
            "--" => {
                passthrough.extend(args);
                break;
            }
            _ => passthrough.push(arg),
        }
    }

    let exe = match std::env::current_exe() {
        Ok(exe) => exe,
        Err(err) => {
            eprintln!("error: failed to locate current executable: {err}");
            return ExitCode::from(2);
        }
    };
    let Some(repo_root) = repo_root_from_exe(&exe) else {
        eprintln!("error: failed to locate repo root from {}", exe.display());
        return ExitCode::from(2);
    };

    let Some(substrate_bin) = substrate_bin(&repo_root, bin_override) else {
        eprintln!(
            "error: substrate binary not found. Build it (cargo build -p substrate) or set SUBSTRATE_BIN"
        );
        return ExitCode::from(2);
    };

    let mut needs_no_world = true;
    for arg in &passthrough {
        if arg == OsStr::new("--world") || arg == OsStr::new("--no-world") {
            needs_no_world = false;
            break;
        }
    }
    if needs_no_world {
        passthrough.insert(0, OsString::from("--no-world"));
    }

    let mut cmd = Command::new(substrate_bin);
    cmd.args(&passthrough);
    match cmd.status() {
        Ok(status) => ExitCode::from(status.code().unwrap_or(1) as u8),
        Err(err) => {
            eprintln!("error: failed to run substrate: {err}");
            ExitCode::from(2)
        }
    }
}
"#;
    std::fs::write(&source_path, source).expect("write substrate_shell_driver source");

    let rustc = std::env::var("RUSTC").unwrap_or_else(|_| "rustc".to_string());
    let exe_path = driver_path.with_extension("exe");
    let mut cmd = Command::new(rustc);
    cmd.arg(&source_path)
        .arg("--edition=2021")
        .arg("-C")
        .arg("opt-level=0")
        .arg("-o")
        .arg(&exe_path);

    let status = cmd
        .status()
        .expect("invoke rustc for substrate_shell_driver");
    if !status.success() {
        panic!(
            "failed to build substrate_shell_driver at {}",
            driver_path.display()
        );
    }

    let _ = std::fs::remove_file(&driver_path);
    std::fs::rename(&exe_path, &driver_path).expect("rename substrate_shell_driver.exe");
}
