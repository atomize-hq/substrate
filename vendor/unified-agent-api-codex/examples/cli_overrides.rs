use std::{env, path::PathBuf, time::Duration};

use codex::{CodexClient, ExecRequest, SandboxMode};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Usage: cargo run -p unified-agent-api-codex --example cli_overrides -- "<prompt>" [cd]
    // Maps to: codex exec --config model_verbosity=high --config features.search=true
    // --config model_reasoning_effort=low --ask-for-approval on-request
    // --sandbox workspace-write --local-provider ollama --oss --enable builder-toggle
    // --disable legacy-flow --enable request-toggle --search [--cd <dir>]
    let mut args = env::args().skip(1);
    let prompt = args.next().expect("usage: cli_overrides <prompt> [cd]");
    let cd = args.next().map(PathBuf::from);

    let mut builder = CodexClient::builder()
        .timeout(Duration::from_secs(45))
        .mirror_stdout(false)
        // Some Codex versions surface approval/sandbox as global flags; the wrapper applies them
        // via its shared override plumbing.
        .sandbox_mode(SandboxMode::WorkspaceWrite)
        .enable_feature("shell_tool")
        .disable_feature("web_search_request")
        .config_override("model_verbosity", "high")
        .config_override("features.search", "true");

    if let Some(path) = &cd {
        builder = builder.cd(path);
    }

    let client = builder.build();

    let mut request = ExecRequest::new(prompt).config_override("model_reasoning_effort", "low");
    if cd.is_none() {
        request.overrides.cd = Some(env::current_dir()?);
    }

    let response = client.send_prompt_with(request).await?;
    println!("{response}");
    Ok(())
}
