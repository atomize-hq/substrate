use std::{env, io};

#[path = "fake_claude_stream_json_agent_api/fixtures.rs"]
mod fixtures;
#[path = "fake_claude_stream_json_agent_api/scenarios.rs"]
mod scenarios;
#[path = "fake_claude_stream_json_agent_api/support.rs"]
mod support;

fn main() -> io::Result<()> {
    // Cross-platform test binary used by `agent_api` tests.
    //
    // Emulates: `claude --print --output-format stream-json ...`
    // Scenario is selected via env var so tests can validate incrementality + gating.
    //
    // The universal agent wrapper contract defaults to non-interactive behavior; require that the
    // wrapper passes `--permission-mode bypassPermissions` so tests fail loudly if we regress.
    let args: Vec<String> = env::args().collect();
    scenarios::run(args)
}
