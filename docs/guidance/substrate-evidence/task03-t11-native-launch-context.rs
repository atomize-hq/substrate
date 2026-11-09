//! Immutable initial native wiring, committed through the existing Start origin.
//!
//! Native home/config/session files are mutable state. Only an authenticated
//! origin digest and this separate artifact authorize selected launch inputs.
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use transport_api_types::InitialCodexModel;

use crate::execution::agent_runtime::host_session_authority::{
    canonical_json,
    schema::{CanonicalDirectoryV1, DurableSessionAuthorityOriginV1},
    trusted_fs::TrustedAuthorityRoot,
    HostSessionAuthority,
};

const CONTEXT_FILE: &str = "native-launch-context.json";
const MARKER: &str = "req_native_";
const WORKLOAD_KEYS: &[&str] = &[
    "OPENAI_FEDERATION_RULE_ID",
    "OPENAI_IDENTITY_TOKEN_FILE",
    "OPENAI_WORKLOAD_IDENTITY_CONTEXT",
];
const BLANK_KEYS: &[&str] = &[
    "OPENAI_API_KEY",
    "OPENAI_ACCESS_TOKEN",
    "OPENAI_BASE_URL",
    "OPENAI_ORGANIZATION",
    "OPENAI_PROJECT",
    "CODEX_API_KEY",
    "ANTHROPIC_API_KEY",
    "ANTHROPIC_AUTH_TOKEN",
    "ANTHROPIC_BASE_URL",
    "SUBSTRATE_LLM_AUTH_BUNDLE_FD",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID",
    "SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCESS_TOKEN",
    "SUBSTRATE_LLM_BACKEND_AUTH_API_OPENAI_API_KEY",
    "SUBSTRATE_LLM_BACKEND_AUTH_API_ANTHROPIC_API_KEY",
    "SUBSTRATE_INTERNAL_CODEX_AUTH_SEED_HOME",
    "CODEX_ACCESS_TOKEN",
    "CODEX_REFRESH_TOKEN_URL_OVERRIDE",
    "CODEX_REVOKE_TOKEN_URL_OVERRIDE",
    "CODEX_APP_SERVER_LOGIN_CLIENT_ID",
    "CODEX_OSS_BASE_URL",
    "HTTP_PROXY",
    "HTTPS_PROXY",
    "ALL_PROXY",
    "http_proxy",
    "https_proxy",
    "all_proxy",
];

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct NativeLaunchContext {
    schema_version: u32,
    authority_store_id: String,
    accepted_home: CanonicalDirectoryV1,
    original_session_id: String,
    backend_id: String,
    model: InitialCodexModel,
    codex_home: PathBuf,
    base_url: String,
}

fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn marker_digest(issuer: &str) -> Result<Option<&str>> {
    let Some(rest) = issuer.strip_prefix(MARKER) else {
        if issuer.starts_with("req_native") {
            bail!("malformed selected native origin");
        }
        return Ok(None);
    };
    let Some((context, plan)) = rest.split_once('_') else {
        bail!("malformed selected native origin");
    };
    let valid = |value: &str| {
        value.len() == 64
            && value
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    };
    if !valid(context) || !valid(plan) {
        bail!("malformed selected native origin");
    }
    Ok(Some(context))
}

fn validate_source_posture(workload_present: bool, administrative_present: bool) -> Result<()> {
    if workload_present {
        bail!("selected native launch does not support workload identity");
    }
    if administrative_present {
        bail!("selected native launch does not support administrative native configuration");
    }
    Ok(())
}

/// This deliberately refuses unsupported administrative/auth sources rather
/// than implementing a second native loader or mutating the parent environment.
pub(crate) fn validate_native_posture() -> Result<()> {
    if !cfg!(target_os = "linux") {
        bail!("selected Codex model is supported on Linux only");
    }
    validate_source_posture(
        WORKLOAD_KEYS
            .iter()
            .any(|key| std::env::var_os(key).is_some()),
        false,
    )?;
    for name in ["requirements.toml", "managed_config.toml", "config.toml"] {
        match std::fs::symlink_metadata(Path::new("/etc/codex").join(name)) {
            Ok(_) => validate_source_posture(false, true)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(_) => bail!("cannot establish native administrative configuration posture"),
        }
    }
    #[cfg(target_os = "linux")]
    if let Some(value) = std::env::var_os("SUBSTRATE_LLM_AUTH_BUNDLE_FD").filter(|v| !v.is_empty())
    {
        let fd = value
            .to_str()
            .and_then(|v| v.parse::<i32>().ok())
            .context("invalid auth handoff descriptor")?;
        // SAFETY: F_GETFD only inspects the named descriptor; no data is read.
        let flags = unsafe { libc::fcntl(fd, libc::F_GETFD) };
        if flags < 0 || flags & libc::FD_CLOEXEC == 0 {
            bail!("sensitive auth descriptor could survive native exec");
        }
    }
    Ok(())
}

#[cfg(target_os = "linux")]
fn session_directory(
    root: &TrustedAuthorityRoot,
    session: &str,
    create: bool,
) -> Result<crate::execution::agent_runtime::host_session_authority::trusted_fs::TrustedDirectory> {
    let mut dir = root.directory().open_directory("run").or_else(|error| {
        if create {
            root.directory().create_directory("run")
        } else {
            Err(error)
        }
    })?;
    for part in ["agent-hub", "handles", session] {
        dir = if create {
            dir.create_directory(part)?
        } else {
            dir.open_directory(part)?
        };
    }
    Ok(dir)
}

impl NativeLaunchContext {
    #[cfg(target_os = "linux")]
    pub(crate) fn stage(
        authority: &HostSessionAuthority,
        session: &str,
        backend: &str,
        model: InitialCodexModel,
        prepare_endpoint: impl FnOnce() -> Result<String>,
    ) -> Result<(Self, String)> {
        validate_native_posture()?;
        let opened_root = TrustedAuthorityRoot::open(Path::new(
            &authority.bootstrap_home().identity()?.physical_path,
        ))?;
        let root = &opened_root;
        // Initial context precedes issue_start, so the accepted root may still
        // be V1. Both accessors validate the same existing accepted store.
        // This schema peek chooses an accessor; only its validated result is
        // authority. A malformed/unreadable V2 is never retried as V1.
        let header: serde_json::Value = serde_json::from_slice(
            &root
                .directory()
                .open_directory("authority-v1")?
                .open_file("state-root-v1.json")?
                .read_all()?,
        )?;
        let authority_store_id = match header
            .get("schema_version")
            .and_then(serde_json::Value::as_u64)
        {
            Some(1) => authority.read_root()?.authority_store_id,
            Some(2 | 3) => authority.read_preserved_start_root_v2()?.authority_store_id,
            _ => bail!("unsupported accepted authority root schema"),
        };
        let dir = session_directory(root, session, true)?;
        let home = Path::new(&root.identity().physical_path)
            .join("run/agent-hub/handles")
            .join(session)
            .join("native-codex-home");
        let existing = match dir.open_file(CONTEXT_FILE) {
            Ok(file) => Some(file.read_all()?),
            Err(error) => {
                // Distinguish absence from a substituted/unreadable existing artifact.
                match std::fs::symlink_metadata(
                    Path::new(&root.identity().physical_path)
                        .join("run/agent-hub/handles")
                        .join(session)
                        .join(CONTEXT_FILE),
                ) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => None,
                    _ => return Err(error.into()),
                }
            }
        };
        let context = if let Some(bytes) = existing {
            let context: Self = serde_json::from_slice(&bytes)?;
            context.validate_binding(&authority_store_id, root.identity(), session, backend)?;
            if context.model != model {
                bail!("initial native model conflicts with staged context");
            }
            context
        } else {
            let context = Self {
                schema_version: 1,
                authority_store_id,
                accepted_home: root.identity().clone(),
                original_session_id: session.to_string(),
                backend_id: backend.to_string(),
                model,
                codex_home: home,
                base_url: prepare_endpoint()?,
            };
            context.validate_endpoint()?;
            dir.create_directory("native-codex-home")?;
            let bytes = canonical_json::to_vec(&serde_json::to_value(&context)?)?;
            let mut file = dir.create_file(CONTEXT_FILE)?;
            file.write_all(&bytes)?;
            file.sync()?;
            dir.sync()?;
            context
        };
        let commitment = digest(&canonical_json::to_vec(&serde_json::to_value(&context)?)?);
        Ok((context, commitment))
    }

    fn validate_endpoint(&self) -> Result<()> {
        let Some(port) = self
            .base_url
            .strip_prefix("http://127.0.0.1:")
            .and_then(|v| v.strip_suffix("/v1"))
        else {
            bail!("native endpoint must be the managed loopback gateway");
        };
        if port.parse::<u16>().ok().filter(|p| *p != 0).is_none() {
            bail!("invalid managed native endpoint");
        }
        Ok(())
    }

    fn validate_binding(
        &self,
        store: &str,
        home: &CanonicalDirectoryV1,
        original: &str,
        backend: &str,
    ) -> Result<()> {
        let expected_home = Path::new(&home.physical_path)
            .join("run/agent-hub/handles")
            .join(original)
            .join("native-codex-home");
        if self.schema_version != 1
            || self.authority_store_id != store
            || &self.accepted_home != home
            || self.original_session_id != original
            || self.backend_id != backend
            || self.codex_home != expected_home
            || !matches!(backend, "cli:codex" | "cli:codex-host")
        {
            bail!("native context authority binding mismatch");
        }
        self.validate_endpoint()
    }

    // Callers already hold exact authenticated authority. Inspect only that
    // authenticated origin before any filesystem work: default launches must
    // not add a reconciliation or new refusal opportunity.
    pub(crate) fn for_resolved(
        current: &crate::execution::agent_runtime::host_session_authority::ResolvedCurrentAuthorityV1,
        backend: &str,
    ) -> Result<Option<Self>> {
        let DurableSessionAuthorityOriginV1::StartIntent {
            intent_id,
            issuer_request_id,
            ..
        } = &current.authority.origin;
        let Some(expected_digest) = marker_digest(issuer_request_id)? else {
            return Ok(None);
        };
        if current.host_attach_contract.backend_id != backend {
            bail!("native current backend mismatch");
        }
        #[cfg(target_os = "linux")]
        {
            let root = TrustedAuthorityRoot::open(&substrate_common::paths::substrate_home()?)?;
            let authority = HostSessionAuthority::from_trusted_root(TrustedAuthorityRoot::open(
                Path::new(&root.identity().physical_path),
            )?)?;
            validate_native_posture()?;
            let state = authority.read_preserved_start_root_v2()?;
            let intent = state
                .transition_intent_map
                .get(intent_id)
                .context("native original Start intent missing")?;
            if intent.issuer_request_id != *issuer_request_id {
                bail!("native original Start origin mismatch");
            }
            let original = &intent.orchestration_session_id;
            let bytes = session_directory(&root, original, false)?
                .open_file(CONTEXT_FILE)?
                .read_all()?;
            if digest(&bytes) != expected_digest {
                bail!("native context digest mismatch");
            }
            let context: Self = serde_json::from_slice(&bytes)?;
            context.validate_binding(
                &state.authority_store_id,
                root.identity(),
                original,
                backend,
            )?;
            session_directory(&root, original, false)?.open_directory("native-codex-home")?;
            Ok(Some(context))
        }
        #[cfg(not(target_os = "linux"))]
        {
            let _ = (intent_id, expected_digest);
            bail!("selected native context requires Linux");
        }
    }

    #[cfg(all(test, target_os = "linux"))]
    pub(crate) fn for_session(session: &str, backend: &str) -> Result<Option<Self>> {
        let authority = HostSessionAuthority::from_trusted_root(TrustedAuthorityRoot::open(
            &substrate_common::paths::substrate_home()?,
        )?)?;
        let current = authority.resolve_current_exact(session, None)?;
        Self::for_resolved(&current, backend)
    }

    pub(crate) fn configure(
        &self,
        builder: codex::CodexClientBuilder,
        initial: bool,
    ) -> Result<codex::CodexClientBuilder> {
        validate_native_posture()?;
        #[cfg(target_os = "linux")]
        {
            let root = TrustedAuthorityRoot::open(Path::new(&self.accepted_home.physical_path))?;
            if root.identity() != &self.accepted_home {
                bail!("native accepted home replaced");
            }
            session_directory(&root, &self.original_session_id, false)?
                .open_directory("native-codex-home")?;
        }
        let builder = builder
            .codex_home(&self.codex_home)
            .config_override("model", serde_json::to_string(self.model.as_str())?)
            .config_override("model_provider", "\"openai\"")
            .config_override("openai_base_url", serde_json::to_string(&self.base_url)?)
            .config_override("cli_auth_credentials_store", "\"ephemeral\"");
        Ok(if initial {
            builder.model(self.model.as_str())
        } else {
            builder
        })
    }

    pub(crate) fn environment(&self, env: &mut BTreeMap<String, String>) -> Result<()> {
        validate_native_posture()?;
        validate_source_posture(
            WORKLOAD_KEYS.iter().any(|key| env.contains_key(*key)),
            false,
        )?;
        for key in BLANK_KEYS {
            env.insert((*key).to_string(), String::new());
        }
        for key in ["NO_PROXY", "no_proxy"] {
            env.insert(key.to_string(), "127.0.0.1,::1,localhost".to_string());
        }
        env.insert(
            "CODEX_HOME".to_string(),
            self.codex_home.display().to_string(),
        );
        Ok(())
    }
}

pub(crate) fn selected_issuer(context_digest: &str, plan_digest: &str) -> String {
    format!("{MARKER}{context_digest}_{plan_digest}")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_native_source_posture_rejects_administrative_and_workload_auth() {
        assert!(validate_source_posture(false, false).is_ok());
        assert!(validate_source_posture(true, false).is_err());
        assert!(validate_source_posture(false, true).is_err());
        assert_eq!(BLANK_KEYS.len(), 26);
        let unique: std::collections::BTreeSet<_> = BLANK_KEYS.iter().collect();
        assert_eq!(unique.len(), BLANK_KEYS.len());
        assert!(BLANK_KEYS.contains(&"CODEX_ACCESS_TOKEN"));
    }

    #[test]
    fn test_selected_origin_marker_is_strict_and_legacy_is_unchanged() {
        assert_eq!(marker_digest("req_legacy").unwrap(), None);
        let digest = "a".repeat(64);
        let issuer = selected_issuer(&digest, &"b".repeat(64));
        assert_eq!(marker_digest(&issuer).unwrap(), Some(digest.as_str()));
        for bad in [
            "req_native",
            "req_native_a_b",
            "req_native__",
            "req_native_bad",
        ] {
            assert!(marker_digest(bad).is_err());
        }
    }
}
