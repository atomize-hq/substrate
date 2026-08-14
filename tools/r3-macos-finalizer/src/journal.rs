#[cfg(test)]
use std::cell::Cell;
use std::cell::RefCell;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{Read, Seek, SeekFrom, Write};
use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::{Deserialize, Serialize};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, parse_canonical_v2, sha256_hex_v2, HostRetirementStateV2,
    MAC_R3_MAX_HOST_TARGETS_V2,
};

use crate::contract::{
    JournalEvent, JournalEventKind, JOURNAL_OWNER, JOURNAL_VERSION,
    MAX_EFFECT_INVOCATION_ATTEMPTS_V2,
};

const EMPTY_HEAD: &str = "0000000000000000000000000000000000000000000000000000000000000000";
const HEAD_TEMPORARY_PREFIX: &str = ".HEAD.tmp.";
// FinalizerAccepted + (Prepared + bounded Invoked attempts + Observed) per target +
// EffectsComplete + TerminalAcknowledgementBound + Complete.
pub const MAX_JOURNAL_GENERATIONS: u64 =
    (2 + MAX_EFFECT_INVOCATION_ATTEMPTS_V2 as u64) * MAC_R3_MAX_HOST_TARGETS_V2 as u64 + 4;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct JournalGeneration {
    pub schema_owner: String,
    pub schema_version: u32,
    pub scope_id: String,
    pub generation: u64,
    pub predecessor_head_sha256: String,
    pub request_digest: String,
    pub authority_bytes_sha256: String,
    pub successor_capsule_sha256: String,
    /// Digest of the exact, full process-instance attestation that crossed the first
    /// FinalizerAccepted boundary. It never changes across successor generations.
    pub accepted_peer_attestation_sha256: String,
    /// Digest of the restart-stable coordinator principal (credentials plus executable/code
    /// identity). A later process instance may rejoin only when this value is unchanged.
    pub accepted_peer_identity_sha256: String,
    pub event: JournalEvent,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalHead {
    pub generation: u64,
    pub sha256: String,
    pub record: JournalGeneration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct JournalClaimIdentity {
    pub request_digest: String,
    pub authority_bytes_sha256: String,
    pub successor_capsule_sha256: String,
    pub accepted_peer_attestation_sha256: String,
    pub accepted_peer_identity_sha256: String,
}

pub struct LockedJournal {
    root: PathBuf,
    scope_dir: PathBuf,
    // The installed root directory is also the launch/cleanup activation membrane. The fixed
    // lock-file membrane is acquired inside it, followed by the per-scope lock. Keeping the
    // exact no-follow directory descriptors alive makes path replacement observable while all
    // three locks remain held for the journal lifetime.
    _scope_lock: File,
    _scope_directory: File,
    _root_lock: File,
    _activation_root: File,
    expected_uid: u32,
    // The exclusive scope lock makes an authenticated head stable between this process's own
    // appends. Cache the fully verified chain so a maximum-size plan remains linear rather than
    // re-reading every predecessor before each of its ~12k journal transitions.
    head_cache: RefCell<HeadCache>,
    #[cfg(test)]
    full_chain_load_count: Cell<u64>,
}

#[derive(Debug, Clone)]
enum HeadCache {
    Unknown,
    Absent(DirectoryStamp),
    Present {
        head: Box<JournalHead>,
        scope_stamp: DirectoryStamp,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DirectoryStamp {
    device: u64,
    inode: u64,
    uid: u32,
    gid: u32,
    mode: u32,
    modified_seconds: i64,
    modified_nanoseconds: i64,
    changed_seconds: i64,
    changed_nanoseconds: i64,
}

impl LockedJournal {
    pub fn open_existing_fixed(
        root: &Path,
        scope_id: &str,
        expected_uid: u32,
    ) -> Result<Option<Self>> {
        require_component(scope_id, "journal scope")?;
        match fs::symlink_metadata(root) {
            Ok(_) => verify_directory(root, 0o700, expected_uid)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("inspect existing finalizer journal root"),
        }
        let activation_root = open_fixed_directory(root, 0o700, expected_uid)?;
        lock_file_exclusive(&activation_root, "lock existing finalizer activation root")?;
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;

        // The root installer deliberately creates this exact empty activation directory before
        // the first finalizer process. Missing `journal-root.lock` is recoverable only while that
        // held directory is still completely empty. Any other child proves this is not the one
        // crash/installation frontier that can safely be completed.
        let root_lock = open_or_initialize_empty_directory_lock(
            &activation_root,
            root,
            "journal-root.lock",
            expected_uid,
            "journal root",
        )?;
        if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&root_lock), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error()).context("lock existing finalizer root");
        }
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;
        verify_held_file(
            &root_lock,
            &root.join("journal-root.lock"),
            0o600,
            expected_uid,
        )?;

        let scope_dir = root.join(scope_id);
        match fs::symlink_metadata(&scope_dir) {
            Ok(_) => verify_directory(&scope_dir, 0o700, expected_uid)?,
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
            Err(error) => return Err(error).context("inspect existing finalizer scope"),
        }
        let scope_directory = open_fixed_directory(&scope_dir, 0o700, expected_uid)?;
        let scope_lock = open_or_initialize_empty_directory_lock(
            &scope_directory,
            &scope_dir,
            "journal.lock",
            expected_uid,
            "journal scope",
        )?;
        if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&scope_lock), libc::LOCK_EX) } != 0
        {
            return Err(std::io::Error::last_os_error()).context("lock existing finalizer scope");
        }
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;
        verify_held_directory(&scope_directory, &scope_dir, 0o700, expected_uid)?;
        verify_held_file(
            &scope_lock,
            &scope_dir.join("journal.lock"),
            0o600,
            expected_uid,
        )?;
        Ok(Some(Self {
            root: root.to_path_buf(),
            scope_dir,
            _scope_lock: scope_lock,
            _scope_directory: scope_directory,
            _root_lock: root_lock,
            _activation_root: activation_root,
            expected_uid,
            head_cache: RefCell::new(HeadCache::Unknown),
            #[cfg(test)]
            full_chain_load_count: Cell::new(0),
        }))
    }

    pub fn open_fixed(root: &Path, scope_id: &str, expected_uid: u32) -> Result<Self> {
        require_component(scope_id, "journal scope")?;
        ensure_directory(root, 0o700, expected_uid)?;
        let activation_root = open_fixed_directory(root, 0o700, expected_uid)?;
        lock_file_exclusive(&activation_root, "lock fixed finalizer activation root")?;
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;
        let root_lock = open_or_initialize_empty_directory_lock(
            &activation_root,
            root,
            "journal-root.lock",
            expected_uid,
            "journal root",
        )?;
        if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&root_lock), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error()).context("lock fixed finalizer root");
        }
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;
        verify_held_file(
            &root_lock,
            &root.join("journal-root.lock"),
            0o600,
            expected_uid,
        )?;

        let scope_dir = root.join(scope_id);
        ensure_directory(&scope_dir, 0o700, expected_uid)?;
        let scope_directory = open_fixed_directory(&scope_dir, 0o700, expected_uid)?;
        let scope_lock = open_or_initialize_empty_directory_lock(
            &scope_directory,
            &scope_dir,
            "journal.lock",
            expected_uid,
            "journal scope",
        )?;
        let rc =
            unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(&scope_lock), libc::LOCK_EX) };
        if rc != 0 {
            return Err(std::io::Error::last_os_error()).context("lock exact finalizer scope");
        }
        verify_held_directory(&activation_root, root, 0o700, expected_uid)?;
        verify_held_directory(&scope_directory, &scope_dir, 0o700, expected_uid)?;
        verify_held_file(
            &scope_lock,
            &scope_dir.join("journal.lock"),
            0o600,
            expected_uid,
        )?;
        Ok(Self {
            root: root.to_path_buf(),
            scope_dir,
            _scope_lock: scope_lock,
            _scope_directory: scope_directory,
            _root_lock: root_lock,
            _activation_root: activation_root,
            expected_uid,
            head_cache: RefCell::new(HeadCache::Unknown),
            #[cfg(test)]
            full_chain_load_count: Cell::new(0),
        })
    }

    pub fn head(&self) -> Result<Option<JournalHead>> {
        let cached = self.head_cache.borrow().clone();
        match cached {
            HeadCache::Absent(stamp) => {
                verify_directory(&self.scope_dir, 0o700, self.expected_uid)?;
                if self.scope_directory_stamp()? == stamp {
                    return Ok(None);
                }
            }
            HeadCache::Present { head, scope_stamp } => {
                verify_directory(&self.scope_dir, 0o700, self.expected_uid)?;
                self.require_disk_head_identity(head.generation, &head.sha256)?;
                let bytes = read_exact_required_file(
                    &self.generation_path(head.generation),
                    self.expected_uid,
                    0o400,
                )?;
                let record: JournalGeneration = parse_canonical_v2(&bytes)?;
                validate_generation(&record, head.generation)?;
                if sha256_hex_v2(&bytes) != head.sha256 || record != head.record {
                    bail!("cached journal head generation changed under the scope lock")
                }
                if self.scope_directory_stamp()? == scope_stamp {
                    return Ok(Some(*head));
                }
            }
            HeadCache::Unknown => {}
        }
        let head = self.load_and_recover_head()?;
        let scope_stamp = self.scope_directory_stamp()?;
        *self.head_cache.borrow_mut() = match &head {
            Some(head) => HeadCache::Present {
                head: Box::new(head.clone()),
                scope_stamp,
            },
            None => HeadCache::Absent(scope_stamp),
        };
        Ok(head)
    }

    fn load_and_recover_head(&self) -> Result<Option<JournalHead>> {
        #[cfg(test)]
        self.full_chain_load_count
            .set(self.full_chain_load_count.get() + 1);
        let head_path = self.scope_dir.join("HEAD");
        let mut repaired = false;
        loop {
            let generations: Vec<_> = self.generation_files()?.collect();
            let pending_head = self.single_head_temporary_path()?;
            if generations.len() as u64 > MAX_JOURNAL_GENERATIONS {
                bail!("journal exceeds its exact maximum generation count")
            }
            let Some(head_bytes) = read_exact_optional_file(&head_path, self.expected_uid, 0o600)?
            else {
                if generations.is_empty() {
                    if pending_head.is_some() {
                        bail!(
                            "journal HEAD temporary has no uniquely derivable generation candidate"
                        )
                    }
                    return Ok(None);
                }
                if repaired || generations.len() != 1 || generations[0] != self.generation_path(1) {
                    bail!("journal has an ambiguous generation set but no durable HEAD")
                }
                let bytes = read_exact_required_file(&generations[0], self.expected_uid, 0o400)?;
                let record: JournalGeneration = parse_canonical_v2(&bytes)?;
                validate_generation(&record, 1)?;
                if record.predecessor_head_sha256 != EMPTY_HEAD {
                    bail!("unreferenced first journal generation has the wrong predecessor")
                }
                validate_event_transition(None, &record.event)?;
                let digest = sha256_hex_v2(&bytes);
                self.commit_recovered_head_candidate(
                    format!("{:020} {digest}\n", 1).as_bytes(),
                    pending_head.as_deref(),
                )?;
                repaired = true;
                continue;
            };
            let head_text = std::str::from_utf8(&head_bytes).context("HEAD is not UTF-8")?;
            let Some((generation_text, digest_text)) = head_text
                .strip_suffix('\n')
                .and_then(|value| value.split_once(' '))
            else {
                bail!("journal HEAD framing is invalid")
            };
            if generation_text.len() != 20
                || !generation_text.bytes().all(|byte| byte.is_ascii_digit())
                || !is_digest(digest_text)
            {
                bail!("journal HEAD identity is invalid")
            }
            let generation = generation_text
                .parse::<u64>()
                .context("parse HEAD generation")?;
            if generation == 0 || generation > MAX_JOURNAL_GENERATIONS {
                bail!("journal HEAD generation is outside the exact bound")
            }
            let generation_path = self.generation_path(generation);
            let record_bytes =
                read_exact_required_file(&generation_path, self.expected_uid, 0o400)?;
            let sha256 = sha256_hex_v2(&record_bytes);
            if sha256 != digest_text {
                bail!("journal HEAD does not bind its generation bytes")
            }
            let record: JournalGeneration = parse_canonical_v2(&record_bytes)?;
            validate_generation(&record, generation)?;
            if generations.len() == generation as usize + 1
                && generations.last() == Some(&self.generation_path(generation + 1))
            {
                if repaired || generation == MAX_JOURNAL_GENERATIONS {
                    bail!("journal has an extra successor outside its exact recovery bound")
                }
                let successor_bytes = read_exact_required_file(
                    self.generation_path(generation + 1).as_path(),
                    self.expected_uid,
                    0o400,
                )?;
                let successor: JournalGeneration = parse_canonical_v2(&successor_bytes)?;
                validate_generation(&successor, generation + 1)?;
                if successor.predecessor_head_sha256 != sha256
                    || successor.scope_id != record.scope_id
                    || successor.request_digest != record.request_digest
                    || successor.authority_bytes_sha256 != record.authority_bytes_sha256
                    || successor.successor_capsule_sha256 != record.successor_capsule_sha256
                    || successor.accepted_peer_attestation_sha256
                        != record.accepted_peer_attestation_sha256
                    || successor.accepted_peer_identity_sha256
                        != record.accepted_peer_identity_sha256
                {
                    bail!("unreferenced journal successor does not exact-continue the claim")
                }
                validate_event_transition(Some(&record.event), &successor.event)?;
                let successor_hash = sha256_hex_v2(&successor_bytes);
                self.commit_recovered_head_candidate(
                    format!("{:020} {successor_hash}\n", generation + 1).as_bytes(),
                    pending_head.as_deref(),
                )?;
                repaired = true;
                continue;
            }
            if generations.len() != generation as usize {
                bail!("journal generation set has a gap, fork, or ambiguous orphan")
            }
            if pending_head.is_some() {
                bail!("journal HEAD temporary does not bind a unique uncommitted successor")
            }
            self.verify_chain_to(&record, &sha256)?;
            return Ok(Some(JournalHead {
                generation,
                sha256,
                record,
            }));
        }
    }

    pub fn append(
        &self,
        expected_head: Option<&JournalHead>,
        claim: &JournalClaimIdentity,
        event: JournalEvent,
    ) -> Result<JournalHead> {
        for value in [
            claim.request_digest.as_str(),
            claim.authority_bytes_sha256.as_str(),
            claim.successor_capsule_sha256.as_str(),
            claim.accepted_peer_attestation_sha256.as_str(),
            claim.accepted_peer_identity_sha256.as_str(),
        ] {
            if !is_digest(value) {
                bail!("journal join is not a lowercase SHA-256")
            }
        }
        let live = self.head()?;
        match (&live, expected_head) {
            (None, None) => {}
            (Some(live), Some(expected))
                if live.generation == expected.generation && live.sha256 == expected.sha256 => {}
            _ => bail!("journal generation CAS changed before append"),
        }
        let generation = live.as_ref().map_or(1, |head| head.generation + 1);
        if generation > MAX_JOURNAL_GENERATIONS {
            bail!("journal append exceeds the exact maximum generation count")
        }
        let predecessor = live
            .as_ref()
            .map_or_else(|| EMPTY_HEAD.to_string(), |head| head.sha256.clone());
        let record = JournalGeneration {
            schema_owner: JOURNAL_OWNER.to_string(),
            schema_version: JOURNAL_VERSION,
            scope_id: self
                .scope_dir
                .file_name()
                .and_then(OsStr::to_str)
                .expect("validated UTF-8 scope")
                .to_string(),
            generation,
            predecessor_head_sha256: predecessor,
            request_digest: claim.request_digest.clone(),
            authority_bytes_sha256: claim.authority_bytes_sha256.clone(),
            successor_capsule_sha256: claim.successor_capsule_sha256.clone(),
            accepted_peer_attestation_sha256: claim.accepted_peer_attestation_sha256.clone(),
            accepted_peer_identity_sha256: claim.accepted_peer_identity_sha256.clone(),
            event,
        };
        validate_generation(&record, generation)?;
        validate_event_transition(live.as_ref().map(|head| &head.record.event), &record.event)?;
        let bytes = canonical_bytes_v2(&record)?;
        let digest = sha256_hex_v2(&bytes);
        let path = self.generation_path(generation);
        publish_immutable_no_clobber(&path, &bytes, 0o400, self.expected_uid)?;
        let reopened = read_exact_required_file(&path, self.expected_uid, 0o400)?;
        if reopened != bytes || sha256_hex_v2(&reopened) != digest {
            bail!("journal generation failed reopen byte/hash verification")
        }

        // Recheck only the O(1) HEAD pointer after generation persistence. The exclusive scope
        // lock already serialized writers, and `head()` fully authenticated the predecessor
        // chain once when this journal object opened. A crash here still leaves exactly the one
        // successor that a fresh `load_and_recover_head` validates before installing its HEAD.
        self.require_disk_head_matches(live.as_ref())?;
        let head_bytes = format!("{generation:020} {digest}\n").into_bytes();
        self.replace_head_cas(&head_bytes)?;
        self.require_disk_head_identity(generation, &digest)?;
        let durable = JournalHead {
            generation,
            sha256: digest,
            record,
        };
        *self.head_cache.borrow_mut() = HeadCache::Present {
            head: Box::new(durable.clone()),
            scope_stamp: self.scope_directory_stamp()?,
        };
        Ok(durable)
    }

    pub fn persist_terminal_response(&self, request_digest: &str, bytes: &[u8]) -> Result<String> {
        if !is_digest(request_digest) || bytes.is_empty() {
            bail!("terminal response identity is invalid")
        }
        self.persist_immutable_artifact("terminal.response", bytes, "terminal response")
    }

    pub fn persist_preserving_response(
        &self,
        request_digest: &str,
        bytes: &[u8],
    ) -> Result<String> {
        if !is_digest(request_digest) || bytes.is_empty() {
            bail!("preserving response identity is invalid")
        }
        self.persist_immutable_artifact("preserving.response", bytes, "preserving response")
    }

    pub fn persist_terminal_binding_artifact(
        &self,
        basename: &str,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<()> {
        if !matches!(
            basename,
            "effects.response" | "parity.proof" | "terminal.acknowledgement"
        ) || !is_digest(expected_sha256)
            || sha256_hex_v2(bytes) != expected_sha256
        {
            bail!("terminal binding artifact name, bytes, or digest is invalid")
        }
        let digest =
            self.persist_immutable_artifact(basename, bytes, "terminal binding artifact")?;
        if digest != expected_sha256 {
            bail!("terminal binding artifact digest changed after persistence")
        }
        Ok(())
    }

    pub fn persist_acceptance_artifact(
        &self,
        basename: &str,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<()> {
        if !matches!(basename, "authority.request" | "peer.attestation")
            || bytes.is_empty()
            || !is_digest(expected_sha256)
            || sha256_hex_v2(bytes) != expected_sha256
        {
            bail!("acceptance artifact name, bytes, or digest is invalid")
        }
        let path = self.scope_dir.join(basename);
        match read_exact_optional_file(&path, self.expected_uid, 0o400)? {
            Some(existing) if existing == bytes => return Ok(()),
            Some(_) => bail!("durable acceptance artifact already differs"),
            None => {}
        }
        publish_immutable_no_clobber(&path, bytes, 0o400, self.expected_uid)?;
        let reopened = read_exact_required_file(&path, self.expected_uid, 0o400)?;
        if reopened != bytes || sha256_hex_v2(&reopened) != expected_sha256 {
            bail!("acceptance artifact failed reopen byte/hash verification")
        }
        self.refresh_cached_scope_stamp_after_owned_mutation()?;
        Ok(())
    }

    /// Persist the current restart-instance attestation under a content-derived, closed name.
    /// The initial attestation must already exist and match the digest frozen into generation 1;
    /// a rejoin can therefore never substitute for missing first-acceptance evidence.
    pub fn persist_rejoin_peer_attestation(
        &self,
        bytes: &[u8],
        expected_sha256: &str,
        initial_attestation_sha256: &str,
    ) -> Result<()> {
        if bytes.is_empty()
            || !is_digest(expected_sha256)
            || !is_digest(initial_attestation_sha256)
            || sha256_hex_v2(bytes) != expected_sha256
        {
            bail!("rejoin peer attestation bytes or digest is invalid")
        }
        let initial = read_exact_required_file(
            &self.scope_dir.join("peer.attestation"),
            self.expected_uid,
            0o400,
        )?;
        if sha256_hex_v2(&initial) != initial_attestation_sha256 {
            bail!("initial peer attestation does not match the accepted journal claim")
        }
        if initial == bytes {
            return Ok(());
        }
        let basename = format!("peer.rejoin.{expected_sha256}.attestation");
        let digest =
            self.persist_immutable_artifact(&basename, bytes, "restart peer attestation")?;
        if digest != expected_sha256 {
            bail!("restart peer attestation digest changed after persistence")
        }
        Ok(())
    }

    /// Persist the canonical observation for one compiled effect ordinal.  The caller cannot
    /// choose a filename: the signed plan ordinal is the complete artifact namespace.
    pub fn persist_effect_observation_artifact(
        &self,
        ordinal: u16,
        bytes: &[u8],
        expected_sha256: &str,
    ) -> Result<()> {
        if ordinal == 0
            || bytes.is_empty()
            || !is_digest(expected_sha256)
            || sha256_hex_v2(bytes) != expected_sha256
        {
            bail!("effect-observation artifact ordinal, bytes, or digest is invalid")
        }
        let basename = effect_observation_basename(ordinal);
        let digest =
            self.persist_immutable_artifact(&basename, bytes, "effect-observation artifact")?;
        if digest != expected_sha256 {
            bail!("effect-observation artifact digest changed after persistence")
        }
        Ok(())
    }

    pub fn effect_observation_artifact(&self, ordinal: u16) -> Result<Option<Vec<u8>>> {
        if ordinal == 0 {
            bail!("effect-observation artifact ordinal zero is invalid")
        }
        read_exact_optional_file(
            &self.scope_dir.join(effect_observation_basename(ordinal)),
            self.expected_uid,
            0o400,
        )
    }

    pub fn acceptance_artifact(&self, basename: &str) -> Result<Option<Vec<u8>>> {
        if !matches!(basename, "authority.request" | "peer.attestation") {
            bail!("acceptance artifact name is not fixed")
        }
        read_exact_optional_file(&self.scope_dir.join(basename), self.expected_uid, 0o400)
    }

    pub fn terminal_response(&self) -> Result<Option<Vec<u8>>> {
        read_exact_optional_file(
            &self.scope_dir.join("terminal.response"),
            self.expected_uid,
            0o400,
        )
    }

    pub fn preserving_response(&self) -> Result<Option<Vec<u8>>> {
        read_exact_optional_file(
            &self.scope_dir.join("preserving.response"),
            self.expected_uid,
            0o400,
        )
    }

    pub fn terminal_binding_artifact(&self, basename: &str) -> Result<Option<Vec<u8>>> {
        if !matches!(
            basename,
            "effects.response" | "parity.proof" | "terminal.acknowledgement"
        ) {
            bail!("terminal binding artifact name is not fixed")
        }
        read_exact_optional_file(&self.scope_dir.join(basename), self.expected_uid, 0o400)
    }

    pub fn generation(&self, generation: u64) -> Result<JournalGeneration> {
        if generation == 0 || generation > MAX_JOURNAL_GENERATIONS {
            bail!("journal generation is outside the exact bound")
        }
        let bytes =
            read_exact_required_file(&self.generation_path(generation), self.expected_uid, 0o400)?;
        let record: JournalGeneration = parse_canonical_v2(&bytes)?;
        validate_generation(&record, generation)?;
        if generation == 1 {
            if record.predecessor_head_sha256 != EMPTY_HEAD {
                bail!("first journal generation has a nonempty predecessor")
            }
            validate_event_transition(None, &record.event)?;
        } else {
            let predecessor_bytes = read_exact_required_file(
                &self.generation_path(generation - 1),
                self.expected_uid,
                0o400,
            )?;
            let predecessor: JournalGeneration = parse_canonical_v2(&predecessor_bytes)?;
            validate_generation(&predecessor, generation - 1)?;
            if record.predecessor_head_sha256 != sha256_hex_v2(&predecessor_bytes)
                || record.scope_id != predecessor.scope_id
                || record.request_digest != predecessor.request_digest
                || record.authority_bytes_sha256 != predecessor.authority_bytes_sha256
                || record.successor_capsule_sha256 != predecessor.successor_capsule_sha256
                || record.accepted_peer_attestation_sha256
                    != predecessor.accepted_peer_attestation_sha256
                || record.accepted_peer_identity_sha256 != predecessor.accepted_peer_identity_sha256
            {
                bail!("journal generation does not exact-continue its direct predecessor")
            }
            validate_event_transition(Some(&predecessor.event), &record.event)?;
        }
        Ok(record)
    }

    fn require_disk_head_matches(&self, expected: Option<&JournalHead>) -> Result<()> {
        let observed =
            read_exact_optional_file(&self.scope_dir.join("HEAD"), self.expected_uid, 0o600)?;
        match (observed, expected) {
            (None, None) => Ok(()),
            (Some(bytes), Some(head))
                if bytes == format!("{:020} {}\n", head.generation, head.sha256).as_bytes() =>
            {
                Ok(())
            }
            _ => bail!("journal HEAD changed while the exclusive scope lock was held"),
        }
    }

    fn require_disk_head_identity(&self, generation: u64, sha256: &str) -> Result<()> {
        let bytes =
            read_exact_required_file(&self.scope_dir.join("HEAD"), self.expected_uid, 0o600)?;
        if bytes != format!("{generation:020} {sha256}\n").as_bytes() {
            bail!("journal durable HEAD does not exact-match appended generation")
        }
        Ok(())
    }

    fn replace_head_cas(&self, bytes: &[u8]) -> Result<()> {
        let tmp = self
            .scope_dir
            .join(format!(".HEAD.tmp.{}", std::process::id()));
        complete_durable_exact_prefix(&tmp, bytes, 0o600, self.expected_uid)?;
        fs::rename(&tmp, self.scope_dir.join("HEAD")).context("commit journal HEAD CAS")?;
        sync_directory(&self.scope_dir)?;
        let reopened =
            read_exact_required_file(&self.scope_dir.join("HEAD"), self.expected_uid, 0o600)?;
        if reopened != bytes {
            bail!("journal HEAD failed reopen verification")
        }
        Ok(())
    }

    fn commit_recovered_head_candidate(
        &self,
        expected_bytes: &[u8],
        pending_head: Option<&Path>,
    ) -> Result<()> {
        let Some(pending_head) = pending_head else {
            return self.replace_head_cas(expected_bytes);
        };
        complete_durable_exact_prefix(pending_head, expected_bytes, 0o600, self.expected_uid)?;
        fs::rename(pending_head, self.scope_dir.join("HEAD"))
            .context("recover exact journal HEAD temporary")?;
        sync_directory(&self.scope_dir)?;
        let reopened =
            read_exact_required_file(&self.scope_dir.join("HEAD"), self.expected_uid, 0o600)?;
        if reopened != expected_bytes {
            bail!("recovered journal HEAD failed reopen verification")
        }
        match fs::symlink_metadata(pending_head) {
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Ok(_) => bail!("recovered journal HEAD temporary source still exists"),
            Err(error) => {
                return Err(error).context("verify recovered journal HEAD temporary absence")
            }
        }
        Ok(())
    }

    fn verify_chain_to(&self, terminal: &JournalGeneration, terminal_hash: &str) -> Result<()> {
        let generation_files: Vec<_> = self.generation_files()?.collect();
        if generation_files.len() != terminal.generation as usize {
            bail!("journal generation set has a gap, fork, or orphan")
        }
        let mut predecessor = EMPTY_HEAD.to_string();
        let mut predecessor_event: Option<JournalEvent> = None;
        for generation in 1..=terminal.generation {
            let bytes = read_exact_required_file(
                &self.generation_path(generation),
                self.expected_uid,
                0o400,
            )?;
            let record: JournalGeneration = parse_canonical_v2(&bytes)?;
            validate_generation(&record, generation)?;
            if record.scope_id != terminal.scope_id
                || record.request_digest != terminal.request_digest
                || record.authority_bytes_sha256 != terminal.authority_bytes_sha256
                || record.successor_capsule_sha256 != terminal.successor_capsule_sha256
                || record.accepted_peer_attestation_sha256
                    != terminal.accepted_peer_attestation_sha256
                || record.accepted_peer_identity_sha256 != terminal.accepted_peer_identity_sha256
                || record.predecessor_head_sha256 != predecessor
            {
                bail!("journal hash chain or immutable request claim diverged")
            }
            validate_event_transition(predecessor_event.as_ref(), &record.event)?;
            predecessor = sha256_hex_v2(&bytes);
            predecessor_event = Some(record.event);
        }
        if predecessor != terminal_hash {
            bail!("journal terminal hash does not match verified chain")
        }
        Ok(())
    }

    fn generation_files(&self) -> Result<impl Iterator<Item = PathBuf>> {
        let mut paths = Vec::new();
        for entry in fs::read_dir(&self.scope_dir).context("enumerate exact journal scope")? {
            let entry = entry.context("read journal directory entry")?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                bail!("journal contains a non-UTF-8 entry")
            };
            if name.starts_with("generation-") && name.ends_with(".json") {
                paths.push(entry.path());
            }
        }
        paths.sort();
        Ok(paths.into_iter())
    }

    fn single_head_temporary_path(&self) -> Result<Option<PathBuf>> {
        let mut temporary = None;
        for entry in fs::read_dir(&self.scope_dir).context("enumerate exact journal scope")? {
            let entry = entry.context("read journal directory entry")?;
            let name = entry.file_name();
            let Some(name) = name.to_str() else {
                bail!("journal contains a non-UTF-8 entry")
            };
            let Some(pid_text) = name.strip_prefix(HEAD_TEMPORARY_PREFIX) else {
                continue;
            };
            let pid = pid_text
                .parse::<u32>()
                .context("journal HEAD temporary PID is invalid")?;
            if pid == 0 || pid.to_string() != pid_text {
                bail!("journal HEAD temporary PID is not canonical")
            }
            if temporary.replace(entry.path()).is_some() {
                bail!("journal has more than one HEAD temporary candidate")
            }
        }
        Ok(temporary)
    }

    fn generation_path(&self, generation: u64) -> PathBuf {
        self.scope_dir
            .join(format!("generation-{generation:020}.json"))
    }

    fn scope_directory_stamp(&self) -> Result<DirectoryStamp> {
        let metadata = fs::symlink_metadata(&self.scope_dir)
            .context("inspect exact journal scope change stamp")?;
        if !metadata.file_type().is_dir()
            || metadata.uid() != self.expected_uid
            || metadata.mode() & 0o7777 != 0o700
        {
            bail!("exact journal scope change stamp is not the fixed directory identity")
        }
        Ok(DirectoryStamp {
            device: metadata.dev(),
            inode: metadata.ino(),
            uid: metadata.uid(),
            gid: metadata.gid(),
            mode: metadata.mode(),
            modified_seconds: metadata.mtime(),
            modified_nanoseconds: metadata.mtime_nsec(),
            changed_seconds: metadata.ctime(),
            changed_nanoseconds: metadata.ctime_nsec(),
        })
    }

    /// Refresh only the physical directory stamp after this locked journal durably creates one
    /// of its own immutable side artifacts. The authenticated head itself is unchanged. Without
    /// this refresh, the next append would conservatively interpret the journal's own directory
    /// mutation as possible external tampering and revalidate the entire chain once per effect.
    fn refresh_cached_scope_stamp_after_owned_mutation(&self) -> Result<()> {
        let stamp = self.scope_directory_stamp()?;
        match &mut *self.head_cache.borrow_mut() {
            HeadCache::Unknown => {}
            HeadCache::Absent(cached_stamp) => *cached_stamp = stamp,
            HeadCache::Present { scope_stamp, .. } => *scope_stamp = stamp,
        }
        Ok(())
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    fn persist_immutable_artifact(
        &self,
        basename: &str,
        bytes: &[u8],
        label: &str,
    ) -> Result<String> {
        require_component(basename, label)?;
        if bytes.is_empty() {
            bail!("{label} is empty")
        }
        let path = self.scope_dir.join(basename);
        let digest = sha256_hex_v2(bytes);
        match read_exact_optional_file(&path, self.expected_uid, 0o400)? {
            Some(existing) if existing == bytes => return Ok(digest),
            Some(_) => bail!("{label} is immutable and already differs"),
            None => {}
        }
        publish_immutable_no_clobber(&path, bytes, 0o400, self.expected_uid)?;
        let reopened = read_exact_required_file(&path, self.expected_uid, 0o400)?;
        if reopened != bytes || sha256_hex_v2(&reopened) != digest {
            bail!("{label} failed reopen byte/hash verification")
        }
        self.refresh_cached_scope_stamp_after_owned_mutation()?;
        Ok(digest)
    }
}

fn effect_observation_basename(ordinal: u16) -> String {
    format!("effect-observation-{ordinal:05}.json")
}

fn validate_generation(record: &JournalGeneration, generation: u64) -> Result<()> {
    if record.schema_owner != JOURNAL_OWNER
        || record.schema_version != JOURNAL_VERSION
        || record.generation != generation
        || generation == 0
        || generation > MAX_JOURNAL_GENERATIONS
        || !is_digest(&record.predecessor_head_sha256)
        || !is_digest(&record.request_digest)
        || !is_digest(&record.authority_bytes_sha256)
        || !is_digest(&record.successor_capsule_sha256)
        || !is_digest(&record.accepted_peer_attestation_sha256)
        || !is_digest(&record.accepted_peer_identity_sha256)
    {
        bail!("journal generation contract is invalid")
    }
    require_component(&record.scope_id, "journal scope")?;
    validate_event_shape(&record.event)
}

fn validate_event_shape(event: &JournalEvent) -> Result<()> {
    use JournalEventKind as K;

    let no_effect = event.effect_ordinal.is_none()
        && event.effect_role.is_none()
        && event.effect_identity_sha256.is_none()
        && event.effect_invocation_attempt.is_none();
    let effect = event.effect_ordinal.is_some_and(|ordinal| ordinal != 0)
        && event.effect_role.is_some()
        && event
            .effect_identity_sha256
            .as_deref()
            .is_some_and(is_digest);
    let observation_is_digest = event.observation_sha256.as_deref().is_some_and(is_digest);
    let response_is_digest = event.response_sha256.as_deref().is_some_and(is_digest);
    let valid = match event.kind {
        K::FinalizerAccepted => {
            event.host_state == HostRetirementStateV2::FinalizerAccepted
                && no_effect
                && event.observation_sha256.is_none()
                && event.response_sha256.is_none()
                && event.preserving_classification.is_none()
        }
        K::EffectPrepared => {
            event.host_state == HostRetirementStateV2::Removing
                && effect
                && event.effect_invocation_attempt == Some(0)
                && event.observation_sha256.is_none()
                && event.response_sha256.is_none()
                && event.preserving_classification.is_none()
        }
        K::EffectInvoked => {
            event.host_state == HostRetirementStateV2::Removing
                && effect
                && event.effect_invocation_attempt.is_some_and(|attempt| {
                    (1..=MAX_EFFECT_INVOCATION_ATTEMPTS_V2).contains(&attempt)
                })
                && event.observation_sha256.is_none()
                && event.response_sha256.is_none()
                && event.preserving_classification.is_none()
        }
        K::EffectObserved => {
            event.host_state == HostRetirementStateV2::Removing
                && effect
                && event
                    .effect_invocation_attempt
                    .is_some_and(|attempt| attempt <= MAX_EFFECT_INVOCATION_ATTEMPTS_V2)
                && observation_is_digest
                && event.response_sha256.is_none()
                && event.preserving_classification.is_none()
        }
        K::EffectsComplete => {
            event.host_state == HostRetirementStateV2::EffectsComplete
                && no_effect
                && observation_is_digest
                && event.response_sha256.is_none()
                && event.preserving_classification.is_none()
        }
        K::TerminalAcknowledgementBound => {
            event.host_state == HostRetirementStateV2::TerminalAcknowledgementBound
                && no_effect
                && observation_is_digest
                && response_is_digest
                && event.preserving_classification.is_none()
        }
        K::Complete => {
            event.host_state == HostRetirementStateV2::Complete
                && no_effect
                && observation_is_digest
                && response_is_digest
                && event.preserving_classification.is_none()
        }
        K::PreservingStop => {
            matches!(
                event.host_state,
                HostRetirementStateV2::FinalizerAccepted
                    | HostRetirementStateV2::Removing
                    | HostRetirementStateV2::EffectsComplete
            ) && no_effect
                && event.observation_sha256.is_none()
                && event.response_sha256.is_none()
                && matches!(
                    event.preserving_classification.as_deref(),
                    Some(
                        "identity_or_authority_mismatch"
                            | "interaction_required"
                            | "ambiguous_effect_state"
                            | "terminal_proof_mismatch"
                    )
                )
        }
    };
    if !valid {
        bail!("journal event shape, host state, or digest fields are invalid")
    }
    Ok(())
}

fn validate_event_transition(previous: Option<&JournalEvent>, next: &JournalEvent) -> Result<()> {
    use JournalEventKind as K;

    validate_event_shape(next)?;
    let valid = match previous {
        None => next.kind == K::FinalizerAccepted,
        Some(previous) => {
            validate_event_shape(previous)?;
            match (previous.kind, next.kind) {
                (K::FinalizerAccepted, K::EffectPrepared) => next.effect_ordinal == Some(1),
                (K::EffectPrepared, K::EffectInvoked) => {
                    same_effect_identity(previous, next)
                        && next.effect_invocation_attempt == Some(1)
                }
                (K::EffectPrepared, K::EffectObserved) => {
                    same_effect_identity(previous, next)
                        && next.effect_invocation_attempt == Some(0)
                }
                (K::EffectInvoked, K::EffectInvoked) => {
                    same_effect_identity(previous, next)
                        && previous
                            .effect_invocation_attempt
                            .and_then(|attempt| attempt.checked_add(1))
                            == next.effect_invocation_attempt
                }
                (K::EffectInvoked, K::EffectObserved) => {
                    same_effect_identity(previous, next)
                        && previous.effect_invocation_attempt == next.effect_invocation_attempt
                }
                (
                    K::FinalizerAccepted | K::EffectPrepared | K::EffectInvoked | K::EffectObserved,
                    K::PreservingStop,
                ) => true,
                (K::EffectObserved, K::EffectPrepared) => {
                    previous
                        .effect_ordinal
                        .and_then(|ordinal| ordinal.checked_add(1))
                        == next.effect_ordinal
                }
                (K::EffectObserved, K::EffectsComplete) => true,
                (K::EffectsComplete, K::PreservingStop) => {
                    next.host_state == HostRetirementStateV2::EffectsComplete
                        && next.preserving_classification.as_deref()
                            == Some("terminal_proof_mismatch")
                }
                (K::EffectsComplete, K::TerminalAcknowledgementBound) => true,
                (K::TerminalAcknowledgementBound, K::Complete) => true,
                _ => false,
            }
        }
    };
    if !valid {
        bail!("journal event transition is not causally valid")
    }
    Ok(())
}

fn same_effect_identity(previous: &JournalEvent, next: &JournalEvent) -> bool {
    previous.effect_ordinal == next.effect_ordinal
        && previous.effect_role == next.effect_role
        && previous.effect_identity_sha256 == next.effect_identity_sha256
}

fn ensure_directory(path: &Path, mode: u32, expected_uid: u32) -> Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.mode(mode);
    match builder.create(path) {
        Ok(()) => {
            fs::set_permissions(path, fs::Permissions::from_mode(mode))
                .context("set finalizer journal directory mode")?;
            sync_directory(path.parent().context("journal directory has no parent")?)?;
        }
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(error) => return Err(error).context("create fixed finalizer journal directory"),
    }
    verify_directory(path, mode, expected_uid)
}

fn verify_directory(path: &Path, mode: u32, expected_uid: u32) -> Result<()> {
    let metadata = fs::symlink_metadata(path).context("inspect finalizer journal directory")?;
    if !metadata.file_type().is_dir()
        || metadata.file_type().is_symlink()
        || metadata.uid() != expected_uid
        || metadata.mode() & 0o7777 != mode
        || metadata.nlink() < 2
    {
        bail!("finalizer journal directory identity, owner, mode, or link count differs")
    }
    Ok(())
}

fn open_fixed_directory(path: &Path, mode: u32, expected_uid: u32) -> Result<File> {
    let directory = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_DIRECTORY)
        .open(path)
        .context("open exact no-follow journal directory")?;
    verify_held_directory(&directory, path, mode, expected_uid)?;
    Ok(directory)
}

fn verify_held_directory(held: &File, path: &Path, mode: u32, expected_uid: u32) -> Result<()> {
    let descriptor = held
        .metadata()
        .context("inspect held no-follow journal directory")?;
    let live = fs::symlink_metadata(path).context("reinspect held journal directory path")?;
    if !descriptor.file_type().is_dir()
        || !live.file_type().is_dir()
        || descriptor.uid() != expected_uid
        || live.uid() != expected_uid
        || descriptor.mode() & 0o7777 != mode
        || live.mode() & 0o7777 != mode
        || descriptor.nlink() < 2
        || live.nlink() < 2
        || descriptor.dev() != live.dev()
        || descriptor.ino() != live.ino()
        || descriptor.uid() != live.uid()
        || descriptor.gid() != live.gid()
        || descriptor.mode() != live.mode()
        || descriptor.nlink() != live.nlink()
    {
        bail!("held journal directory physical identity differs from its exact path")
    }
    Ok(())
}

fn verify_held_file(held: &File, path: &Path, mode: u32, expected_uid: u32) -> Result<()> {
    let descriptor = held.metadata().context("inspect held fixed journal file")?;
    let live = fs::symlink_metadata(path).context("reinspect held fixed journal file path")?;
    if !descriptor.file_type().is_file()
        || !live.file_type().is_file()
        || descriptor.uid() != expected_uid
        || live.uid() != expected_uid
        || descriptor.mode() & 0o7777 != mode
        || live.mode() & 0o7777 != mode
        || descriptor.nlink() != 1
        || live.nlink() != 1
        || descriptor.len() != 0
        || live.len() != 0
        || descriptor.dev() != live.dev()
        || descriptor.ino() != live.ino()
        || descriptor.uid() != live.uid()
        || descriptor.gid() != live.gid()
        || descriptor.mode() != live.mode()
        || descriptor.nlink() != live.nlink()
    {
        bail!("held fixed journal file physical identity differs from its exact path")
    }
    Ok(())
}

fn exact_directory_names(path: &Path) -> Result<Vec<String>> {
    let mut names = Vec::new();
    for entry in fs::read_dir(path).context("enumerate exact journal initialization directory")? {
        let entry = entry.context("read exact journal initialization directory entry")?;
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| anyhow::anyhow!("journal initialization directory has non-UTF-8 state"))?;
        require_component(&name, "journal initialization directory entry")?;
        names.push(name);
    }
    names.sort();
    Ok(names)
}

fn lock_file_exclusive(file: &File, label: &str) -> Result<()> {
    if unsafe { libc::flock(std::os::fd::AsRawFd::as_raw_fd(file), libc::LOCK_EX) } != 0 {
        return Err(std::io::Error::last_os_error()).context(label.to_owned());
    }
    Ok(())
}

/// Open the one fixed empty lock file, or create it only when its held directory has no children.
/// This admits exactly the install/crash frontier before the first lock creation. A missing lock
/// beside any other state is ambiguous and remains untouched.
fn open_or_initialize_empty_directory_lock(
    held_directory: &File,
    directory_path: &Path,
    basename: &str,
    expected_uid: u32,
    label: &str,
) -> Result<File> {
    require_component(basename, label)?;
    verify_held_directory(held_directory, directory_path, 0o700, expected_uid)?;
    let lock_path = directory_path.join(basename);
    match fs::symlink_metadata(&lock_path) {
        Ok(_) => return open_fixed_file(&lock_path, 0o600, expected_uid, false),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error).context(format!("inspect {label} lock")),
    }
    let names = exact_directory_names(directory_path)?;
    if !names.is_empty() {
        bail!("missing {label} lock has nonempty or alternate sibling state")
    }
    verify_held_directory(held_directory, directory_path, 0o700, expected_uid)?;
    let lock = open_fixed_file(&lock_path, 0o600, expected_uid, true)?;
    lock.sync_all()
        .with_context(|| format!("fsync initialized {label} lock"))?;
    sync_directory(directory_path)?;
    verify_held_directory(held_directory, directory_path, 0o700, expected_uid)?;
    verify_held_file(&lock, &lock_path, 0o600, expected_uid)?;
    Ok(lock)
}

fn open_fixed_file(path: &Path, mode: u32, expected_uid: u32, create: bool) -> Result<File> {
    let mut options = OpenOptions::new();
    options
        .read(true)
        .write(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .mode(mode);
    if create {
        options.create_new(true);
    }
    let file = options
        .open(path)
        .context("open fixed no-follow journal file")?;
    verify_held_file(&file, path, mode, expected_uid)?;
    Ok(file)
}

#[derive(Debug)]
struct RecoverableFileSnapshot {
    bytes: Vec<u8>,
    device: u64,
    inode: u64,
    link_count: u64,
}

fn immutable_publish_temporary_path(path: &Path) -> Result<PathBuf> {
    let basename = path
        .file_name()
        .and_then(OsStr::to_str)
        .context("immutable journal artifact basename is not UTF-8")?;
    require_component(basename, "immutable journal artifact basename")?;
    let temporary_basename = format!(".{basename}.pending");
    require_component(
        &temporary_basename,
        "immutable journal artifact temporary basename",
    )?;
    Ok(path
        .parent()
        .context("immutable journal artifact has no parent")?
        .join(temporary_basename))
}

fn read_recoverable_optional_file(
    path: &Path,
    expected_uid: u32,
    allowed_modes: &[u32],
) -> Result<Option<RecoverableFileSnapshot>> {
    let mut file = match OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(error) => return Err(error).context("open recoverable journal artifact"),
    };
    let metadata = file
        .metadata()
        .context("inspect recoverable journal artifact")?;
    let mode = metadata.mode() & 0o7777;
    if !metadata.file_type().is_file()
        || metadata.uid() != expected_uid
        || !allowed_modes.contains(&mode)
        || !(1..=2).contains(&metadata.nlink())
    {
        bail!("recoverable journal artifact owner, mode, type, or link count differs")
    }
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .context("read recoverable journal artifact")?;
    Ok(Some(RecoverableFileSnapshot {
        bytes,
        device: metadata.dev(),
        inode: metadata.ino(),
        link_count: metadata.nlink(),
    }))
}

/// Complete one exact temporary file without ever truncating bytes. An empty or strict-prefix
/// file is a recognized interrupted write and can receive only its missing suffix. Alternate
/// bytes, extra links, owner/mode drift, and non-files are preserved and rejected.
fn complete_durable_exact_prefix(
    path: &Path,
    expected_bytes: &[u8],
    final_mode: u32,
    expected_uid: u32,
) -> Result<()> {
    if expected_bytes.is_empty() {
        bail!("recoverable journal artifact expected bytes are empty")
    }
    let staging_mode = final_mode | 0o200;
    let mut file = match OpenOptions::new()
        .read(true)
        .write(true)
        .create_new(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
        .mode(staging_mode)
        .open(path)
    {
        Ok(file) => file,
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .context("open interrupted journal artifact temporary")?,
        Err(error) => return Err(error).context("create recoverable journal artifact temporary"),
    };
    let mut metadata = file
        .metadata()
        .context("inspect recoverable journal artifact temporary")?;
    let mode = metadata.mode() & 0o7777;
    if !metadata.file_type().is_file()
        || metadata.uid() != expected_uid
        || !matches!(mode, value if value == final_mode || value == staging_mode)
        || metadata.nlink() != 1
    {
        bail!("recoverable journal artifact temporary identity differs")
    }
    let device = metadata.dev();
    let inode = metadata.ino();
    let mut observed = Vec::new();
    file.read_to_end(&mut observed)
        .context("read interrupted journal artifact temporary")?;
    if !expected_bytes.starts_with(&observed) {
        bail!("interrupted journal artifact bytes are not the exact expected prefix")
    }

    if observed.len() != expected_bytes.len() {
        if mode != staging_mode {
            file.set_permissions(fs::Permissions::from_mode(staging_mode))
                .context("restore journal artifact temporary staging mode")?;
            file.sync_all()
                .context("fsync journal artifact temporary staging mode")?;
        }
        drop(file);
        file = OpenOptions::new()
            .read(true)
            .write(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .open(path)
            .context("reopen journal artifact temporary for prefix completion")?;
        metadata = file
            .metadata()
            .context("reinspect journal artifact temporary before prefix completion")?;
        if metadata.dev() != device
            || metadata.ino() != inode
            || metadata.uid() != expected_uid
            || metadata.mode() & 0o7777 != staging_mode
            || metadata.nlink() != 1
        {
            bail!("journal artifact temporary changed before prefix completion")
        }
        file.seek(SeekFrom::End(0))
            .context("seek journal artifact temporary to its exact prefix end")?;
        file.write_all(&expected_bytes[observed.len()..])
            .context("complete journal artifact exact suffix")?;
    }
    file.sync_all()
        .context("fsync completed journal artifact temporary")?;
    file.set_permissions(fs::Permissions::from_mode(final_mode))
        .context("freeze completed journal artifact temporary mode")?;
    file.sync_all()
        .context("fsync frozen journal artifact temporary mode")?;
    drop(file);

    let reopened = read_recoverable_optional_file(path, expected_uid, &[final_mode])?
        .context("completed journal artifact temporary disappeared")?;
    if reopened.link_count != 1 || reopened.bytes != expected_bytes {
        bail!("completed journal artifact temporary failed byte or identity verification")
    }
    sync_directory(
        path.parent()
            .context("journal artifact temporary has no parent")?,
    )
}

/// Publish immutable bytes by completing a deterministic exact-prefix temporary, then creating
/// the final name with hard-link no-clobber semantics. A restart can resume before the link or
/// remove only the exact same-inode temporary after the link; alternate bytes are never replaced.
fn publish_immutable_no_clobber(
    path: &Path,
    bytes: &[u8],
    mode: u32,
    expected_uid: u32,
) -> Result<()> {
    let parent = path
        .parent()
        .context("immutable journal artifact has no parent")?;
    let temporary = immutable_publish_temporary_path(path)?;
    if let Some(existing) = read_recoverable_optional_file(path, expected_uid, &[mode])? {
        if existing.bytes != bytes {
            bail!("immutable journal artifact final bytes already differ")
        }
        match read_recoverable_optional_file(&temporary, expected_uid, &[mode])? {
            None if existing.link_count == 1 => return Ok(()),
            Some(pending)
                if existing.link_count == 2
                    && pending.link_count == 2
                    && existing.device == pending.device
                    && existing.inode == pending.inode
                    && pending.bytes == bytes =>
            {
                fs::remove_file(&temporary)
                    .context("remove exact completed immutable journal temporary")?;
                sync_directory(parent)?;
                let reopened = read_recoverable_optional_file(path, expected_uid, &[mode])?
                    .context("immutable journal artifact disappeared after recovery")?;
                if reopened.link_count != 1 || reopened.bytes != bytes {
                    bail!("immutable journal artifact failed post-recovery verification")
                }
                return Ok(());
            }
            _ => {
                bail!("immutable journal artifact final and temporary identities are ambiguous")
            }
        }
    }

    complete_durable_exact_prefix(&temporary, bytes, mode, expected_uid)?;
    fs::hard_link(&temporary, path)
        .context("publish immutable journal artifact with no-clobber link")?;
    sync_directory(parent)?;
    let pending = read_recoverable_optional_file(&temporary, expected_uid, &[mode])?
        .context("immutable journal artifact temporary disappeared after link")?;
    let published = read_recoverable_optional_file(path, expected_uid, &[mode])?
        .context("immutable journal artifact final name is absent after link")?;
    if pending.link_count != 2
        || published.link_count != 2
        || pending.device != published.device
        || pending.inode != published.inode
        || pending.bytes != bytes
        || published.bytes != bytes
    {
        bail!("immutable journal artifact no-clobber link identity differs")
    }
    fs::remove_file(&temporary).context("remove immutable journal artifact temporary")?;
    sync_directory(parent)?;
    let reopened = read_recoverable_optional_file(path, expected_uid, &[mode])?
        .context("immutable journal artifact disappeared after temporary removal")?;
    if reopened.link_count != 1 || reopened.bytes != bytes {
        bail!("immutable journal artifact failed final reopen verification")
    }
    Ok(())
}

fn read_exact_optional_file(
    path: &Path,
    expected_uid: u32,
    expected_mode: u32,
) -> Result<Option<Vec<u8>>> {
    let Some(observed) = read_recoverable_optional_file(path, expected_uid, &[expected_mode])?
    else {
        return Ok(None);
    };
    let temporary = immutable_publish_temporary_path(path)?;
    match read_recoverable_optional_file(&temporary, expected_uid, &[expected_mode])? {
        None if observed.link_count == 1 => Ok(Some(observed.bytes)),
        Some(pending)
            if observed.link_count == 2
                && pending.link_count == 2
                && observed.device == pending.device
                && observed.inode == pending.inode
                && observed.bytes == pending.bytes =>
        {
            fs::remove_file(&temporary)
                .context("remove exact linked immutable journal temporary during read")?;
            sync_directory(
                path.parent()
                    .context("durable journal artifact has no parent")?,
            )?;
            let reopened = read_recoverable_optional_file(path, expected_uid, &[expected_mode])?
                .context("durable journal artifact disappeared during linked-temp recovery")?;
            if reopened.link_count != 1 || reopened.bytes != observed.bytes {
                bail!("durable journal artifact changed during linked-temp recovery")
            }
            Ok(Some(reopened.bytes))
        }
        _ => bail!("durable journal artifact has an ambiguous temporary or link count"),
    }
}

fn read_exact_required_file(path: &Path, expected_uid: u32, expected_mode: u32) -> Result<Vec<u8>> {
    read_exact_optional_file(path, expected_uid, expected_mode)?
        .context("required journal artifact is absent")
}

fn sync_directory(path: &Path) -> Result<()> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC | libc::O_DIRECTORY)
        .open(path)
        .context("open no-follow directory for fsync")?;
    file.sync_all().context("fsync directory")
}

fn is_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn require_component(value: &str, label: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || value == "."
        || value == ".."
        || value.contains(['/', '\0', '\n', '\r'])
    {
        bail!("{label} is not one exact path component")
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract::{JournalEvent, JournalEventKind};
    use substrate_common::macos_retirement_v2::{HostRetirementStateV2, HostTargetRoleV2};

    fn digest(byte: u8) -> String {
        format!("{byte:02x}").repeat(32)
    }

    fn claim() -> JournalClaimIdentity {
        JournalClaimIdentity {
            request_digest: digest(1),
            authority_bytes_sha256: digest(9),
            successor_capsule_sha256: digest(2),
            accepted_peer_attestation_sha256: digest(3),
            accepted_peer_identity_sha256: digest(4),
        }
    }

    fn event(kind: JournalEventKind) -> JournalEvent {
        let is_effect = matches!(
            kind,
            JournalEventKind::EffectPrepared
                | JournalEventKind::EffectInvoked
                | JournalEventKind::EffectObserved
        );
        JournalEvent {
            kind,
            host_state: if kind == JournalEventKind::FinalizerAccepted {
                HostRetirementStateV2::FinalizerAccepted
            } else {
                HostRetirementStateV2::Removing
            },
            effect_ordinal: is_effect.then_some(1),
            effect_role: is_effect.then_some(HostTargetRoleV2::ProtectedWrapper),
            effect_identity_sha256: is_effect.then(|| digest(7)),
            effect_invocation_attempt: match kind {
                JournalEventKind::EffectPrepared => Some(0),
                JournalEventKind::EffectInvoked | JournalEventKind::EffectObserved => Some(1),
                _ => None,
            },
            observation_sha256: (kind == JournalEventKind::EffectObserved).then(|| digest(8)),
            response_sha256: None,
            preserving_classification: None,
        }
    }

    fn create_exact_test_directory(path: &Path) {
        let mut builder = fs::DirBuilder::new();
        builder.mode(0o700);
        builder.create(path).unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o700)).unwrap();
    }

    fn create_exact_test_lock(path: &Path) {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create_new(true)
            .mode(0o600)
            .open(path)
            .unwrap()
            .sync_all()
            .unwrap();
        fs::set_permissions(path, fs::Permissions::from_mode(0o600)).unwrap();
    }

    #[test]
    fn existing_open_initializes_only_the_empty_precreated_activation_root() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);

        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
        let lock = root.join("journal-root.lock");
        let metadata = fs::symlink_metadata(&lock).unwrap();
        assert!(metadata.file_type().is_file());
        assert_eq!(metadata.uid(), uid);
        assert_eq!(metadata.mode() & 0o7777, 0o600);
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.len(), 0);

        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
    }

    #[test]
    fn existing_open_recovers_only_an_empty_scope_before_its_lock_creation() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let scope = root.join("scope");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
        create_exact_test_directory(&scope);

        let journal = LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .expect("empty interrupted scope must be recoverable");
        assert!(journal.head().unwrap().is_none());
        let metadata = fs::symlink_metadata(scope.join("journal.lock")).unwrap();
        assert!(metadata.file_type().is_file());
        assert_eq!(metadata.uid(), uid);
        assert_eq!(metadata.mode() & 0o7777, 0o600);
        assert_eq!(metadata.nlink(), 1);
        assert_eq!(metadata.len(), 0);
    }

    #[test]
    fn exact_lock_create_boundaries_rejoin_without_replacing_either_membrane() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let scope = root.join("scope");
        let root_lock = root.join("journal-root.lock");
        let scope_lock = scope.join("journal.lock");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);
        create_exact_test_lock(&root_lock);
        let root_identity = fs::symlink_metadata(&root_lock).unwrap().ino();
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
        assert_eq!(
            fs::symlink_metadata(&root_lock).unwrap().ino(),
            root_identity
        );

        create_exact_test_directory(&scope);
        create_exact_test_lock(&scope_lock);
        let scope_identity = fs::symlink_metadata(&scope_lock).unwrap().ino();
        let journal = LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .expect("exact scope-lock create boundary must rejoin");
        assert!(journal.head().unwrap().is_none());
        assert_eq!(
            fs::symlink_metadata(&root_lock).unwrap().ino(),
            root_identity
        );
        assert_eq!(
            fs::symlink_metadata(&scope_lock).unwrap().ino(),
            scope_identity
        );
    }

    #[test]
    fn missing_root_lock_with_any_sibling_is_preserved_and_rejected() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);
        let alternate = root.join("alternate");
        fs::write(&alternate, b"preserve").unwrap();

        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid).is_err());
        assert_eq!(fs::read(&alternate).unwrap(), b"preserve");
        assert!(!root.join("journal-root.lock").exists());
    }

    #[test]
    fn missing_scope_lock_with_any_sibling_is_preserved_and_rejected() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let scope = root.join("scope");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
        create_exact_test_directory(&scope);
        let alternate = scope.join("generation-00000000000000000001.json");
        fs::write(&alternate, b"partial-or-alternate").unwrap();

        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid).is_err());
        assert_eq!(fs::read(&alternate).unwrap(), b"partial-or-alternate");
        assert!(!scope.join("journal.lock").exists());
    }

    #[test]
    fn nonempty_or_substituted_lock_membranes_are_never_adopted() {
        let temporary = tempfile::tempdir().unwrap();
        let root = temporary.path().join("journal");
        let uid = unsafe { libc::geteuid() };
        create_exact_test_directory(&root);
        let root_lock = root.join("journal-root.lock");
        fs::write(&root_lock, b"not-an-empty-lock").unwrap();
        fs::set_permissions(&root_lock, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid).is_err());
        assert_eq!(fs::read(&root_lock).unwrap(), b"not-an-empty-lock");

        fs::remove_file(&root_lock).unwrap();
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid)
            .unwrap()
            .is_none());
        let outside = temporary.path().join("outside");
        create_exact_test_directory(&outside);
        std::os::unix::fs::symlink(&outside, root.join("scope")).unwrap();
        assert!(LockedJournal::open_existing_fixed(&root, "scope", uid).is_err());
        assert!(root
            .join("scope")
            .symlink_metadata()
            .unwrap()
            .file_type()
            .is_symlink());
    }

    #[test]
    fn journal_generation_bound_matches_the_maximum_closed_effect_plan() {
        assert_eq!(MAX_EFFECT_INVOCATION_ATTEMPTS_V2, 2);
        assert_eq!(MAX_JOURNAL_GENERATIONS, 4 * 4096 + 4);
        let record = JournalGeneration {
            schema_owner: JOURNAL_OWNER.to_string(),
            schema_version: JOURNAL_VERSION,
            scope_id: "scope".to_string(),
            generation: MAX_JOURNAL_GENERATIONS + 1,
            predecessor_head_sha256: digest(0),
            request_digest: digest(1),
            authority_bytes_sha256: digest(2),
            successor_capsule_sha256: digest(3),
            accepted_peer_attestation_sha256: digest(4),
            accepted_peer_identity_sha256: digest(5),
            event: event(JournalEventKind::FinalizerAccepted),
        };
        assert!(validate_generation(&record, MAX_JOURNAL_GENERATIONS + 1).is_err());
    }

    #[test]
    fn maximum_target_plan_with_every_retry_exactly_fits_the_generation_bound() {
        let mut generation = 1_u64;
        let mut previous = event(JournalEventKind::FinalizerAccepted);
        validate_event_transition(None, &previous).unwrap();

        for ordinal in 1..=MAC_R3_MAX_HOST_TARGETS_V2 {
            for (kind, attempt) in [
                (JournalEventKind::EffectPrepared, 0),
                (JournalEventKind::EffectInvoked, 1),
                (JournalEventKind::EffectInvoked, 2),
                (JournalEventKind::EffectObserved, 2),
            ] {
                let mut next = event(kind);
                next.effect_ordinal = Some(u16::try_from(ordinal).unwrap());
                next.effect_invocation_attempt = Some(attempt);
                validate_event_transition(Some(&previous), &next).unwrap();
                generation += 1;
                previous = next;
            }
        }

        for next in [
            JournalEvent {
                kind: JournalEventKind::EffectsComplete,
                host_state: HostRetirementStateV2::EffectsComplete,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(digest(8)),
                response_sha256: None,
                preserving_classification: None,
            },
            JournalEvent {
                kind: JournalEventKind::TerminalAcknowledgementBound,
                host_state: HostRetirementStateV2::TerminalAcknowledgementBound,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(digest(8)),
                response_sha256: Some(digest(9)),
                preserving_classification: None,
            },
            JournalEvent {
                kind: JournalEventKind::Complete,
                host_state: HostRetirementStateV2::Complete,
                effect_ordinal: None,
                effect_role: None,
                effect_identity_sha256: None,
                effect_invocation_attempt: None,
                observation_sha256: Some(digest(8)),
                response_sha256: Some(digest(9)),
                preserving_classification: None,
            },
        ] {
            validate_event_transition(Some(&previous), &next).unwrap();
            generation += 1;
            previous = next;
        }
        assert_eq!(generation, MAX_JOURNAL_GENERATIONS);
    }

    #[test]
    fn effect_invocation_attempts_are_bounded_monotonic_and_immutable() {
        let prepared = event(JournalEventKind::EffectPrepared);
        let first = event(JournalEventKind::EffectInvoked);
        validate_event_transition(Some(&prepared), &first).unwrap();

        let mut second = first.clone();
        second.effect_invocation_attempt = Some(2);
        validate_event_transition(Some(&first), &second).unwrap();

        let mut observed = event(JournalEventKind::EffectObserved);
        observed.effect_invocation_attempt = Some(2);
        validate_event_transition(Some(&second), &observed).unwrap();

        let mut repeated = second.clone();
        repeated.effect_invocation_attempt = Some(2);
        assert!(validate_event_transition(Some(&second), &repeated).is_err());

        let mut third = second.clone();
        third.effect_invocation_attempt = Some(3);
        assert!(validate_event_shape(&third).is_err());

        let mut drifted_observation = observed;
        drifted_observation.effect_invocation_attempt = Some(1);
        assert!(validate_event_transition(Some(&second), &drifted_observation).is_err());
    }

    #[test]
    fn durable_hash_chain_and_generation_cas_reject_forks() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let second = journal
            .append(
                Some(&first),
                &claim(),
                event(JournalEventKind::EffectPrepared),
            )
            .unwrap();
        assert_eq!(journal.head().unwrap(), Some(second));
        assert!(journal
            .append(
                Some(&first),
                &claim(),
                event(JournalEventKind::EffectInvoked),
            )
            .is_err());
    }

    fn simulate_head_commit_interrupted_before_rename(
        scope_dir: &Path,
        committed_head: Option<&JournalHead>,
        pending_head: &JournalHead,
        suffix: &str,
    ) -> PathBuf {
        let head_path = scope_dir.join("HEAD");
        let pending_path = scope_dir.join(format!(".HEAD.tmp.{suffix}"));
        fs::rename(&head_path, &pending_path).unwrap();
        if let Some(committed_head) = committed_head {
            complete_durable_exact_prefix(
                &head_path,
                format!(
                    "{:020} {}\n",
                    committed_head.generation, committed_head.sha256
                )
                .as_bytes(),
                0o600,
                unsafe { libc::geteuid() },
            )
            .unwrap();
        }
        assert_eq!(
            read_exact_required_file(&pending_path, unsafe { libc::geteuid() }, 0o600).unwrap(),
            format!("{:020} {}\n", pending_head.generation, pending_head.sha256).as_bytes()
        );
        pending_path
    }

    #[test]
    fn exact_stale_head_temporary_recovers_the_first_generation() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let pending =
            simulate_head_commit_interrupted_before_rename(&journal.scope_dir, None, &first, "101");
        drop(journal);

        let recovered = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        assert_eq!(recovered.head().unwrap(), Some(first));
        assert!(!pending.exists());
    }

    #[test]
    fn exact_prefix_head_temporary_resumes_before_recovery_rename() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let pending =
            simulate_head_commit_interrupted_before_rename(&journal.scope_dir, None, &first, "106");
        let expected = fs::read(&pending).unwrap();
        fs::write(&pending, &expected[..17]).unwrap();
        drop(journal);

        let recovered = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        assert_eq!(recovered.head().unwrap(), Some(first));
        assert!(!pending.exists());
    }

    #[test]
    fn exact_stale_head_temporary_recovers_the_unique_successor() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let second = journal
            .append(
                Some(&first),
                &claim(),
                event(JournalEventKind::EffectPrepared),
            )
            .unwrap();
        let pending = simulate_head_commit_interrupted_before_rename(
            &journal.scope_dir,
            Some(&first),
            &second,
            "102",
        );
        drop(journal);

        let recovered = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        assert_eq!(recovered.head().unwrap(), Some(second.clone()));
        assert!(!pending.exists());

        let third = recovered
            .append(
                Some(&second),
                &claim(),
                event(JournalEventKind::EffectInvoked),
            )
            .unwrap();
        assert_eq!(third.generation, 3);
    }

    #[test]
    fn stale_head_temporary_mismatch_fails_closed_without_cleanup() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let pending =
            simulate_head_commit_interrupted_before_rename(&journal.scope_dir, None, &first, "103");
        fs::write(&pending, format!("{:020} {}\n", 1, digest(15))).unwrap();
        drop(journal);

        let recovered = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        assert!(recovered.head().is_err());
        assert!(pending.exists());
        assert!(!recovered.scope_dir.join("HEAD").exists());
    }

    #[test]
    fn multiple_stale_head_temporaries_fail_closed_without_cleanup() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let first = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let first_pending =
            simulate_head_commit_interrupted_before_rename(&journal.scope_dir, None, &first, "104");
        let second_pending = journal.scope_dir.join(".HEAD.tmp.105");
        complete_durable_exact_prefix(
            &second_pending,
            &fs::read(&first_pending).unwrap(),
            0o600,
            uid,
        )
        .unwrap();
        drop(journal);

        let recovered = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        assert!(recovered.head().is_err());
        assert!(first_pending.exists());
        assert!(second_pending.exists());
        assert!(!recovered.scope_dir.join("HEAD").exists());
    }

    fn create_interrupted_publish_prefix(path: &Path, bytes: &[u8], final_mode: u32) {
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_CLOEXEC)
            .mode(final_mode | 0o200)
            .open(path)
            .unwrap();
        file.write_all(bytes).unwrap();
        file.sync_all().unwrap();
    }

    #[test]
    fn immutable_publish_resumes_create_write_fsync_and_link_boundaries() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let expected = b"canonical immutable journal artifact bytes";

        for (basename, prefix_length) in [("after-create", 0), ("after-write", 11)] {
            let path = temp.path().join(basename);
            let pending = immutable_publish_temporary_path(&path).unwrap();
            create_interrupted_publish_prefix(&pending, &expected[..prefix_length], 0o400);
            publish_immutable_no_clobber(&path, expected, 0o400, uid).unwrap();
            assert_eq!(
                read_exact_required_file(&path, uid, 0o400).unwrap(),
                expected
            );
            assert!(!pending.exists());
        }

        let after_file_fsync = temp.path().join("after-file-fsync");
        let after_file_fsync_pending = immutable_publish_temporary_path(&after_file_fsync).unwrap();
        complete_durable_exact_prefix(&after_file_fsync_pending, expected, 0o400, uid).unwrap();
        publish_immutable_no_clobber(&after_file_fsync, expected, 0o400, uid).unwrap();
        assert_eq!(
            read_exact_required_file(&after_file_fsync, uid, 0o400).unwrap(),
            expected
        );
        assert!(!after_file_fsync_pending.exists());

        let after_link = temp.path().join("after-no-clobber-link");
        let after_link_pending = immutable_publish_temporary_path(&after_link).unwrap();
        complete_durable_exact_prefix(&after_link_pending, expected, 0o400, uid).unwrap();
        fs::hard_link(&after_link_pending, &after_link).unwrap();
        sync_directory(temp.path()).unwrap();
        publish_immutable_no_clobber(&after_link, expected, 0o400, uid).unwrap();
        assert_eq!(
            read_exact_required_file(&after_link, uid, 0o400).unwrap(),
            expected
        );
        assert!(!after_link_pending.exists());
    }

    #[test]
    fn generation_and_side_artifact_rejoin_resume_the_shared_prefix_publisher() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let claim = claim();
        let first = journal
            .append(None, &claim, event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        let prepared_event = event(JournalEventKind::EffectPrepared);
        let expected_record = JournalGeneration {
            schema_owner: JOURNAL_OWNER.to_string(),
            schema_version: JOURNAL_VERSION,
            scope_id: "scope".to_string(),
            generation: 2,
            predecessor_head_sha256: first.sha256.clone(),
            request_digest: claim.request_digest.clone(),
            authority_bytes_sha256: claim.authority_bytes_sha256.clone(),
            successor_capsule_sha256: claim.successor_capsule_sha256.clone(),
            accepted_peer_attestation_sha256: claim.accepted_peer_attestation_sha256.clone(),
            accepted_peer_identity_sha256: claim.accepted_peer_identity_sha256.clone(),
            event: prepared_event.clone(),
        };
        let expected_generation_bytes = canonical_bytes_v2(&expected_record).unwrap();
        let generation_path = journal.generation_path(2);
        let generation_pending = immutable_publish_temporary_path(&generation_path).unwrap();
        create_interrupted_publish_prefix(
            &generation_pending,
            &expected_generation_bytes[..expected_generation_bytes.len() / 2],
            0o400,
        );

        let second = journal
            .append(Some(&first), &claim, prepared_event)
            .unwrap();
        assert_eq!(second.record, expected_record);
        assert!(!generation_pending.exists());

        let authority_bytes = b"exact authority request bytes";
        let authority_path = journal.scope_dir.join("authority.request");
        let authority_pending = immutable_publish_temporary_path(&authority_path).unwrap();
        create_interrupted_publish_prefix(&authority_pending, &authority_bytes[..9], 0o400);
        journal
            .persist_acceptance_artifact(
                "authority.request",
                authority_bytes,
                &sha256_hex_v2(authority_bytes),
            )
            .unwrap();
        assert_eq!(
            journal
                .acceptance_artifact("authority.request")
                .unwrap()
                .unwrap(),
            authority_bytes
        );
        assert!(!authority_pending.exists());
    }

    #[test]
    fn immutable_read_recovers_only_the_exact_same_inode_link_residue() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let path = temp.path().join("artifact");
        let pending = immutable_publish_temporary_path(&path).unwrap();
        let expected = b"linked immutable bytes";
        complete_durable_exact_prefix(&pending, expected, 0o400, uid).unwrap();
        fs::hard_link(&pending, &path).unwrap();

        assert_eq!(
            read_exact_required_file(&path, uid, 0o400).unwrap(),
            expected
        );
        assert!(!pending.exists());
        assert_eq!(fs::metadata(&path).unwrap().nlink(), 1);
    }

    #[test]
    fn immutable_publish_preserves_and_rejects_an_alternate_temporary() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let path = temp.path().join("artifact");
        let pending = immutable_publish_temporary_path(&path).unwrap();
        create_interrupted_publish_prefix(&pending, b"alternate", 0o400);

        assert!(publish_immutable_no_clobber(&path, b"expected", 0o400, uid).is_err());
        assert!(!path.exists());
        let preserved = read_recoverable_optional_file(&pending, uid, &[0o400, 0o600])
            .unwrap()
            .unwrap();
        assert_eq!(preserved.bytes, b"alternate");
        assert_eq!(preserved.link_count, 1);
    }

    #[test]
    fn journal_owned_side_artifacts_preserve_the_authenticated_head_cache() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let root = temp.path().join("journal");
        let journal = LockedJournal::open_fixed(&root, "scope", uid).unwrap();
        let mut head = journal
            .append(None, &claim(), event(JournalEventKind::FinalizerAccepted))
            .unwrap();
        assert_eq!(journal.full_chain_load_count.get(), 1);

        for (basename, bytes) in [
            ("authority.request", b"authority".as_slice()),
            ("peer.attestation", b"peer".as_slice()),
        ] {
            journal
                .persist_acceptance_artifact(basename, bytes, &sha256_hex_v2(bytes))
                .unwrap();
        }

        for ordinal in 1..=64_u16 {
            let mut prepared = event(JournalEventKind::EffectPrepared);
            prepared.effect_ordinal = Some(ordinal);
            prepared.effect_identity_sha256 = Some(digest(7));
            head = journal.append(Some(&head), &claim(), prepared).unwrap();

            let mut invoked = event(JournalEventKind::EffectInvoked);
            invoked.effect_ordinal = Some(ordinal);
            invoked.effect_identity_sha256 = Some(digest(7));
            head = journal.append(Some(&head), &claim(), invoked).unwrap();

            let observation = format!("observation-{ordinal}").into_bytes();
            let observation_sha256 = sha256_hex_v2(&observation);
            journal
                .persist_effect_observation_artifact(ordinal, &observation, &observation_sha256)
                .unwrap();

            let mut observed = event(JournalEventKind::EffectObserved);
            observed.effect_ordinal = Some(ordinal);
            observed.effect_identity_sha256 = Some(digest(7));
            observed.observation_sha256 = Some(observation_sha256);
            head = journal.append(Some(&head), &claim(), observed).unwrap();
        }

        assert_eq!(head.generation, 1 + 3 * 64);
        assert_eq!(journal.full_chain_load_count.get(), 1);
    }

    #[test]
    fn terminal_response_is_immutable() {
        let temp = tempfile::tempdir().unwrap();
        let uid = unsafe { libc::geteuid() };
        let journal =
            LockedJournal::open_fixed(&temp.path().join("journal"), "scope", uid).unwrap();
        journal
            .persist_terminal_response(&digest(1), b"one")
            .unwrap();
        assert_eq!(journal.terminal_response().unwrap().unwrap(), b"one");
        assert!(journal
            .persist_terminal_response(&digest(1), b"two")
            .is_err());
    }

    #[test]
    fn effect_transition_rejects_target_identity_substitution() {
        let prepared = event(JournalEventKind::EffectPrepared);
        let mut invoked = event(JournalEventKind::EffectInvoked);
        invoked.effect_identity_sha256 = Some(digest(6));
        assert!(validate_event_transition(Some(&prepared), &invoked).is_err());

        let mut missing = event(JournalEventKind::EffectObserved);
        missing.effect_identity_sha256 = None;
        assert!(validate_event_shape(&missing).is_err());
    }
}
