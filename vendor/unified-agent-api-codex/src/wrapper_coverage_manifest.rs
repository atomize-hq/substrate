use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CoverageLevel {
    Explicit,
    Passthrough,
    Unsupported,
    IntentionallyUnsupported,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperSurfaceScopedTargets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platforms: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_triples: Option<Vec<String>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperFlagCoverageV1 {
    pub key: String,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperArgCoverageV1 {
    pub name: String,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperCommandCoverageV1 {
    pub path: Vec<String>,
    pub level: CoverageLevel,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<WrapperSurfaceScopedTargets>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flags: Option<Vec<WrapperFlagCoverageV1>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<Vec<WrapperArgCoverageV1>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WrapperCoverageManifestV1 {
    pub schema_version: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wrapper_version: Option<String>,
    pub coverage: Vec<WrapperCommandCoverageV1>,
}

pub fn wrapper_crate_version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// The single source of truth for wrapper coverage declarations.
///
/// This value is consumed by `xtask codex-wrapper-coverage` to generate
/// `cli_manifests/codex/wrapper_coverage.json`.
pub fn wrapper_coverage_manifest() -> WrapperCoverageManifestV1 {
    fn flag(key: &str, level: CoverageLevel) -> WrapperFlagCoverageV1 {
        WrapperFlagCoverageV1 {
            key: key.to_string(),
            level,
            note: None,
            scope: None,
        }
    }

    fn flag_note(key: &str, level: CoverageLevel, note: &str) -> WrapperFlagCoverageV1 {
        WrapperFlagCoverageV1 {
            key: key.to_string(),
            level,
            note: Some(note.to_string()),
            scope: None,
        }
    }

    fn arg(name: &str, level: CoverageLevel) -> WrapperArgCoverageV1 {
        WrapperArgCoverageV1 {
            name: name.to_string(),
            level,
            note: None,
            scope: None,
        }
    }

    fn arg_note(name: &str, level: CoverageLevel, note: &str) -> WrapperArgCoverageV1 {
        WrapperArgCoverageV1 {
            name: name.to_string(),
            level,
            note: Some(note.to_string()),
            scope: None,
        }
    }

    fn command(
        path: &[&str],
        level: CoverageLevel,
        note: Option<&str>,
        flags: Vec<WrapperFlagCoverageV1>,
        args: Vec<WrapperArgCoverageV1>,
    ) -> WrapperCommandCoverageV1 {
        WrapperCommandCoverageV1 {
            path: path.iter().map(|s| s.to_string()).collect(),
            level,
            note: note.map(|s| s.to_string()),
            scope: None,
            flags: (!flags.is_empty()).then_some(flags),
            args: (!args.is_empty()).then_some(args),
        }
    }

    WrapperCoverageManifestV1 {
        schema_version: 1,
        generated_at: None,
        wrapper_version: None,
        coverage: vec![
            // Scenario 0: root/global flags and probe flags.
            command(
                &[],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--help", CoverageLevel::Explicit),
                    flag("--version", CoverageLevel::Explicit),
                    flag("--model", CoverageLevel::Explicit),
                    flag("--image", CoverageLevel::Explicit),
                    flag_note("--add-dir", CoverageLevel::Explicit, "capability-guarded"),
                    flag_note(
                        "--config",
                        CoverageLevel::Passthrough,
                        "Generic config overrides are forwarded as strings; keys are not typed/validated by the wrapper.",
                    ),
                    flag_note(
                        "--enable",
                        CoverageLevel::Passthrough,
                        "Generic feature toggles are forwarded as strings; individual feature flags are not typed by the wrapper.",
                    ),
                    flag_note(
                        "--disable",
                        CoverageLevel::Passthrough,
                        "Generic feature toggles are forwarded as strings; individual feature flags are not typed by the wrapper.",
                    ),
                    flag("--profile", CoverageLevel::Explicit),
                    flag("--cd", CoverageLevel::Explicit),
                    flag("--remote", CoverageLevel::Explicit),
                    flag("--remote-auth-token-env", CoverageLevel::Explicit),
                    flag("--ask-for-approval", CoverageLevel::Explicit),
                    flag("--sandbox", CoverageLevel::Explicit),
                    flag("--full-auto", CoverageLevel::Explicit),
                    flag(
                        "--dangerously-bypass-approvals-and-sandbox",
                        CoverageLevel::Explicit,
                    ),
                    flag("--local-provider", CoverageLevel::Explicit),
                    flag("--oss", CoverageLevel::Explicit),
                    flag("--search", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            // Scenario 1+2: `codex exec` (single-response + streaming).
            command(
                &["exec"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--color", CoverageLevel::Explicit),
                    flag("--ephemeral", CoverageLevel::Explicit),
                    flag("--ignore-rules", CoverageLevel::Explicit),
                    flag("--ignore-user-config", CoverageLevel::Explicit),
                    flag("--skip-git-repo-check", CoverageLevel::Explicit),
                    flag("--json", CoverageLevel::Explicit),
                    flag("--output-last-message", CoverageLevel::Explicit),
                    flag_note(
                        "--output-schema",
                        CoverageLevel::Explicit,
                        "capability-guarded",
                    ),
                ],
                vec![arg("PROMPT", CoverageLevel::Explicit)],
            ),
            // Scenario 3: `codex exec resume` (streaming resume).
            command(
                &["exec", "resume"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--ephemeral", CoverageLevel::Explicit),
                    flag("--ignore-rules", CoverageLevel::Explicit),
                    flag("--ignore-user-config", CoverageLevel::Explicit),
                    flag("--json", CoverageLevel::Explicit),
                    flag("--output-last-message", CoverageLevel::Explicit),
                    flag("--skip-git-repo-check", CoverageLevel::Explicit),
                    flag("--last", CoverageLevel::Explicit),
                    flag("--all", CoverageLevel::Explicit),
                ],
                vec![
                    arg("PROMPT", CoverageLevel::Explicit),
                    arg("SESSION_ID", CoverageLevel::Explicit),
                ],
            ),
            // Scenario 4: `codex apply <TASK_ID>`.
            command(
                &["apply"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("TASK_ID", CoverageLevel::Explicit)],
            ),
            // Scenario 4: `codex cloud diff <TASK_ID>`.
            command(
                &["cloud", "diff"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--attempt", CoverageLevel::Explicit)],
                vec![arg("TASK_ID", CoverageLevel::Explicit)],
            ),
            command(
                &["cloud", "apply"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--attempt", CoverageLevel::Explicit)],
                vec![arg("TASK_ID", CoverageLevel::Explicit)],
            ),
            command(
                &["cloud", "status"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("TASK_ID", CoverageLevel::Explicit)],
            ),
            command(
                &["cloud", "list"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--cursor", CoverageLevel::Explicit),
                    flag("--env", CoverageLevel::Explicit),
                    flag("--json", CoverageLevel::Explicit),
                    flag("--limit", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command(
                &["cloud", "exec"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--env", CoverageLevel::Explicit),
                    flag("--attempts", CoverageLevel::Explicit),
                    flag("--branch", CoverageLevel::Explicit),
                ],
                vec![arg("QUERY", CoverageLevel::Explicit)],
            ),
            // Scenario 5: login/logout.
            command(
                &["login"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag_note("--mcp", CoverageLevel::Explicit, "capability-guarded"),
                    flag("--api-key", CoverageLevel::Explicit),
                    flag("--device-auth", CoverageLevel::Explicit),
                    flag("--with-access-token", CoverageLevel::Explicit),
                    flag("--with-api-key", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command(
                &["login", "status"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command(&["logout"], CoverageLevel::Explicit, None, vec![], vec![]),
            // Scenario 6: `codex features list`.
            command(
                &["features", "list"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            // Scenario 7: `codex app-server generate-*`.
            command(
                &["app-server", "generate-ts"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--experimental", CoverageLevel::Explicit),
                    flag("--out", CoverageLevel::Explicit),
                    flag("--prettier", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command(
                &["app-server", "generate-json-schema"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--experimental", CoverageLevel::Explicit),
                    flag("--out", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            // Scenario 8: `codex responses-api-proxy`.
            command(
                &["responses-api-proxy"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--port", CoverageLevel::Explicit),
                    flag("--server-info", CoverageLevel::Explicit),
                    flag("--http-shutdown", CoverageLevel::Explicit),
                    flag("--upstream-url", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            // Scenario 9: `codex stdio-to-uds`.
            command(
                &["stdio-to-uds"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("SOCKET_PATH", CoverageLevel::Explicit)],
            ),
            // Scenario 10: `codex sandbox <platform>`.
            command(&["sandbox"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["sandbox", "macos"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--allow-unix-socket", CoverageLevel::Explicit),
                    flag("--include-managed-config", CoverageLevel::Explicit),
                    flag("--log-denials", CoverageLevel::Explicit),
                    flag("--permissions-profile", CoverageLevel::Explicit),
                ],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["sandbox", "linux"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--include-managed-config", CoverageLevel::Explicit),
                    flag("--permissions-profile", CoverageLevel::Explicit),
                ],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["sandbox", "windows"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--include-managed-config", CoverageLevel::Explicit),
                    flag("--permissions-profile", CoverageLevel::Explicit),
                ],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            // Scenario 11: `codex execpolicy check`.
            command(
                &["execpolicy", "check"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--policy", CoverageLevel::Explicit),
                    flag("--pretty", CoverageLevel::Explicit),
                ],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            // Scenario 12: stdio servers.
            command(
                &["mcp-server"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command(
                &["app-server"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--analytics-default-enabled", CoverageLevel::Explicit),
                    flag("--listen", CoverageLevel::Explicit),
                    flag("--ws-audience", CoverageLevel::Explicit),
                    flag("--ws-auth", CoverageLevel::Explicit),
                    flag("--ws-issuer", CoverageLevel::Explicit),
                    flag("--ws-max-clock-skew-seconds", CoverageLevel::Explicit),
                    flag("--ws-shared-secret-file", CoverageLevel::Explicit),
                    flag("--ws-token-file", CoverageLevel::Explicit),
                    flag("--ws-token-sha256", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command(
                &["app-server", "proxy"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--sock", CoverageLevel::Explicit)],
                vec![],
            ),
            command(&["cloud"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["exec-server"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--executor-id", CoverageLevel::Explicit),
                    flag("--listen", CoverageLevel::Explicit),
                    flag("--name", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command(&["update"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(&["mcp"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["mcp", "list"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--json", CoverageLevel::Explicit)],
                vec![],
            ),
            command(
                &["mcp", "get"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--json", CoverageLevel::Explicit)],
                vec![arg("NAME", CoverageLevel::Explicit)],
            ),
            command(
                &["mcp", "add"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--url", CoverageLevel::Explicit),
                    flag("--bearer-token-env-var", CoverageLevel::Explicit),
                    flag("--env", CoverageLevel::Explicit),
                ],
                vec![
                    arg("NAME", CoverageLevel::Explicit),
                    arg("COMMAND", CoverageLevel::Explicit),
                ],
            ),
            command(
                &["mcp", "remove"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("NAME", CoverageLevel::Explicit)],
            ),
            command(
                &["mcp", "logout"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("NAME", CoverageLevel::Explicit)],
            ),
            command(
                &["mcp", "login"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--scopes", CoverageLevel::Explicit)],
                vec![arg("NAME", CoverageLevel::Explicit)],
            ),
            // `codex help` command family (variadic COMMAND).
            command(
                &["help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["exec", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["features", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["login", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["app-server", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["sandbox", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["cloud", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["mcp", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            // New 0.92.0+ command surfaces.
            command(&["features"], CoverageLevel::Explicit, None, vec![], vec![]),
            // New 0.97.0 command surfaces.
            command(
                &["features", "enable"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("FEATURE", CoverageLevel::Explicit)],
            ),
            command(
                &["features", "disable"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("FEATURE", CoverageLevel::Explicit)],
            ),
            command(&["debug"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["debug", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["debug", "app-server"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command(
                &["debug", "app-server", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["debug", "app-server", "send-message-v2"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("USER_MESSAGE", CoverageLevel::Explicit)],
            ),
            command(
                &["debug", "models"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--bundled", CoverageLevel::Explicit)],
                vec![],
            ),
            command(
                &["debug", "prompt-input"],
                CoverageLevel::Explicit,
                None,
                vec![flag("--image", CoverageLevel::Explicit)],
                vec![arg("PROMPT", CoverageLevel::Explicit)],
            ),
            command(
                &["exec", "review"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--base", CoverageLevel::Explicit),
                    flag("--commit", CoverageLevel::Explicit),
                    flag("--ephemeral", CoverageLevel::Explicit),
                    flag("--ignore-rules", CoverageLevel::Explicit),
                    flag("--ignore-user-config", CoverageLevel::Explicit),
                    flag("--json", CoverageLevel::Explicit),
                    flag("--output-last-message", CoverageLevel::Explicit),
                    flag("--skip-git-repo-check", CoverageLevel::Explicit),
                    flag("--title", CoverageLevel::Explicit),
                    flag("--uncommitted", CoverageLevel::Explicit),
                ],
                vec![arg("PROMPT", CoverageLevel::Explicit)],
            ),
            command(
                &["review"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--base", CoverageLevel::Explicit),
                    flag("--commit", CoverageLevel::Explicit),
                    flag("--title", CoverageLevel::Explicit),
                    flag("--uncommitted", CoverageLevel::Explicit),
                ],
                vec![arg("PROMPT", CoverageLevel::Explicit)],
            ),
            command(
                &["resume"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--all", CoverageLevel::Explicit),
                    flag("--include-non-interactive", CoverageLevel::Explicit),
                    flag("--last", CoverageLevel::Explicit),
                ],
                vec![
                    arg("PROMPT", CoverageLevel::Explicit),
                    arg("SESSION_ID", CoverageLevel::Explicit),
                ],
            ),
            command(
                &["fork"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--all", CoverageLevel::Explicit),
                    flag("--last", CoverageLevel::Explicit),
                ],
                vec![
                    arg("PROMPT", CoverageLevel::Explicit),
                    arg("SESSION_ID", CoverageLevel::Explicit),
                ],
            ),
            command(&["plugin"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["plugin", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["plugin", "marketplace"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command(
                &["plugin", "marketplace", "add"],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--ref", CoverageLevel::Explicit),
                    flag("--sparse", CoverageLevel::Explicit),
                ],
                vec![arg("SOURCE", CoverageLevel::Explicit)],
            ),
            command(
                &["plugin", "marketplace", "help"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("COMMAND", CoverageLevel::Explicit)],
            ),
            command(
                &["plugin", "marketplace", "remove"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("MARKETPLACE_NAME", CoverageLevel::Explicit)],
            ),
            command(
                &["plugin", "marketplace", "upgrade"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![arg("MARKETPLACE_NAME", CoverageLevel::Explicit)],
            ),
            WrapperCommandCoverageV1 {
                path: vec!["completion".to_string()],
                level: CoverageLevel::IntentionallyUnsupported,
                note: Some(
                    "Shell completion generation is out of scope for the wrapper.".to_string(),
                ),
                scope: None,
                flags: None,
                args: Some(vec![arg_note(
                    "SHELL",
                    CoverageLevel::IntentionallyUnsupported,
                    "Shell completion generation is out of scope for the wrapper.",
                )]),
            },
        ],
    }
}
