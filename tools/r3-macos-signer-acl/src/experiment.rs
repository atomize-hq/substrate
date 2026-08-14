//! Fixed filesystem/executable bindings for the two-repetition creator-route experiment.

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use std::ffi::CString;
use std::fs::File;
use std::io::{Read, Write};
use std::mem::MaybeUninit;
use std::os::fd::{AsRawFd, FromRawFd, OwnedFd};
use std::path::Path;

use crate::{
    ExactDeleteClassification, ExactDeleteReceipt, FixedRepetitionV2, MARKER_PATH, MARKER_ROOT,
};

const MARKER_NAME: &str = "creator-route.v2";
const NEXT_MARKER_NAME: &str = ".creator-route.v2.next";
const ROLLBACK_MARKER_NAME: &str = "creator-emergency-rollback.v2.json";
const NEXT_ROLLBACK_MARKER_NAME: &str = ".creator-emergency-rollback.v2.next";
const ROOT_MODE: libc::mode_t = libc::S_IFDIR | 0o700;
const MARKER_MODE: libc::mode_t = libc::S_IFREG | 0o600;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CreatorRollbackPhaseV2 {
    Prepared,
    Invoked,
    Observed,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreatorRollbackMarkerV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repetition: FixedRepetitionV2,
    pub creator_state_marker: String,
    pub failure_observation_sha256: String,
    pub phase: CreatorRollbackPhaseV2,
    pub receipt_sha256: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CreatorRollbackReceiptV2 {
    pub schema_owner: String,
    pub schema_version: u32,
    pub repetition: FixedRepetitionV2,
    pub failure_observation_sha256: String,
    pub exact_delete: Option<ExactDeleteReceipt>,
    pub exact_identity_absent: bool,
}

impl CreatorRollbackReceiptV2 {
    pub fn validate(&self, marker: &CreatorRollbackMarkerV2) -> Result<()> {
        marker.validate()?;
        if self.schema_owner != "substrate.r3-macos-signer-acl.creator-emergency-rollback-receipt"
            || self.schema_version != 2
            || self.repetition != marker.repetition
            || self.failure_observation_sha256 != marker.failure_observation_sha256
            || !self.exact_identity_absent
        {
            bail!("creator rollback receipt changed its exact failure or absence binding")
        }
        if let Some(delete) = &self.exact_delete {
            if delete.raw_os_status != 0
                || delete.classification != ExactDeleteClassification::DeletedAndAbsent
                || delete.present_after
            {
                bail!("creator rollback deletion was not exact success and absence")
            }
        }
        if marker.phase == CreatorRollbackPhaseV2::Observed
            && marker.receipt_sha256.as_deref()
                != Some(&substrate_common::macos_retirement_v2::document_sha256_v2(
                    self,
                )?)
        {
            bail!("creator rollback observed cursor does not bind its exact receipt bytes")
        }
        Ok(())
    }
}

impl CreatorRollbackMarkerV2 {
    pub fn validate(&self) -> Result<()> {
        let state = MarkerState::parse(self.creator_state_marker.as_bytes())?;
        let receipt_shape_valid = match self.phase {
            CreatorRollbackPhaseV2::Observed => {
                self.receipt_sha256.as_deref().is_some_and(is_sha256)
            }
            CreatorRollbackPhaseV2::Prepared | CreatorRollbackPhaseV2::Invoked => {
                self.receipt_sha256.is_none()
            }
        };
        if self.schema_owner != "substrate.r3-macos-signer-acl.creator-emergency-rollback"
            || self.schema_version != 2
            || state_repetition(state) != Some(self.repetition)
            || !is_sha256(&self.failure_observation_sha256)
            || !receipt_shape_valid
        {
            bail!("creator rollback marker changed its closed identity or cursor")
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MarkerState {
    FirstQueryPrepared,
    FirstQueryInvoked,
    FirstFreshCreatePrepared,
    FirstFreshCreateInvoked,
    FirstWrongPrepared,
    FirstWrongInvoked,
    FirstFreshDeletePrepared,
    FirstFreshDeleteInvoked,
    FirstAbsentRetryPrepared,
    FirstAbsentRetryInvoked,
    SecondQueryPrepared,
    SecondQueryInvoked,
    SecondFreshCreatePrepared,
    SecondFreshCreateInvoked,
    SecondWrongPrepared,
    SecondWrongInvoked,
    SecondFreshDeletePrepared,
    SecondFreshDeleteInvoked,
    SecondAbsentRetryPrepared,
    SecondAbsentRetryInvoked,
    Complete,
}

impl MarkerState {
    pub const fn all() -> [Self; 21] {
        [
            Self::FirstQueryPrepared,
            Self::FirstQueryInvoked,
            Self::FirstFreshCreatePrepared,
            Self::FirstFreshCreateInvoked,
            Self::FirstWrongPrepared,
            Self::FirstWrongInvoked,
            Self::FirstFreshDeletePrepared,
            Self::FirstFreshDeleteInvoked,
            Self::FirstAbsentRetryPrepared,
            Self::FirstAbsentRetryInvoked,
            Self::SecondQueryPrepared,
            Self::SecondQueryInvoked,
            Self::SecondFreshCreatePrepared,
            Self::SecondFreshCreateInvoked,
            Self::SecondWrongPrepared,
            Self::SecondWrongInvoked,
            Self::SecondFreshDeletePrepared,
            Self::SecondFreshDeleteInvoked,
            Self::SecondAbsentRetryPrepared,
            Self::SecondAbsentRetryInvoked,
            Self::Complete,
        ]
    }

    pub const fn marker(self) -> &'static [u8] {
        match self {
            Self::FirstQueryPrepared => b"creator-route-v2:first-query-prepared\n",
            Self::FirstQueryInvoked => b"creator-route-v2:first-query-invoked\n",
            Self::FirstFreshCreatePrepared => b"creator-route-v2:first-fresh-create-prepared\n",
            Self::FirstFreshCreateInvoked => b"creator-route-v2:first-fresh-create-invoked\n",
            Self::FirstWrongPrepared => b"creator-route-v2:first-wrong-prepared\n",
            Self::FirstWrongInvoked => b"creator-route-v2:first-wrong-invoked\n",
            Self::FirstFreshDeletePrepared => b"creator-route-v2:first-fresh-delete-prepared\n",
            Self::FirstFreshDeleteInvoked => b"creator-route-v2:first-fresh-delete-invoked\n",
            Self::FirstAbsentRetryPrepared => b"creator-route-v2:first-absent-retry-prepared\n",
            Self::FirstAbsentRetryInvoked => b"creator-route-v2:first-absent-retry-invoked\n",
            Self::SecondQueryPrepared => b"creator-route-v2:second-query-prepared\n",
            Self::SecondQueryInvoked => b"creator-route-v2:second-query-invoked\n",
            Self::SecondFreshCreatePrepared => b"creator-route-v2:second-fresh-create-prepared\n",
            Self::SecondFreshCreateInvoked => b"creator-route-v2:second-fresh-create-invoked\n",
            Self::SecondWrongPrepared => b"creator-route-v2:second-wrong-prepared\n",
            Self::SecondWrongInvoked => b"creator-route-v2:second-wrong-invoked\n",
            Self::SecondFreshDeletePrepared => b"creator-route-v2:second-fresh-delete-prepared\n",
            Self::SecondFreshDeleteInvoked => b"creator-route-v2:second-fresh-delete-invoked\n",
            Self::SecondAbsentRetryPrepared => b"creator-route-v2:second-absent-retry-prepared\n",
            Self::SecondAbsentRetryInvoked => b"creator-route-v2:second-absent-retry-invoked\n",
            Self::Complete => b"creator-route-v2:complete\n",
        }
    }

    pub fn parse(bytes: &[u8]) -> Result<Self> {
        Self::all()
            .into_iter()
            .find(|state| state.marker() == bytes)
            .context("creator-route marker has unknown or noncanonical content")
    }
}

pub const fn query_states(
    repetition: FixedRepetitionV2,
) -> (MarkerState, MarkerState, MarkerState) {
    match repetition {
        FixedRepetitionV2::First => (
            MarkerState::FirstQueryPrepared,
            MarkerState::FirstQueryInvoked,
            MarkerState::FirstFreshCreatePrepared,
        ),
        FixedRepetitionV2::Second => (
            MarkerState::SecondQueryPrepared,
            MarkerState::SecondQueryInvoked,
            MarkerState::SecondFreshCreatePrepared,
        ),
    }
}

pub const fn fresh_create_states(
    repetition: FixedRepetitionV2,
) -> (MarkerState, MarkerState, MarkerState) {
    match repetition {
        FixedRepetitionV2::First => (
            MarkerState::FirstFreshCreatePrepared,
            MarkerState::FirstFreshCreateInvoked,
            MarkerState::FirstWrongPrepared,
        ),
        FixedRepetitionV2::Second => (
            MarkerState::SecondFreshCreatePrepared,
            MarkerState::SecondFreshCreateInvoked,
            MarkerState::SecondWrongPrepared,
        ),
    }
}

pub const fn wrong_states(
    repetition: FixedRepetitionV2,
) -> (MarkerState, MarkerState, MarkerState) {
    match repetition {
        FixedRepetitionV2::First => (
            MarkerState::FirstWrongPrepared,
            MarkerState::FirstWrongInvoked,
            MarkerState::FirstFreshDeletePrepared,
        ),
        FixedRepetitionV2::Second => (
            MarkerState::SecondWrongPrepared,
            MarkerState::SecondWrongInvoked,
            MarkerState::SecondFreshDeletePrepared,
        ),
    }
}

pub const fn fresh_delete_states(
    repetition: FixedRepetitionV2,
) -> (MarkerState, MarkerState, MarkerState) {
    match repetition {
        FixedRepetitionV2::First => (
            MarkerState::FirstFreshDeletePrepared,
            MarkerState::FirstFreshDeleteInvoked,
            MarkerState::FirstAbsentRetryPrepared,
        ),
        FixedRepetitionV2::Second => (
            MarkerState::SecondFreshDeletePrepared,
            MarkerState::SecondFreshDeleteInvoked,
            MarkerState::SecondAbsentRetryPrepared,
        ),
    }
}

pub const fn absent_retry_states(
    repetition: FixedRepetitionV2,
) -> (MarkerState, MarkerState, MarkerState) {
    match repetition {
        FixedRepetitionV2::First => (
            MarkerState::FirstAbsentRetryPrepared,
            MarkerState::FirstAbsentRetryInvoked,
            MarkerState::SecondQueryPrepared,
        ),
        FixedRepetitionV2::Second => (
            MarkerState::SecondAbsentRetryPrepared,
            MarkerState::SecondAbsentRetryInvoked,
            MarkerState::Complete,
        ),
    }
}

pub struct MarkerRoot {
    directory: OwnedFd,
}

impl MarkerRoot {
    pub fn open() -> Result<Self> {
        if Path::new(MARKER_PATH).parent() != Some(Path::new(MARKER_ROOT)) {
            bail!("compiled marker path is not inside its compiled root")
        }
        let root = CString::new(MARKER_ROOT).context("encode compiled marker root")?;
        // SAFETY: fixed path and exact no-follow directory flags.
        let raw = unsafe {
            libc::open(
                root.as_ptr(),
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open fixed marker root");
        }
        // SAFETY: raw is newly owned.
        let directory = unsafe { OwnedFd::from_raw_fd(raw) };
        require_identity(
            &fstat(directory.as_raw_fd())?,
            ROOT_MODE,
            "fixed marker root",
        )?;
        Ok(Self { directory })
    }

    pub fn read_state(&self) -> Result<MarkerState> {
        let name = component(MARKER_NAME)?;
        // SAFETY: fixed component under held directory and no-follow.
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("open fixed creator marker");
        }
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        let metadata = fstat(descriptor.as_raw_fd())?;
        require_identity(&metadata, MARKER_MODE, "creator marker")?;
        if metadata.st_size < 0 || metadata.st_size > 128 {
            bail!("creator marker length is outside its fixed bound")
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        File::from(descriptor)
            .read_to_end(&mut bytes)
            .context("read creator marker")?;
        MarkerState::parse(&bytes)
    }

    pub fn transition(&self, expected: MarkerState, next: MarkerState) -> Result<()> {
        if self.read_state()? != expected {
            bail!("creator marker differs from its one expected predecessor")
        }
        let next_name = component(NEXT_MARKER_NAME)?;
        // SAFETY: remove only the compiled temp component; absence is allowed.
        let unlink = unsafe { libc::unlinkat(self.directory.as_raw_fd(), next_name.as_ptr(), 0) };
        if unlink != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT) {
            return Err(std::io::Error::last_os_error())
                .context("remove stale creator temp marker");
        }
        self.create_exact(NEXT_MARKER_NAME, next.marker())?;
        let marker = component(MARKER_NAME)?;
        // SAFETY: both fixed components are under the held root.
        if unsafe {
            libc::renameat(
                self.directory.as_raw_fd(),
                next_name.as_ptr(),
                self.directory.as_raw_fd(),
                marker.as_ptr(),
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error()).context("replace creator marker");
        }
        self.sync_directory()?;
        if self.read_state()? != next {
            bail!("creator marker transition failed strict readback")
        }
        Ok(())
    }

    pub fn read_rollback(&self) -> Result<Option<CreatorRollbackMarkerV2>> {
        let name = component(ROLLBACK_MARKER_NAME)?;
        // SAFETY: fixed component under held directory and no-follow.
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            )
        };
        if raw < 0 {
            let error = std::io::Error::last_os_error();
            if error.raw_os_error() == Some(libc::ENOENT) {
                return Ok(None);
            }
            return Err(error).context("open fixed creator rollback marker");
        }
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        let metadata = fstat(descriptor.as_raw_fd())?;
        require_identity(&metadata, MARKER_MODE, "creator rollback marker")?;
        if metadata.st_size < 0 || metadata.st_size > 4096 {
            bail!("creator rollback marker length is outside its fixed bound")
        }
        let mut bytes = Vec::with_capacity(metadata.st_size as usize);
        File::from(descriptor).read_to_end(&mut bytes)?;
        let value: CreatorRollbackMarkerV2 = serde_json::from_slice(&bytes)?;
        if serde_json::to_vec(&value)? != bytes {
            bail!("creator rollback marker is not canonical JSON")
        }
        value.validate()?;
        Ok(Some(value))
    }

    pub fn prepare_rollback(
        &self,
        repetition: FixedRepetitionV2,
        failure_observation_sha256: &str,
    ) -> Result<CreatorRollbackMarkerV2> {
        if self.read_rollback()?.is_some() {
            bail!("creator rollback marker already exists")
        }
        let state = self.read_state()?;
        if state_repetition(state) != Some(repetition) || !is_sha256(failure_observation_sha256) {
            bail!("creator rollback preparation differs from the current fixed repetition")
        }
        let value = CreatorRollbackMarkerV2 {
            schema_owner: "substrate.r3-macos-signer-acl.creator-emergency-rollback".to_owned(),
            schema_version: 2,
            repetition,
            creator_state_marker: std::str::from_utf8(state.marker())?.to_owned(),
            failure_observation_sha256: failure_observation_sha256.to_owned(),
            phase: CreatorRollbackPhaseV2::Prepared,
            receipt_sha256: None,
        };
        value.validate()?;
        self.create_exact(ROLLBACK_MARKER_NAME, &serde_json::to_vec(&value)?)?;
        self.sync_directory()?;
        Ok(value)
    }

    pub fn replace_rollback(
        &self,
        expected: CreatorRollbackPhaseV2,
        next: &CreatorRollbackMarkerV2,
    ) -> Result<()> {
        let current = self
            .read_rollback()?
            .context("creator rollback cursor is absent")?;
        next.validate()?;
        if current.phase != expected
            || current.repetition != next.repetition
            || current.creator_state_marker != next.creator_state_marker
            || current.failure_observation_sha256 != next.failure_observation_sha256
        {
            bail!("creator rollback cursor replacement changed its predecessor binding")
        }
        let next_name = component(NEXT_ROLLBACK_MARKER_NAME)?;
        // SAFETY: remove only fixed temp component; absence is allowed.
        let unlink = unsafe { libc::unlinkat(self.directory.as_raw_fd(), next_name.as_ptr(), 0) };
        if unlink != 0 && std::io::Error::last_os_error().raw_os_error() != Some(libc::ENOENT) {
            return Err(std::io::Error::last_os_error())
                .context("remove stale creator rollback temp marker");
        }
        self.create_exact(NEXT_ROLLBACK_MARKER_NAME, &serde_json::to_vec(next)?)?;
        let current_name = component(ROLLBACK_MARKER_NAME)?;
        // SAFETY: both fixed components are beneath the held root.
        if unsafe {
            libc::renameat(
                self.directory.as_raw_fd(),
                next_name.as_ptr(),
                self.directory.as_raw_fd(),
                current_name.as_ptr(),
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error()).context("replace creator rollback cursor");
        }
        self.sync_directory()?;
        if self.read_rollback()?.as_ref() != Some(next) {
            bail!("creator rollback cursor failed exact durable readback")
        }
        Ok(())
    }

    /// Removes only the two compiled creator leaves after a durably observed exact rollback.
    /// The caller must subsequently remove and fsync the now-empty compiled marker directory.
    pub fn remove_after_observed_rollback(
        &self,
        receipt: &CreatorRollbackReceiptV2,
    ) -> Result<MarkerState> {
        let cursor = self
            .read_rollback()?
            .context("creator rollback cursor is absent during terminal restoration")?;
        if cursor.phase != CreatorRollbackPhaseV2::Observed {
            bail!("creator rollback is not durably observed during terminal restoration")
        }
        receipt.validate(&cursor)?;
        let state = self.read_state()?;
        if state.marker() != cursor.creator_state_marker.as_bytes() {
            bail!("creator marker changed after its observed rollback")
        }
        for name in [ROLLBACK_MARKER_NAME, MARKER_NAME] {
            let component = component(name)?;
            // SAFETY: both names are fixed leaves below the held, validated marker directory.
            if unsafe { libc::unlinkat(self.directory.as_raw_fd(), component.as_ptr(), 0) } != 0 {
                return Err(std::io::Error::last_os_error())
                    .with_context(|| format!("remove exact observed creator leaf {name}"));
            }
        }
        self.sync_directory()?;
        if self.read_rollback()?.is_some() {
            bail!("creator rollback cursor remained after exact terminal removal")
        }
        Ok(state)
    }

    fn create_exact(&self, name: &str, bytes: &[u8]) -> Result<()> {
        let name = component(name)?;
        // SAFETY: fixed component, exclusive no-follow creation.
        let raw = unsafe {
            libc::openat(
                self.directory.as_raw_fd(),
                name.as_ptr(),
                libc::O_WRONLY | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0o600,
            )
        };
        if raw < 0 {
            return Err(std::io::Error::last_os_error()).context("create creator temp marker");
        }
        // SAFETY: raw is newly owned.
        let descriptor = unsafe { OwnedFd::from_raw_fd(raw) };
        // SAFETY: live descriptor and fixed ownership/mode.
        if unsafe { libc::fchown(descriptor.as_raw_fd(), 0, 0) } != 0
            || unsafe { libc::fchmod(descriptor.as_raw_fd(), 0o600) } != 0
        {
            return Err(std::io::Error::last_os_error()).context("bind creator marker identity");
        }
        let mut file = File::from(descriptor);
        file.write_all(bytes).context("write creator marker")?;
        file.sync_all().context("sync creator marker")?;
        require_identity(
            &fstat(file.as_raw_fd())?,
            MARKER_MODE,
            "written creator marker",
        )
    }

    fn sync_directory(&self) -> Result<()> {
        // SAFETY: held directory descriptor.
        if unsafe { libc::fsync(self.directory.as_raw_fd()) } != 0 {
            return Err(std::io::Error::last_os_error()).context("sync creator marker root");
        }
        Ok(())
    }
}

fn state_repetition(state: MarkerState) -> Option<FixedRepetitionV2> {
    use MarkerState as M;
    match state {
        M::FirstQueryPrepared
        | M::FirstQueryInvoked
        | M::FirstFreshCreatePrepared
        | M::FirstFreshCreateInvoked
        | M::FirstWrongPrepared
        | M::FirstWrongInvoked
        | M::FirstFreshDeletePrepared
        | M::FirstFreshDeleteInvoked
        | M::FirstAbsentRetryPrepared
        | M::FirstAbsentRetryInvoked => Some(FixedRepetitionV2::First),
        M::SecondQueryPrepared
        | M::SecondQueryInvoked
        | M::SecondFreshCreatePrepared
        | M::SecondFreshCreateInvoked
        | M::SecondWrongPrepared
        | M::SecondWrongInvoked
        | M::SecondFreshDeletePrepared
        | M::SecondFreshDeleteInvoked
        | M::SecondAbsentRetryPrepared
        | M::SecondAbsentRetryInvoked => Some(FixedRepetitionV2::Second),
        M::Complete => None,
    }
}

fn is_sha256(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

pub fn verify_closed_process_surface(expected: &str) -> Result<()> {
    if std::env::args_os().count() != 1 {
        bail!("sealed experiment binary rejects all arguments")
    }
    if std::env::vars_os().any(|(name, _)| {
        name.as_os_str()
            .as_encoded_bytes()
            .starts_with(b"SUBSTRATE_")
    }) {
        bail!("sealed experiment binary rejects all SUBSTRATE_* environment inputs")
    }
    let cwd = std::env::current_dir().context("resolve creator cwd")?;
    if cwd.as_os_str().as_encoded_bytes() != MARKER_ROOT.as_bytes() {
        bail!("sealed experiment binary cwd differs from its compiled root")
    }
    let current = std::env::current_exe().context("resolve creator executable")?;
    if current.as_os_str().as_encoded_bytes() != expected.as_bytes() {
        bail!("experiment executable differs from its one compiled path")
    }
    // SAFETY: scalar process identity call.
    if unsafe { libc::geteuid() } != 0 {
        bail!("sealed creator-route binary is not root")
    }
    clear_and_verify_environment()?;
    Ok(())
}

/// Clear ambient process input before any caller can construct a Security-framework authority.
/// These sealed binaries are single-threaded at this point in their startup sequence.
pub(crate) fn clear_and_verify_environment() -> Result<()> {
    let keys: Vec<_> = std::env::vars_os().map(|(key, _)| key).collect();
    for key in keys {
        std::env::remove_var(key);
    }
    if std::env::vars_os().next().is_some() {
        bail!("sealed experiment process environment did not become empty")
    }
    Ok(())
}

fn component(value: &str) -> Result<CString> {
    if value.is_empty() || value.contains(['/', '\0', '\n', '\r']) {
        bail!("fixed marker component is invalid")
    }
    CString::new(value).context("encode fixed marker component")
}

fn fstat(fd: i32) -> Result<libc::stat> {
    let mut value = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: writable output and live fd.
    if unsafe { libc::fstat(fd, value.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("fstat creator object");
    }
    // SAFETY: successful fstat initialized value.
    Ok(unsafe { value.assume_init() })
}

fn require_identity(value: &libc::stat, expected_mode: libc::mode_t, label: &str) -> Result<()> {
    if value.st_uid != 0
        || value.st_gid != 0
        || (value.st_mode & (libc::S_IFMT | 0o7777)) != expected_mode
        || (expected_mode & libc::S_IFMT == libc::S_IFREG && value.st_nlink != 1)
    {
        bail!("{label} is not exact root:wheel with its compiled identity")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn marker_parser_accepts_exactly_two_closed_repetitions() {
        assert_eq!(MarkerState::all().len(), 21);
        for state in MarkerState::all() {
            assert_eq!(MarkerState::parse(state.marker()).unwrap(), state);
        }
        assert!(MarkerState::parse(b"creator-route-v2:third-query-prepared\n").is_err());
    }

    #[test]
    fn compiled_marker_is_beneath_fixed_private_var_root() {
        assert!(MARKER_ROOT.starts_with("/private/var/"));
        assert_eq!(
            Path::new(MARKER_PATH).parent(),
            Some(Path::new(MARKER_ROOT))
        );
        assert!(!MARKER_ROOT.starts_with("/var/"));
    }
}
