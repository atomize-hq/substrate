use std::{collections::BTreeMap, ffi::OsString};

use crate::{mcp::AgentWrapperMcpAddTransport, AgentWrapperError};

use super::super::{
    claude_mcp_add_argv, claude_mcp_get_argv, claude_mcp_list_argv, claude_mcp_remove_argv,
    PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED,
};

#[test]
fn claude_mcp_list_argv_is_pinned() {
    assert_eq!(
        claude_mcp_list_argv(),
        vec![OsString::from("mcp"), OsString::from("list")]
    );
}

#[test]
fn claude_mcp_get_argv_is_pinned() {
    assert_eq!(
        claude_mcp_get_argv("demo"),
        vec![
            OsString::from("mcp"),
            OsString::from("get"),
            OsString::from("demo"),
        ]
    );
}

#[test]
fn claude_mcp_remove_argv_is_pinned() {
    assert_eq!(
        claude_mcp_remove_argv("demo"),
        vec![
            OsString::from("mcp"),
            OsString::from("remove"),
            OsString::from("demo"),
        ]
    );
}

#[test]
fn claude_mcp_remove_argv_preserves_name_as_single_item() {
    assert_eq!(
        claude_mcp_remove_argv("demo server"),
        vec![
            OsString::from("mcp"),
            OsString::from("remove"),
            OsString::from("demo server"),
        ]
    );
}

#[test]
fn claude_mcp_add_argv_maps_stdio_transport_with_sorted_env_and_no_separator() {
    let transport = AgentWrapperMcpAddTransport::Stdio {
        command: vec!["node".to_string()],
        args: vec!["server.js".to_string(), "--flag".to_string()],
        env: BTreeMap::from([
            ("BETA".to_string(), "two".to_string()),
            ("ALPHA".to_string(), "one".to_string()),
        ]),
    };

    assert_eq!(
        claude_mcp_add_argv("demo", &transport).expect("stdio transport should map"),
        vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from("--transport"),
            OsString::from("stdio"),
            OsString::from("--env"),
            OsString::from("ALPHA=one"),
            OsString::from("--env"),
            OsString::from("BETA=two"),
            OsString::from("demo"),
            OsString::from("node"),
            OsString::from("server.js"),
            OsString::from("--flag"),
        ]
    );
}

#[test]
fn claude_mcp_add_argv_maps_url_transport_without_bearer_env() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: None,
    };

    assert_eq!(
        claude_mcp_add_argv("demo", &transport).expect("url transport should map"),
        vec![
            OsString::from("mcp"),
            OsString::from("add"),
            OsString::from("--transport"),
            OsString::from("http"),
            OsString::from("demo"),
            OsString::from("https://example.test/mcp"),
        ]
    );
}

#[test]
fn claude_mcp_add_argv_rejects_url_transport_with_bearer_env_var() {
    let transport = AgentWrapperMcpAddTransport::Url {
        url: "https://example.test/mcp".to_string(),
        bearer_token_env_var: Some("TOKEN_ENV".to_string()),
    };

    let err = claude_mcp_add_argv("demo", &transport)
        .expect_err("url bearer token env var should be rejected");

    match err {
        AgentWrapperError::InvalidRequest { message } => {
            assert_eq!(message, PINNED_URL_BEARER_TOKEN_ENV_VAR_UNSUPPORTED);
        }
        other => panic!("expected InvalidRequest, got: {other:?}"),
    }
}
