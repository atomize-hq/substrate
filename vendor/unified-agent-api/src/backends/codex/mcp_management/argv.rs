use std::ffi::OsString;

use crate::mcp::AgentWrapperMcpAddTransport;

pub(super) fn codex_mcp_list_argv() -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("list"),
        OsString::from("--json"),
    ]
}

pub(super) fn codex_mcp_get_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("get"),
        OsString::from("--json"),
        OsString::from(name),
    ]
}

pub(super) fn codex_mcp_remove_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("remove"),
        OsString::from(name),
    ]
}

pub(super) fn codex_mcp_add_argv(
    name: &str,
    transport: &AgentWrapperMcpAddTransport,
) -> Vec<OsString> {
    let mut argv = vec![
        OsString::from("mcp"),
        OsString::from("add"),
        OsString::from(name),
    ];

    match transport {
        AgentWrapperMcpAddTransport::Stdio { command, args, env } => {
            for (key, value) in env {
                argv.push(OsString::from("--env"));
                argv.push(OsString::from(format!("{key}={value}")));
            }
            argv.push(OsString::from("--"));
            argv.extend(command.iter().cloned().map(OsString::from));
            argv.extend(args.iter().cloned().map(OsString::from));
        }
        AgentWrapperMcpAddTransport::Url {
            url,
            bearer_token_env_var,
        } => {
            argv.push(OsString::from("--url"));
            argv.push(OsString::from(url));
            if let Some(env_var) = bearer_token_env_var {
                argv.push(OsString::from("--bearer-token-env-var"));
                argv.push(OsString::from(env_var));
            }
        }
    }

    argv
}
