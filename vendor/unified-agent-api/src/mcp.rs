use std::collections::BTreeMap;
use std::path::PathBuf;
use std::process::ExitStatus;
use std::time::Duration;

use crate::AgentWrapperError;

pub(crate) const CAPABILITY_MCP_LIST_V1: &str = "agent_api.tools.mcp.list.v1";
pub(crate) const CAPABILITY_MCP_GET_V1: &str = "agent_api.tools.mcp.get.v1";
pub(crate) const CAPABILITY_MCP_ADD_V1: &str = "agent_api.tools.mcp.add.v1";
pub(crate) const CAPABILITY_MCP_REMOVE_V1: &str = "agent_api.tools.mcp.remove.v1";

const ERR_MCP_SERVER_NAME_EMPTY: &str = "mcp server name must be non-empty";
const ERR_MCP_ADD_STDIO_COMMAND_EMPTY: &str =
    "mcp add stdio.command must contain at least one item";
const ERR_MCP_ADD_URL_EMPTY: &str = "mcp add url must be non-empty";
const ERR_MCP_ADD_URL_INVALID: &str = "mcp add url must be an absolute http or https URL";
const ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_EMPTY: &str =
    "mcp add bearer_token_env_var must be non-empty";
const ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_INVALID: &str =
    "mcp add bearer_token_env_var must match ^[A-Za-z_][A-Za-z0-9_]*$";

#[derive(Clone, Debug, Default)]
pub struct AgentWrapperMcpCommandContext {
    pub working_dir: Option<PathBuf>,
    pub timeout: Option<Duration>,
    pub env: BTreeMap<String, String>,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpCommandOutput {
    pub status: ExitStatus,
    /// Backends should populate this via `crate::bounds::enforce_mcp_output_bound` so stdout and
    /// stderr stay aligned with the pinned MM-C04 truncation algorithm.
    pub stdout: String,
    pub stderr: String,
    pub stdout_truncated: bool,
    pub stderr_truncated: bool,
}

#[derive(Clone, Debug, Default)]
pub struct AgentWrapperMcpListRequest {
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpGetRequest {
    pub name: String,
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpRemoveRequest {
    pub name: String,
    pub context: AgentWrapperMcpCommandContext,
}

#[derive(Clone, Debug)]
pub enum AgentWrapperMcpAddTransport {
    /// Launches an MCP server via stdio.
    Stdio {
        /// Command argv (MUST be non-empty).
        command: Vec<String>,
        /// Additional argv items appended after `command`.
        args: Vec<String>,
        /// Env vars injected into the MCP server process.
        env: BTreeMap<String, String>,
    },
    /// Connects to a streamable HTTP MCP server.
    Url {
        url: String,
        bearer_token_env_var: Option<String>,
    },
}

#[derive(Clone, Debug)]
pub struct AgentWrapperMcpAddRequest {
    pub name: String,
    pub transport: AgentWrapperMcpAddTransport,
    pub context: AgentWrapperMcpCommandContext,
}

pub(crate) fn normalize_server_name(name: &str) -> Result<String, AgentWrapperError> {
    let name = name.trim();
    if name.is_empty() {
        return Err(invalid_request(ERR_MCP_SERVER_NAME_EMPTY));
    }

    Ok(name.to_string())
}

pub(crate) fn normalize_add_transport(
    transport: AgentWrapperMcpAddTransport,
) -> Result<AgentWrapperMcpAddTransport, AgentWrapperError> {
    match transport {
        AgentWrapperMcpAddTransport::Stdio { command, args, env } => {
            if command.is_empty() {
                return Err(invalid_request(ERR_MCP_ADD_STDIO_COMMAND_EMPTY));
            }

            Ok(AgentWrapperMcpAddTransport::Stdio {
                command: normalize_stdio_items(command, "mcp add stdio.command")?,
                args: normalize_stdio_items(args, "mcp add stdio.args")?,
                env,
            })
        }
        AgentWrapperMcpAddTransport::Url {
            url,
            bearer_token_env_var,
        } => {
            let url = normalize_url(url)?;
            let bearer_token_env_var = normalize_bearer_token_env_var(bearer_token_env_var)?;
            Ok(AgentWrapperMcpAddTransport::Url {
                url,
                bearer_token_env_var,
            })
        }
    }
}

pub(crate) fn normalize_mcp_get_request(
    request: AgentWrapperMcpGetRequest,
) -> Result<AgentWrapperMcpGetRequest, AgentWrapperError> {
    Ok(AgentWrapperMcpGetRequest {
        name: normalize_server_name(&request.name)?,
        context: request.context,
    })
}

pub(crate) fn normalize_mcp_add_request(
    request: AgentWrapperMcpAddRequest,
) -> Result<AgentWrapperMcpAddRequest, AgentWrapperError> {
    Ok(AgentWrapperMcpAddRequest {
        name: normalize_server_name(&request.name)?,
        transport: normalize_add_transport(request.transport)?,
        context: request.context,
    })
}

pub(crate) fn normalize_mcp_remove_request(
    request: AgentWrapperMcpRemoveRequest,
) -> Result<AgentWrapperMcpRemoveRequest, AgentWrapperError> {
    Ok(AgentWrapperMcpRemoveRequest {
        name: normalize_server_name(&request.name)?,
        context: request.context,
    })
}

fn normalize_stdio_items(
    items: Vec<String>,
    field: &str,
) -> Result<Vec<String>, AgentWrapperError> {
    items
        .into_iter()
        .enumerate()
        .map(|(idx, item)| {
            let trimmed = item.trim();
            if trimmed.is_empty() {
                return Err(invalid_request(format!("{field}[{idx}] must be non-empty")));
            }

            Ok(trimmed.to_string())
        })
        .collect()
}

fn normalize_url(url: String) -> Result<String, AgentWrapperError> {
    let url = url.trim();
    if url.is_empty() {
        return Err(invalid_request(ERR_MCP_ADD_URL_EMPTY));
    }

    let parsed = url::Url::parse(url).map_err(|_| invalid_request(ERR_MCP_ADD_URL_INVALID))?;
    match parsed.scheme() {
        "http" | "https" if has_http_authority_separator(url) => Ok(url.to_string()),
        _ => Err(invalid_request(ERR_MCP_ADD_URL_INVALID)),
    }
}

fn has_http_authority_separator(url: &str) -> bool {
    url.get(..7)
        .is_some_and(|prefix| prefix.eq_ignore_ascii_case("http://"))
        || url
            .get(..8)
            .is_some_and(|prefix| prefix.eq_ignore_ascii_case("https://"))
}

fn normalize_bearer_token_env_var(
    value: Option<String>,
) -> Result<Option<String>, AgentWrapperError> {
    match value {
        Some(value) => {
            let trimmed = value.trim();
            if trimmed.is_empty() {
                return Err(invalid_request(ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_EMPTY));
            }
            if !is_valid_env_var_name(trimmed) {
                return Err(invalid_request(ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_INVALID));
            }

            Ok(Some(trimmed.to_string()))
        }
        None => Ok(None),
    }
}

fn is_valid_env_var_name(value: &str) -> bool {
    let mut chars = value.chars();
    let Some(first) = chars.next() else {
        return false;
    };
    if !(first.is_ascii_alphabetic() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_alphanumeric() || ch == '_')
}

fn invalid_request(message: impl Into<String>) -> AgentWrapperError {
    AgentWrapperError::InvalidRequest {
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_context() -> AgentWrapperMcpCommandContext {
        let mut env = BTreeMap::new();
        env.insert("UNCHANGED".to_string(), "  value with spaces  ".to_string());
        AgentWrapperMcpCommandContext {
            working_dir: Some(PathBuf::from("relative/workdir")),
            timeout: Some(Duration::from_secs(30)),
            env,
        }
    }

    fn assert_invalid_request(
        result: Result<(), AgentWrapperError>,
        expected_message: &str,
        redacted_values: &[&str],
    ) {
        match result {
            Err(AgentWrapperError::InvalidRequest { message }) => {
                assert_eq!(message, expected_message);
                for value in redacted_values {
                    assert!(
                        !message.contains(value),
                        "message leaked raw input `{value}`: {message}"
                    );
                }
            }
            Err(other) => panic!("expected InvalidRequest, got {other:?}"),
            Ok(()) => panic!("expected InvalidRequest"),
        }
    }

    #[test]
    fn normalize_server_name_trims_and_rejects_empty_values() {
        assert_eq!(
            normalize_server_name("  demo-server  ").expect("name should normalize"),
            "demo-server"
        );

        assert_invalid_request(
            normalize_server_name("   \n\t  ").map(|_| ()),
            ERR_MCP_SERVER_NAME_EMPTY,
            &[],
        );
    }

    #[test]
    fn normalize_mcp_add_request_trims_fields_and_preserves_context_and_env_maps() {
        let context = sample_context();
        let mut transport_env = BTreeMap::new();
        transport_env.insert("KEEP".to_string(), "  exact value  ".to_string());

        let request = AgentWrapperMcpAddRequest {
            name: "  example  ".to_string(),
            transport: AgentWrapperMcpAddTransport::Stdio {
                command: vec!["  bin/example  ".to_string()],
                args: vec!["  --flag  ".to_string(), "  value  ".to_string()],
                env: transport_env.clone(),
            },
            context: context.clone(),
        };

        let normalized = normalize_mcp_add_request(request).expect("request should normalize");
        assert_eq!(normalized.name, "example");
        assert_eq!(normalized.context.working_dir, context.working_dir);
        assert_eq!(normalized.context.timeout, context.timeout);
        assert_eq!(normalized.context.env, context.env);
        match normalized.transport {
            AgentWrapperMcpAddTransport::Stdio { command, args, env } => {
                assert_eq!(command, vec!["bin/example".to_string()]);
                assert_eq!(args, vec!["--flag".to_string(), "value".to_string()]);
                assert_eq!(env, transport_env);
            }
            AgentWrapperMcpAddTransport::Url { .. } => panic!("expected stdio transport"),
        }
    }

    #[test]
    fn normalize_add_transport_accepts_and_trims_valid_url_transport() {
        let transport = AgentWrapperMcpAddTransport::Url {
            url: "  https://example.com/mcp  ".to_string(),
            bearer_token_env_var: Some("  TOKEN_NAME  ".to_string()),
        };

        let normalized =
            normalize_add_transport(transport).expect("url transport should normalize");
        match normalized {
            AgentWrapperMcpAddTransport::Url {
                url,
                bearer_token_env_var,
            } => {
                assert_eq!(url, "https://example.com/mcp");
                assert_eq!(bearer_token_env_var.as_deref(), Some("TOKEN_NAME"));
            }
            AgentWrapperMcpAddTransport::Stdio { .. } => panic!("expected url transport"),
        }
    }

    #[test]
    fn normalize_add_transport_rejects_invalid_stdio_fields_without_leaking_raw_values() {
        let secret = "SECRET_STDIO_VALUE";

        assert_invalid_request(
            normalize_add_transport(AgentWrapperMcpAddTransport::Stdio {
                command: Vec::new(),
                args: Vec::new(),
                env: BTreeMap::new(),
            })
            .map(|_| ()),
            ERR_MCP_ADD_STDIO_COMMAND_EMPTY,
            &[],
        );

        assert_invalid_request(
            normalize_add_transport(AgentWrapperMcpAddTransport::Stdio {
                command: vec![format!("  {secret}  "), "   ".to_string()],
                args: Vec::new(),
                env: BTreeMap::new(),
            })
            .map(|_| ()),
            "mcp add stdio.command[1] must be non-empty",
            &[secret],
        );

        assert_invalid_request(
            normalize_add_transport(AgentWrapperMcpAddTransport::Stdio {
                command: vec!["cmd".to_string()],
                args: vec![format!("  {secret}  "), "   ".to_string()],
                env: BTreeMap::new(),
            })
            .map(|_| ()),
            "mcp add stdio.args[1] must be non-empty",
            &[secret],
        );
    }

    #[test]
    fn normalize_add_transport_rejects_invalid_url_fields_without_leaking_raw_values() {
        let secret = "SECRET_URL_VALUE";

        assert_invalid_request(
            normalize_add_transport(AgentWrapperMcpAddTransport::Url {
                url: "   ".to_string(),
                bearer_token_env_var: None,
            })
            .map(|_| ()),
            ERR_MCP_ADD_URL_EMPTY,
            &[],
        );

        for raw in [
            format!(" {secret} "),
            format!("relative/{secret}"),
            format!("ftp://{secret}.example.com"),
            format!("http:// space/{secret}"),
            format!("https:{secret}.example.com"),
            format!("http:{secret}"),
            format!("https:/{secret}.example.com"),
        ] {
            assert_invalid_request(
                normalize_add_transport(AgentWrapperMcpAddTransport::Url {
                    url: raw,
                    bearer_token_env_var: None,
                })
                .map(|_| ()),
                ERR_MCP_ADD_URL_INVALID,
                &[secret],
            );
        }

        assert_invalid_request(
            normalize_add_transport(AgentWrapperMcpAddTransport::Url {
                url: "https://example.com/mcp".to_string(),
                bearer_token_env_var: Some("   ".to_string()),
            })
            .map(|_| ()),
            ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_EMPTY,
            &[],
        );

        for raw in [
            format!("9{secret}"),
            format!("BAD-{secret}"),
            format!("bad space {secret}"),
        ] {
            assert_invalid_request(
                normalize_add_transport(AgentWrapperMcpAddTransport::Url {
                    url: "https://example.com/mcp".to_string(),
                    bearer_token_env_var: Some(raw),
                })
                .map(|_| ()),
                ERR_MCP_ADD_BEARER_TOKEN_ENV_VAR_INVALID,
                &[secret],
            );
        }
    }

    #[test]
    fn normalize_get_and_remove_requests_trim_name_and_preserve_context() {
        let context = sample_context();

        let get = normalize_mcp_get_request(AgentWrapperMcpGetRequest {
            name: "  get-name  ".to_string(),
            context: context.clone(),
        })
        .expect("get request should normalize");
        assert_eq!(get.name, "get-name");
        assert_eq!(get.context.env, context.env);

        let remove = normalize_mcp_remove_request(AgentWrapperMcpRemoveRequest {
            name: "  remove-name  ".to_string(),
            context: context.clone(),
        })
        .expect("remove request should normalize");
        assert_eq!(remove.name, "remove-name");
        assert_eq!(remove.context.working_dir, context.working_dir);
        assert_eq!(remove.context.timeout, context.timeout);
    }
}
