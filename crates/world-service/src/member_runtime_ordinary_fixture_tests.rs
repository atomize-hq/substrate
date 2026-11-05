//! Linux unit-test executable custody for the ordinary component fixture only.
//! The wire descriptor stays the canonical guest literal. This held local file
//! does not qualify installation, guest namespaces, kernel isolation or providers.
use super::*;
use std::os::{fd::AsRawFd, unix::fs::FileExt};

const GUEST_CODEX: &str = "/var/lib/substrate/world-deps/bin/codex";

#[derive(PartialEq, Eq)]
struct FixtureWorkerIdentityV1 {
    start: transport_api_types::OrdinaryStartSelectionCarrierV1,
    runtime: transport_api_types::OrdinaryGatewayRuntimeIdentityV1,
    selected: transport_api_types::ResolvedAgentSelectionV1,
}

impl FixtureWorkerIdentityV1 {
    fn identity_for_effect(effect: &crate::service::OrdinaryWorldEffectContextV1) -> Result<Self> {
        effect.revalidate_for_effect()?;
        Self::identity_data(effect)
    }

    // Identity projection only. The child-specific composition below authenticates
    // the source issuer/child effect before this may be used for source data.
    fn identity_data(effect: &crate::service::OrdinaryWorldEffectContextV1) -> Result<Self> {
        let mut runtime = effect.authenticated_runtime();
        runtime.validate().map_err(anyhow::Error::msg)?;
        match &mut runtime {
            transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
                run_id, ..
            } => {
                // Admission authenticates each current run independently. All
                // other original worker and World identity must stay exact.
                run_id.clear();
            }
            _ => anyhow::bail!("component executable requires an admitted World worker"),
        }
        Ok(Self {
            start: effect.ordinary_start_selection().clone(),
            runtime,
            selected: effect.requested_selection().clone(),
        })
    }
}

#[derive(PartialEq, Eq)]
struct FixtureFileIdentityV1 {
    dev: u64,
    ino: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    nlink: u64,
    len: u64,
    mtime: i64,
    mtime_nsec: i64,
    ctime: i64,
    ctime_nsec: i64,
}

pub(super) struct CheckedOrdinaryNativeFixtureV1 {
    path: std::path::PathBuf,
    workspace: std::path::PathBuf,
    parent: fs::File,
    file: fs::File,
    identity: FixtureFileIdentityV1,
    digest: String,
    worker: Mutex<Option<FixtureWorkerIdentityV1>>,
}

impl CheckedOrdinaryNativeFixtureV1 {
    /// Separate child custody over the same checked executable; never relabel the
    /// original worker. This module exists only in the component test build.
    pub(super) fn for_authenticated_fork(
        &self,
        child: &Arc<crate::service::OrdinaryWorldEffectContextV1>,
        source: &Arc<crate::service::OrdinaryWorldEffectContextV1>,
    ) -> Result<Arc<Self>> {
        // This genuine current child/source issuer join precedes any child mapping.
        child.validate_initial_fork_source(source)?;
        self.physical()?;
        let source_identity = FixtureWorkerIdentityV1::identity_data(source)?;
        if self
            .worker
            .lock()
            .map_err(|_| anyhow!("fixture worker lock poisoned"))?
            .as_ref()
            != Some(&source_identity)
        {
            anyhow::bail!("component Fork executable does not hold the exact admitted source");
        }
        let child_identity = FixtureWorkerIdentityV1::identity_for_effect(child)?;
        if child_identity == source_identity {
            anyhow::bail!("component Fork executable requires a distinct authenticated child");
        }
        let mapped = Arc::new(Self {
            path: self.path.clone(),
            workspace: self.workspace.clone(),
            parent: self.parent.try_clone()?,
            file: self.file.try_clone()?,
            identity: Self::file_identity(&self.file.metadata()?),
            digest: self.digest.clone(),
            worker: Mutex::new(Some(child_identity)),
        });
        self.physical()?;
        mapped.physical()?;
        child.validate_initial_fork_source(source)?;
        Ok(mapped)
    }

    pub(super) fn open(path: &Path, digest: &str) -> Result<Self> {
        if unsafe { libc::geteuid() } == 0
            || unsafe { libc::getuid() } != unsafe { libc::geteuid() }
            || !path.is_absolute()
            || path
                .file_name()
                .is_none_or(|name| name != "codex" && name != "claude")
            || fs::canonicalize(path)? != path
            || digest.len() != 64
            || !digest
                .bytes()
                .all(|c| c.is_ascii_hexdigit() && !c.is_ascii_uppercase())
        {
            anyhow::bail!("component executable requires exact canonical fixture path and digest");
        }
        let workspace = path.parent().context("fixture parent")?.to_path_buf();
        let parent = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_DIRECTORY | libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(&workspace)?;
        let file = fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)?;
        let fixture = Self {
            path: path.to_path_buf(),
            workspace,
            parent,
            identity: Self::file_identity(&file.metadata()?),
            file,
            digest: digest.to_owned(),
            worker: Mutex::new(None),
        };
        fixture.physical()?;
        Ok(fixture)
    }

    fn file_identity(m: &fs::Metadata) -> FixtureFileIdentityV1 {
        FixtureFileIdentityV1 {
            dev: m.dev(),
            ino: m.ino(),
            uid: m.uid(),
            gid: m.gid(),
            mode: m.mode(),
            nlink: m.nlink(),
            len: m.len(),
            mtime: m.mtime(),
            mtime_nsec: m.mtime_nsec(),
            ctime: m.ctime(),
            ctime_nsec: m.ctime_nsec(),
        }
    }

    fn physical(&self) -> Result<()> {
        self.check_workspace(&self.workspace)?;
        let held = self.file.metadata()?;
        let named = fs::symlink_metadata(&self.path)?;
        if !held.is_file()
            || !named.is_file()
            || held.uid() != unsafe { libc::geteuid() }
            || held.permissions().mode() & 0o777 != 0o700
            || held.nlink() != 1
            || held.len() > 1024 * 1024
            || Self::file_identity(&held) != self.identity
            || Self::file_identity(&named) != self.identity
        {
            anyhow::bail!("component executable physical custody changed");
        }
        let mut bytes = vec![0; held.len() as usize];
        self.file.read_exact_at(&mut bytes, 0)?;
        if format!("{:x}", Sha256::digest(bytes)) != self.digest
            || Self::file_identity(&self.file.metadata()?) != self.identity
        {
            anyhow::bail!("component executable content changed");
        }
        Ok(())
    }

    fn check_workspace(&self, workspace: &Path) -> Result<()> {
        let held = self.parent.metadata()?;
        let named = fs::symlink_metadata(&self.workspace)?;
        if workspace != self.workspace
            || !named.is_dir()
            || (held.dev(), held.ino(), held.uid(), held.gid(), held.mode())
                != (
                    named.dev(),
                    named.ino(),
                    named.uid(),
                    named.gid(),
                    named.mode(),
                )
            || held.uid() != unsafe { libc::geteuid() }
            || held.permissions().mode() & 0o022 != 0
        {
            anyhow::bail!("component executable workspace custody changed");
        }
        Ok(())
    }

    fn require_canonical_descriptor(
        &self,
        runtime: &transport_api_types::ResolvedMemberRuntimeDescriptorV1,
    ) -> Result<()> {
        let exact = match runtime.backend_kind {
            MemberRuntimeBackendKindV1::Codex => {
                self.path.file_name().is_some_and(|name| name == "codex")
                    && runtime.binary_path == GUEST_CODEX
            }
            MemberRuntimeBackendKindV1::ClaudeCode => {
                // Current ordinary Claude resolves its configured absolute file.
                // This is no guest-entrypoint substitution or installation claim.
                self.path.file_name().is_some_and(|name| name == "claude")
                    && Path::new(&runtime.binary_path) == self.path
            }
        };
        if !exact {
            anyhow::bail!("component executable differs from its exact backend descriptor");
        }
        Ok(())
    }

    pub(super) fn bind_dispatch(
        &self,
        effect: &crate::service::OrdinaryWorldEffectContextV1,
        dispatch: &MemberDispatchRequestV1,
        binding: &SharedWorldBindingSnapshot,
        workspace: &Path,
    ) -> Result<std::path::PathBuf> {
        self.require_canonical_descriptor(&dispatch.resolved_runtime)?;
        self.check_workspace(workspace)?;
        self.physical()?;
        let actual = transport_api_types::OrdinaryGatewayRuntimeIdentityV1::WorldWorker {
            orchestration_session_id: dispatch.orchestration_session_id.clone(),
            caller_participant_id: dispatch.orchestrator_participant_id.clone(),
            participant_id: dispatch.participant_id.clone(),
            backend_id: dispatch.backend_id.clone(),
            run_id: dispatch.run_id.clone(),
            world_id: binding.world_id.clone(),
            world_generation: binding.world_generation,
        };
        if actual != effect.authenticated_runtime()
            || dispatch.world_id != binding.world_id
            || dispatch.world_generation != binding.world_generation
        {
            anyhow::bail!("component executable dispatch differs from actual admitted target");
        }
        let identity = FixtureWorkerIdentityV1::identity_for_effect(effect)?;
        let mut worker = self
            .worker
            .lock()
            .map_err(|_| anyhow!("fixture worker lock poisoned"))?;
        match worker.as_ref() {
            None => *worker = Some(identity),
            Some(original) if original == &identity => {}
            _ => anyhow::bail!("component executable cannot authorize another worker"),
        }
        // Retained policy preparation requires a regular non-symlink path.
        // Keep the FD and revalidate its exact named identity before every
        // native effect; this component seam does not exclude atomic swaps.
        Ok(self.path.clone())
    }

    pub(super) fn revalidate_for_effect(
        &self,
        effect: &crate::service::OrdinaryWorldEffectContextV1,
    ) -> Result<()> {
        self.physical()?;
        let current = FixtureWorkerIdentityV1::identity_for_effect(effect)?;
        if self
            .worker
            .lock()
            .map_err(|_| anyhow!("fixture worker lock poisoned"))?
            .as_ref()
            != Some(&current)
        {
            anyhow::bail!("component executable does not hold this admitted worker");
        }
        Ok(())
    }
}

#[test]
fn test_checked_ordinary_fixture_rejects_substitution() {
    use std::os::unix::fs::symlink;
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("codex");
    let bytes = b"#!/bin/sh\nexit 0\n";
    fs::write(&path, bytes).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    let digest = format!("{:x}", Sha256::digest(bytes));
    assert!(CheckedOrdinaryNativeFixtureV1::open(&path, "bad").is_err());
    assert!(CheckedOrdinaryNativeFixtureV1::open(&path, &"0".repeat(64)).is_err());
    let held = CheckedOrdinaryNativeFixtureV1::open(&path, &digest).unwrap();
    held.physical().unwrap();
    let fd_path = std::path::PathBuf::from(format!(
        "/proc/{}/fd/{}",
        std::process::id(),
        held.file.as_raw_fd()
    ));
    assert!(fs::symlink_metadata(&fd_path)
        .unwrap()
        .file_type()
        .is_symlink());
    assert!(!fs::symlink_metadata(&fd_path)
        .unwrap()
        .file_type()
        .is_file());
    assert!(fs::symlink_metadata(&held.path)
        .unwrap()
        .file_type()
        .is_file());
    assert_eq!(
        (
            fs::metadata(&fd_path).unwrap().dev(),
            fs::metadata(&fd_path).unwrap().ino()
        ),
        (
            held.file.metadata().unwrap().dev(),
            held.file.metadata().unwrap().ino()
        )
    );
    assert!(held
        .check_workspace(&temp.path().join("different"))
        .is_err());
    for (kind, binary) in [
        (MemberRuntimeBackendKindV1::Codex, "/bin/true"),
        (MemberRuntimeBackendKindV1::ClaudeCode, GUEST_CODEX),
    ] {
        assert!(held
            .require_canonical_descriptor(&transport_api_types::ResolvedMemberRuntimeDescriptorV1 {
                backend_kind: kind,
                binary_path: binary.into()
            })
            .is_err());
    }
    let claude_path = temp.path().join("claude");
    fs::write(&claude_path, bytes).unwrap();
    fs::set_permissions(&claude_path, fs::Permissions::from_mode(0o700)).unwrap();
    let claude = CheckedOrdinaryNativeFixtureV1::open(&claude_path, &digest).unwrap();
    let claude_descriptor = transport_api_types::ResolvedMemberRuntimeDescriptorV1 {
        backend_kind: MemberRuntimeBackendKindV1::ClaudeCode,
        binary_path: claude_path.display().to_string(),
    };
    claude
        .require_canonical_descriptor(&claude_descriptor)
        .unwrap();
    assert!(held
        .require_canonical_descriptor(&claude_descriptor)
        .is_err());
    let mut wrong = claude_descriptor;
    wrong.binary_path = temp.path().join("other-claude").display().to_string();
    assert!(claude.require_canonical_descriptor(&wrong).is_err());
    fs::rename(&path, temp.path().join("saved")).unwrap();
    fs::write(&path, bytes).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o700)).unwrap();
    assert!(held.physical().is_err());
    let held = CheckedOrdinaryNativeFixtureV1::open(&path, &digest).unwrap();
    fs::write(&path, b"#!/bin/sh\nexit 1\n").unwrap();
    assert!(held.physical().is_err());
    fs::remove_file(&path).unwrap();
    symlink(temp.path().join("saved"), &path).unwrap();
    assert!(CheckedOrdinaryNativeFixtureV1::open(&path, &digest).is_err());
    fs::remove_file(&path).unwrap();
    fs::hard_link(temp.path().join("saved"), &path).unwrap();
    assert!(CheckedOrdinaryNativeFixtureV1::open(&path, &digest).is_err());
    fs::remove_file(&path).unwrap();
    fs::rename(temp.path().join("saved"), &path).unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o720)).unwrap();
    assert!(CheckedOrdinaryNativeFixtureV1::open(&path, &digest).is_err());
    assert!(MemberRuntimeManager::new()
        .ordinary_native_fixture
        .is_none());
}
