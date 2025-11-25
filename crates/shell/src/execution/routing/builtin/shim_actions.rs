//! Shim-facing helpers for builtin command handling.

use super::super::log_command_event;
use crate::execution::ShellConfig;
use anyhow::Result;
use serde_json::json;
use uuid::Uuid;

pub(super) fn log_builtin_command(
    config: &ShellConfig,
    command: &str,
    parent_cmd_id: &str,
) -> Result<()> {
    let builtin_cmd_id = Uuid::now_v7().to_string();
    let extra = json!({ "parent_cmd_id": parent_cmd_id });

    let redacted_command = redact_sensitive_exports(command);

    log_command_event(
        config,
        "builtin_command",
        &redacted_command,
        &builtin_cmd_id,
        Some(extra),
    )
}

fn redact_sensitive_exports(command: &str) -> String {
    let tokens = shell_words::split(command).unwrap_or_else(|_| vec![command.to_string()]);
    let mut out = Vec::new();
    let mut i = 0;

    while i < tokens.len() {
        let t = &tokens[i];

        // Check for environment variable exports with sensitive names
        if tokens.len() > 1 && tokens[0] == "export" && t.contains('=') {
            if let Some((k, _)) = t.split_once('=') {
                let kl = k.to_lowercase();
                if kl.contains("token")
                    || kl.contains("password")
                    || kl.contains("secret")
                    || kl.contains("apikey")
                    || kl.contains("api_key")
                {
                    out.push(format!("{k}=***"));
                    i += 1;
                    continue;
                }
            }
        }

        out.push(t.clone());
        i += 1;
    }

    out.join(" ")
}
