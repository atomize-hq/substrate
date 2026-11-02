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
/// This value is consumed by `xtask claude-wrapper-coverage` to generate
/// `cli_manifests/claude_code/wrapper_coverage.json`.
pub fn wrapper_coverage_manifest() -> WrapperCoverageManifestV1 {
    fn flag(key: &str, level: CoverageLevel) -> WrapperFlagCoverageV1 {
        WrapperFlagCoverageV1 {
            key: key.to_string(),
            level,
            note: None,
            scope: None,
        }
    }

    fn scope_targets(target_triples: &[&str]) -> WrapperSurfaceScopedTargets {
        WrapperSurfaceScopedTargets {
            platforms: None,
            target_triples: Some(target_triples.iter().map(|t| t.to_string()).collect()),
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

    fn command_scoped(
        path: &[&str],
        level: CoverageLevel,
        note: Option<&str>,
        scope: WrapperSurfaceScopedTargets,
        flags: Vec<WrapperFlagCoverageV1>,
        args: Vec<WrapperArgCoverageV1>,
    ) -> WrapperCommandCoverageV1 {
        let mut out = command(path, level, note, flags, args);
        out.scope = Some(scope);
        out
    }

    WrapperCoverageManifestV1 {
        schema_version: 1,
        generated_at: None,
        wrapper_version: None,
        coverage: vec![
            // Root `claude` in non-interactive print mode.
            command(
                &[],
                CoverageLevel::Explicit,
                None,
                vec![
                    flag("--help", CoverageLevel::Explicit),
                    flag("--version", CoverageLevel::Explicit),
                    flag("--print", CoverageLevel::Explicit),
                    flag("--output-format", CoverageLevel::Explicit),
                    flag("--input-format", CoverageLevel::Explicit),
                    flag("--json-schema", CoverageLevel::Explicit),
                    flag("--model", CoverageLevel::Explicit),
                    flag("--allowedTools", CoverageLevel::Explicit),
                    flag("--disallowedTools", CoverageLevel::Explicit),
                    flag("--permission-mode", CoverageLevel::Explicit),
                    flag("--dangerously-skip-permissions", CoverageLevel::Explicit),
                    flag("--add-dir", CoverageLevel::Explicit),
                    flag("--mcp-config", CoverageLevel::Explicit),
                    flag("--strict-mcp-config", CoverageLevel::Explicit),
                    flag("--agent", CoverageLevel::Explicit),
                    flag("--agents", CoverageLevel::Explicit),
                    flag(
                        "--allow-dangerously-skip-permissions",
                        CoverageLevel::Explicit,
                    ),
                    flag("--append-system-prompt", CoverageLevel::Explicit),
                    flag("--betas", CoverageLevel::Explicit),
                    flag("--chrome", CoverageLevel::Explicit),
                    flag("--continue", CoverageLevel::Explicit),
                    flag("--debug", CoverageLevel::Explicit),
                    flag("--debug-file", CoverageLevel::Explicit),
                    flag("--disable-slash-commands", CoverageLevel::Explicit),
                    flag("--fallback-model", CoverageLevel::Explicit),
                    flag("--file", CoverageLevel::Explicit),
                    flag("--fork-session", CoverageLevel::Explicit),
                    flag("--from-pr", CoverageLevel::Explicit),
                    flag("--ide", CoverageLevel::Explicit),
                    flag("--include-partial-messages", CoverageLevel::Explicit),
                    flag("--max-budget-usd", CoverageLevel::Explicit),
                    flag("--mcp-debug", CoverageLevel::Explicit),
                    flag("--no-chrome", CoverageLevel::Explicit),
                    flag("--no-session-persistence", CoverageLevel::Explicit),
                    flag("--plugin-dir", CoverageLevel::Explicit),
                    flag("--replay-user-messages", CoverageLevel::Explicit),
                    flag("--resume", CoverageLevel::Explicit),
                    flag("--session-id", CoverageLevel::Explicit),
                    flag("--setting-sources", CoverageLevel::Explicit),
                    flag("--settings", CoverageLevel::Explicit),
                    flag("--system-prompt", CoverageLevel::Explicit),
                    flag("--tools", CoverageLevel::Explicit),
                    flag("--verbose", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            // IU subtree roots (policy): updater/diagnostics.
            command_scoped(
                &["install"],
                CoverageLevel::IntentionallyUnsupported,
                Some("Claude Code installation is out of scope for this wrapper."),
                scope_targets(&["win32-x64"]),
                vec![WrapperFlagCoverageV1 {
                    key: "--force".to_string(),
                    level: CoverageLevel::IntentionallyUnsupported,
                    note: Some(
                        "Claude Code installation is out of scope for this wrapper.".to_string(),
                    ),
                    scope: None,
                }],
                vec![],
            ),
            command(&["update"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(&["doctor"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["setup-token"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            // MCP management is supported via explicit typed requests.
            command(&["mcp"], CoverageLevel::Explicit, None, vec![], vec![]),
            command(
                &["mcp", "list"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command_scoped(
                &["mcp", "get"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![WrapperArgCoverageV1 {
                    name: "name".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["mcp", "add"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![
                    flag("--scope", CoverageLevel::Explicit),
                    flag("--transport", CoverageLevel::Explicit),
                    flag("--env", CoverageLevel::Explicit),
                    flag("--header", CoverageLevel::Explicit),
                ],
                vec![
                    WrapperArgCoverageV1 {
                        name: "name".to_string(),
                        level: CoverageLevel::Explicit,
                        note: None,
                        scope: None,
                    },
                    WrapperArgCoverageV1 {
                        name: "commandOrUrl".to_string(),
                        level: CoverageLevel::Explicit,
                        note: None,
                        scope: None,
                    },
                ],
            ),
            command_scoped(
                &["mcp", "remove"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![WrapperArgCoverageV1 {
                    name: "name".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["mcp", "add-json"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![
                    WrapperArgCoverageV1 {
                        name: "name".to_string(),
                        level: CoverageLevel::Explicit,
                        note: None,
                        scope: None,
                    },
                    WrapperArgCoverageV1 {
                        name: "json".to_string(),
                        level: CoverageLevel::Explicit,
                        note: None,
                        scope: None,
                    },
                ],
            ),
            command(
                &["mcp", "reset-project-choices"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command_scoped(
                &["mcp", "serve"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![],
            ),
            command_scoped(
                &["mcp", "add-from-claude-desktop"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![],
            ),
            // Plugin management is supported via typed requests (may have side effects).
            command(&["plugin"], CoverageLevel::Explicit, None, vec![], vec![]),
            command_scoped(
                &["plugin", "list"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![
                    flag("--available", CoverageLevel::Explicit),
                    flag("--json", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command_scoped(
                &["plugin", "enable"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![WrapperArgCoverageV1 {
                    name: "plugin".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["plugin", "disable"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![
                    flag("--all", CoverageLevel::Explicit),
                    flag("--scope", CoverageLevel::Explicit),
                ],
                vec![],
            ),
            command_scoped(
                &["plugin", "install"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![],
            ),
            command_scoped(
                &["plugin", "uninstall"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![],
            ),
            command_scoped(
                &["plugin", "update"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--scope", CoverageLevel::Explicit)],
                vec![WrapperArgCoverageV1 {
                    name: "plugin".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["plugin", "validate"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![WrapperArgCoverageV1 {
                    name: "path".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["plugin", "manifest"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["linux-x64", "darwin-arm64"]),
                vec![],
                vec![],
            ),
            command_scoped(
                &["plugin", "manifest", "marketplace"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["linux-x64", "darwin-arm64"]),
                vec![],
                vec![],
            ),
            command(
                &["plugin", "marketplace"],
                CoverageLevel::Explicit,
                None,
                vec![],
                vec![],
            ),
            command_scoped(
                &["plugin", "marketplace", "add"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![WrapperArgCoverageV1 {
                    name: "source".to_string(),
                    level: CoverageLevel::Explicit,
                    note: None,
                    scope: None,
                }],
            ),
            command_scoped(
                &["plugin", "marketplace", "list"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![flag("--json", CoverageLevel::Explicit)],
                vec![],
            ),
            command_scoped(
                &["plugin", "marketplace", "remove"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![],
            ),
            command_scoped(
                &["plugin", "marketplace", "repo"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["linux-x64", "darwin-arm64"]),
                vec![],
                vec![],
            ),
            command_scoped(
                &["plugin", "marketplace", "update"],
                CoverageLevel::Explicit,
                None,
                scope_targets(&["win32-x64"]),
                vec![],
                vec![],
            ),
        ],
    }
}
