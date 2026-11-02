use std::ffi::OsString;

use crate::{mcp::AgentWrapperMcpAddTransport, AgentWrapperError};

use super::{invalid_request, PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED};

pub(super) fn claude_mcp_list_argv() -> Vec<OsString> {
    vec![OsString::from("mcp"), OsString::from("list")]
}

pub(super) fn claude_mcp_get_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("get"),
        OsString::from(name),
    ]
}

pub(super) fn claude_mcp_remove_argv(name: &str) -> Vec<OsString> {
    vec![
        OsString::from("mcp"),
        OsString::from("remove"),
        OsString::from(name),
    ]
}

pub(super) fn claude_mcp_add_argv(
    name: &str,
    transport: &AgentWrapperMcpAddTransport,
) -> Result<Vec<OsString>, AgentWrapperError> {
    let mut argv = vec![OsString::from("mcp"), OsString::from("add")];

    match transport {
        AgentWrapperMcpAddTransport::Stdio { command, args, env } => {
            argv.push(OsString::from("--transport"));
            argv.push(OsString::from("stdio"));
            for (key, value) in env {
                argv.push(OsString::from("--env"));
                argv.push(OsString::from(format!("{key}={value}")));
            }
            argv.push(OsString::from(name));
            argv.extend(command.iter().map(OsString::from));
            argv.extend(args.iter().map(OsString::from));
            Ok(argv)
        }
        AgentWrapperMcpAddTransport::Url {
            url,
            bearer_token_env_var,
        } => {
            if bearer_token_env_var.is_some() {
                return Err(invalid_request(PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED));
            }

            argv.push(OsString::from("--transport"));
            argv.push(OsString::from("http"));
            argv.push(OsString::from(name));
            argv.push(OsString::from(url));
            Ok(argv)
        }
    }
}
