use crate::execution::agent_inventory::{discover_agent_files, validate_agent_file};
use crate::execution::cli::{AgentsAction, AgentsCmd, Cli};
use crate::execution::config_model;
use anyhow::Result;
use std::env;
use std::path::PathBuf;

pub(crate) fn handle_agents_command(cmd: &AgentsCmd, _cli: &Cli) -> i32 {
    let result = match &cmd.action {
        AgentsAction::Validate => run_validate(),
    };

    match result {
        Ok(()) => 0,
        Err(err) if config_model::is_user_error(&err) => {
            eprintln!("{err}");
            2
        }
        Err(err) => {
            eprintln!("{:#}", err);
            1
        }
    }
}

fn run_validate() -> Result<()> {
    let cwd = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let (base_policy, _) = substrate_broker::resolve_effective_policy_with_explain(&cwd, false)
        .map_err(|err| config_model::user_error(err.to_string()))?;
    let agent_files = discover_agent_files(&cwd)?;

    for path in agent_files {
        validate_agent_file(&path, &base_policy)?;
    }

    Ok(())
}
