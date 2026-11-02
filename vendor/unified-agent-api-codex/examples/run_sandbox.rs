use std::{env, ffi::OsString};

use codex::{CodexClient, SandboxCommandRequest, SandboxPlatform};

fn default_platform() -> SandboxPlatform {
    match env::consts::OS {
        "macos" => SandboxPlatform::Macos,
        "windows" => SandboxPlatform::Windows,
        _ => SandboxPlatform::Linux,
    }
}

fn parse_platform(raw: &OsString) -> Option<SandboxPlatform> {
    let lower = raw.to_string_lossy().to_ascii_lowercase();
    match lower.as_str() {
        "macos" | "seatbelt" => Some(SandboxPlatform::Macos),
        "linux" | "landlock" => Some(SandboxPlatform::Linux),
        "windows" => Some(SandboxPlatform::Windows),
        _ => None,
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Usage: cargo run -p unified-agent-api-codex --example run_sandbox -- [platform] [--full-auto] [--log-denials] -- <command...>
    // Defaults platform from the host OS when omitted. macOS-only `--log-denials` is ignored on other platforms.
    // Maps to: codex sandbox <platform> [--full-auto] [--log-denials] [--config/--enable/--disable] -- <command...>
    let args: Vec<OsString> = env::args_os().skip(1).collect();
    let (platform, mut index) = match args.first().and_then(parse_platform) {
        Some(platform) => (platform, 1),
        None => (default_platform(), 0),
    };

    let mut full_auto = false;
    let mut log_denials = false;
    let mut command: Vec<OsString> = Vec::new();

    while index < args.len() {
        let arg = &args[index];
        if arg == "--" {
            command.extend_from_slice(&args[index + 1..]);
            break;
        } else if arg == "--full-auto" {
            full_auto = true;
        } else if arg == "--log-denials" {
            log_denials = true;
        } else {
            // First non-flag token starts the command list; consume the rest.
            command.extend_from_slice(&args[index..]);
            break;
        }
        index += 1;
    }

    if command.is_empty() {
        eprintln!("usage: run_sandbox [platform] [--full-auto] [--log-denials] -- <command...>");
        eprintln!("example: run_sandbox linux --full-auto -- echo \"hello from sandbox\"");
        return Ok(());
    }

    let client = CodexClient::builder()
        .mirror_stdout(false)
        .quiet(true)
        .build();

    let run = client
        .run_sandbox(
            SandboxCommandRequest::new(platform, command)
                .full_auto(full_auto)
                .log_denials(log_denials),
        )
        .await?;

    println!("exit code: {:?}", run.status.code());
    if !run.stdout.is_empty() {
        println!("stdout:\n{}", run.stdout);
    }
    if !run.stderr.is_empty() {
        eprintln!("stderr:\n{}", run.stderr);
    }

    Ok(())
}
