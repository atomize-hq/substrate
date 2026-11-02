//! Demonstrates `codex debug ...` surfaces via the wrapper.
//!
//! Usage:
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- help`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- help app-server`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- app-server`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- app-server help`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- app-server send-message-v2 "hello"`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- models [--bundled]`
//! - `cargo run -p unified-agent-api-codex --example debug_cmd -- prompt-input [--image <FILE> ...] [PROMPT]`
//!
//! Environment:
//! - `CODEX_BINARY` (optional): path to the `codex` CLI binary (defaults to `codex` in PATH).
//! - `CODEX_HOME` (optional): Codex home directory for config/auth.

use codex::{
    DebugAppServerHelpRequest, DebugAppServerRequest, DebugAppServerSendMessageV2Request,
    DebugHelpRequest, DebugModelsRequest, DebugPromptInputRequest,
};

#[path = "support/real_cli.rs"]
mod real_cli;

fn usage() {
    eprintln!("usage: debug_cmd <help|app-server ...|models [--bundled]|prompt-input [--image <FILE> ...] [PROMPT]>");
    eprintln!("examples:");
    eprintln!("  debug_cmd help");
    eprintln!("  debug_cmd help app-server");
    eprintln!("  debug_cmd app-server");
    eprintln!("  debug_cmd app-server help");
    eprintln!("  debug_cmd app-server send-message-v2 \"hello\"");
    eprintln!("  debug_cmd models --bundled");
    eprintln!("  debug_cmd prompt-input --image ./diagram.png \"summarize this prompt payload\"");
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        usage();
        return Ok(());
    }

    let client = real_cli::default_client();

    match args[0].to_ascii_lowercase().as_str() {
        "help" => {
            let command = args.drain(1..).collect::<Vec<_>>();
            let output = client
                .debug_help(DebugHelpRequest::new().command(command))
                .await?;
            print!("{}", output.stdout);
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
        }
        "app-server" => {
            if args.len() == 1 {
                let output = client
                    .debug_app_server(DebugAppServerRequest::new())
                    .await?;
                print!("{}", output.stdout);
                if !output.stderr.is_empty() {
                    eprintln!("{}", output.stderr);
                }
                return Ok(());
            }

            match args[1].to_ascii_lowercase().as_str() {
                "help" => {
                    let command = args.drain(2..).collect::<Vec<_>>();
                    let output = client
                        .debug_app_server_help(DebugAppServerHelpRequest::new().command(command))
                        .await?;
                    print!("{}", output.stdout);
                    if !output.stderr.is_empty() {
                        eprintln!("{}", output.stderr);
                    }
                }
                "send-message-v2" => {
                    if args.len() < 3 {
                        usage();
                        return Ok(());
                    }
                    let message = args.drain(2..).collect::<Vec<_>>().join(" ");
                    let output = client
                        .debug_app_server_send_message_v2(DebugAppServerSendMessageV2Request::new(
                            message,
                        ))
                        .await?;
                    print!("{}", output.stdout);
                    if !output.stderr.is_empty() {
                        eprintln!("{}", output.stderr);
                    }
                }
                _ => usage(),
            }
        }
        "models" => {
            let bundled = args.iter().skip(1).any(|arg| arg == "--bundled");
            let output = client
                .debug_models(DebugModelsRequest::new().bundled(bundled))
                .await?;
            print!("{}", output.stdout);
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
        }
        "prompt-input" => {
            let mut request = DebugPromptInputRequest::new();
            let mut idx = 1;
            let mut prompt_parts = Vec::new();
            while idx < args.len() {
                if args[idx] == "--image" {
                    idx += 1;
                    if idx >= args.len() {
                        usage();
                        return Ok(());
                    }
                    request = request.image(&args[idx]);
                } else {
                    prompt_parts.push(args[idx].clone());
                }
                idx += 1;
            }
            if !prompt_parts.is_empty() {
                request = request.prompt(prompt_parts.join(" "));
            }
            let output = client.debug_prompt_input(request).await?;
            print!("{}", output.stdout);
            if !output.stderr.is_empty() {
                eprintln!("{}", output.stderr);
            }
        }
        _ => usage(),
    }

    Ok(())
}
