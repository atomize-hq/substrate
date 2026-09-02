use anyhow::Context as _;
use hmac::{Hmac, Mac};
use rand::RngCore as _;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use sha2::{Digest as _, Sha256};
use std::{
    collections::{HashMap, HashSet},
    ffi::{CStr, CString},
    fs::File,
    io::{Read as _, Write as _},
    mem::MaybeUninit,
    os::fd::{AsRawFd, FromRawFd, RawFd},
    os::unix::ffi::OsStrExt as _,
    path::{Component, Path, PathBuf},
    sync::{Arc, Mutex},
    task::{Context as TaskContext, Poll},
    time::Duration,
};
use tokio::sync::{
    mpsc::{self, Receiver, Sender},
    watch,
};
use transport_api_types::{
    ExecuteStreamFrame, MemberTurnSubmitRequestV1, RuntimeFrameIdentityV1,
    RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
};

const TURN_KEY_DOMAIN: &[u8] = b"substrate.world-service.e2-member-turn-key.v1";
const REQUEST_DIGEST_DOMAIN: &[u8] = b"substrate.world-service.e2-member-turn-request.v1";
const REGISTRY_DIRECTORY: &str = "member-turn-join-v1";
const FORMAT_FILE: &str = "format-v1.json";
const KEY_FILE: &str = "digest-key-v1.json";
const REGISTRY_LOCK_FILE: &str = "registry.lock";
const RECORDS_DIRECTORY: &str = "records";
const RECORD_FILE: &str = "record-v1.json";
const FRAMES_DIRECTORY: &str = "frames";
const LEADER_LOCK_FILE: &str = "leader.lock";
const PRIVATE_DIRECTORY_MODE: libc::mode_t = 0o700;
const PRIVATE_FILE_MODE: libc::mode_t = 0o600;
const MAX_DURABLE_RECORDS: usize = 131_072;
const MAX_LIVE_SUBSCRIBERS_PER_RECORD: usize = 1_024;
const DURABLE_STATE_POLL_INTERVAL: Duration = Duration::from_millis(25);

type HmacSha256 = Hmac<Sha256>;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnStableKeyV1 {
    schema_version: u32,
    authority_store_id: String,
    orchestration_session_id: String,
    request_id: String,
    subject_kind: String,
    retained_participant_id: String,
    active_run_id: String,
}

impl MemberTurnStableKeyV1 {
    fn from_request(request: &MemberTurnSubmitRequestV1) -> anyhow::Result<Self> {
        request.validate().map_err(anyhow::Error::msg)?;
        let carrier = request.policy_snapshot_carrier.as_ref().ok_or_else(|| {
            anyhow::anyhow!("E2 member-turn reservation requires policy_snapshot_carrier")
        })?;
        let acceptance = request.acceptance_context.as_ref().ok_or_else(|| {
            anyhow::anyhow!("E2 member-turn reservation requires acceptance_context")
        })?;
        if carrier.subject.message_id.as_ref() != acceptance.message_id.as_ref() {
            anyhow::bail!("E2 member-turn carrier/acceptance message identity mismatch");
        }
        Ok(Self {
            schema_version: 1,
            authority_store_id: carrier.immutable_worker_cap_ref.authority_store_id.clone(),
            orchestration_session_id: request.orchestration_session_id.clone(),
            request_id: request.run_id.clone(),
            subject_kind: "retained_worker_turn".to_string(),
            retained_participant_id: carrier.subject.retained_participant_id.clone(),
            active_run_id: carrier.subject.active_run_id.clone(),
        })
    }
}

fn canonical_request_bytes(request: &MemberTurnSubmitRequestV1) -> anyhow::Result<Vec<u8>> {
    request.validate().map_err(anyhow::Error::msg)?;
    let value = serde_json::to_value(request)?;
    serde_json::to_vec(&recursively_sorted_json(value)).map_err(Into::into)
}

fn recursively_sorted_json(value: Value) -> Value {
    match value {
        Value::Array(values) => {
            Value::Array(values.into_iter().map(recursively_sorted_json).collect())
        }
        Value::Object(values) => {
            let mut entries = values.into_iter().collect::<Vec<_>>();
            entries.sort_by(|left, right| left.0.cmp(&right.0));
            Value::Object(
                entries
                    .into_iter()
                    .map(|(key, value)| (key, recursively_sorted_json(value)))
                    .collect(),
            )
        }
        scalar => scalar,
    }
}

fn domain_separated_hmac(key: &[u8], domain: &[u8], message: &[u8]) -> anyhow::Result<[u8; 32]> {
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| anyhow::anyhow!("member-turn HMAC key length is invalid"))?;
    mac.update(&(domain.len() as u64).to_be_bytes());
    mac.update(domain);
    mac.update(&(message.len() as u64).to_be_bytes());
    mac.update(message);
    Ok(mac.finalize().into_bytes().into())
}

fn lowercase_hex(bytes: &[u8]) -> String {
    let mut value = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        use std::fmt::Write as _;
        write!(&mut value, "{byte:02x}").expect("write to String");
    }
    value
}

fn stable_key_id(key: &[u8], stable_key: &MemberTurnStableKeyV1) -> anyhow::Result<String> {
    let bytes = serde_json::to_vec(stable_key)?;
    Ok(lowercase_hex(&domain_separated_hmac(
        key,
        TURN_KEY_DOMAIN,
        &bytes,
    )?))
}

fn request_digest(key: &[u8], request: &MemberTurnSubmitRequestV1) -> anyhow::Result<String> {
    Ok(lowercase_hex(&domain_separated_hmac(
        key,
        REQUEST_DIGEST_DOMAIN,
        &canonical_request_bytes(request)?,
    )?))
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnJoinFormatV1 {
    schema_version: u32,
    format: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnDigestKeyEnvelopeV1 {
    schema_version: u32,
    key_id: String,
    secret_key_base64: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(tag = "state", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum MemberTurnJoinStateV1 {
    Reserved,
    LaunchingPreCall {
        leader_instance_id: String,
        leader_token_digest: String,
    },
    LaunchingCallEntered {
        leader_instance_id: String,
        leader_token_digest: String,
    },
    Started,
    Completed {
        terminal_frame_sequence: u64,
    },
    FailedBeforeLaunch {
        error_code: String,
        http_status: u16,
    },
    LaunchIndeterminate {
        reason_code: String,
    },
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
struct MemberTurnJoinRecordV1 {
    schema_version: u32,
    record_revision: u64,
    record_id: String,
    stable_key: MemberTurnStableKeyV1,
    request_digest_hex: String,
    span_id: String,
    stream_id: String,
    acceptance_record_id: String,
    message_id: String,
    state: MemberTurnJoinStateV1,
    last_frame_sequence: u64,
    retained_bytes: u64,
    frame_chain_sha256: String,
}

impl MemberTurnJoinRecordV1 {
    fn validate(&self, expected_record_id: &str) -> anyhow::Result<()> {
        if self.schema_version != 1
            || self.record_revision == 0
            || self.record_id != expected_record_id
            || self.record_id.len() != 64
            || !is_lower_hex(&self.record_id)
            || self.request_digest_hex.len() != 64
            || !is_lower_hex(&self.request_digest_hex)
            || !self.span_id.starts_with("spn_")
            || !self.stream_id.starts_with("rts_")
            || self.acceptance_record_id.trim().is_empty()
            || self.message_id.trim().is_empty()
            || self.frame_chain_sha256.len() != 64
            || !is_lower_hex(&self.frame_chain_sha256)
        {
            anyhow::bail!("member-turn durable record is invalid");
        }
        if self.last_frame_sequence == 0 {
            if self.retained_bytes != 0 || self.frame_chain_sha256 != "0".repeat(64) {
                anyhow::bail!("empty member-turn frame journal has invalid commitment");
            }
        } else if matches!(
            self.state,
            MemberTurnJoinStateV1::Reserved
                | MemberTurnJoinStateV1::LaunchingPreCall { .. }
                | MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                | MemberTurnJoinStateV1::FailedBeforeLaunch { .. }
        ) {
            anyhow::bail!("pre-Start member-turn state contains durable frames");
        }
        match self.state {
            MemberTurnJoinStateV1::Started if self.last_frame_sequence == 0 => {
                anyhow::bail!("Started member-turn record omitted its real Start frame")
            }
            MemberTurnJoinStateV1::Completed {
                terminal_frame_sequence,
            } if terminal_frame_sequence == 0
                || terminal_frame_sequence != self.last_frame_sequence =>
            {
                anyhow::bail!("Completed member-turn record has invalid terminal reference")
            }
            _ => {}
        }
        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub(crate) enum MemberTurnJoinError {
    #[error("member_turn_request_conflict_v1")]
    RequestConflict,
    #[error("member_turn_launch_indeterminate_v1")]
    LaunchIndeterminate,
    #[error("unsupported_legacy_state")]
    UnsupportedLegacyState,
    #[error("member_turn_join_capacity_exhausted_v1")]
    CapacityExhausted,
    #[error("{error_code}")]
    FailedBeforeLaunch {
        error_code: String,
        http_status: u16,
    },
    #[error("member-turn durable authority is unsafe: {0}")]
    Unsafe(String),
}

#[derive(Clone, Default)]
pub(crate) struct MemberTurnAcceptanceNamespace {
    volatile_acceptance_record_ids: Arc<Mutex<HashSet<String>>>,
}

impl MemberTurnAcceptanceNamespace {
    pub(crate) fn lock(
        &self,
    ) -> Result<std::sync::MutexGuard<'_, HashSet<String>>, MemberTurnJoinError> {
        self.volatile_acceptance_record_ids.lock().map_err(|_| {
            MemberTurnJoinError::Unsafe(
                "member-turn acceptance namespace lock poisoned".to_string(),
            )
        })
    }
}

#[derive(Clone)]
pub(crate) struct MemberTurnJoinRegistry {
    inner: Arc<MemberTurnJoinRegistryInner>,
}

struct MemberTurnJoinRegistryInner {
    state_root_path: PathBuf,
    state_root_device: u64,
    state_root_inode: u64,
    registry_device: u64,
    registry_inode: u64,
    records_device: u64,
    records_inode: u64,
    registry: TrustedDirectory,
    records: TrustedDirectory,
    digest_key: [u8; 32],
    key_id: String,
    acceptance_namespace: MemberTurnAcceptanceNamespace,
    subscribers: Mutex<HashMap<String, Vec<Sender<()>>>>,
    state_notifiers: Mutex<HashMap<String, watch::Sender<u64>>>,
}

#[derive(Clone, Debug)]
pub(crate) struct MemberTurnJoinReservation {
    registry: MemberTurnJoinRegistry,
    record: MemberTurnJoinRecordV1,
}

pub(crate) struct MemberTurnJoinLeader {
    reservation: MemberTurnJoinReservation,
    leader_instance_id: String,
    leader_token_digest: String,
    _lock: TrustedOwnedFileLock,
}

pub(crate) struct MemberTurnJoinSubscription {
    registry: MemberTurnJoinRegistry,
    record_id: String,
    stream_id: String,
    next_frame_sequence: u64,
    receiver: Receiver<()>,
    poll_delay: Option<std::pin::Pin<Box<tokio::time::Sleep>>>,
    local_notifications_closed: bool,
    finished: bool,
}

impl futures_util::Stream for MemberTurnJoinSubscription {
    type Item = ExecuteStreamFrame;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        context: &mut TaskContext<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.as_mut().get_mut();
        loop {
            if this.finished {
                return Poll::Ready(None);
            }
            match this.registry.read_subscription_frame(
                &this.record_id,
                &this.stream_id,
                this.next_frame_sequence,
            ) {
                Ok(SubscriptionFrameRead::Frame(frame)) => {
                    this.next_frame_sequence = this.next_frame_sequence.saturating_add(1);
                    return Poll::Ready(Some(*frame));
                }
                Ok(SubscriptionFrameRead::Finished) | Err(_) => {
                    this.finished = true;
                    return Poll::Ready(None);
                }
                Ok(SubscriptionFrameRead::Pending) => {}
            }

            if !this.local_notifications_closed {
                match this.receiver.poll_recv(context) {
                    Poll::Ready(Some(())) => continue,
                    Poll::Ready(None) => this.local_notifications_closed = true,
                    Poll::Pending => {}
                }
            }
            let delay = this
                .poll_delay
                .get_or_insert_with(|| Box::pin(tokio::time::sleep(DURABLE_STATE_POLL_INTERVAL)));
            if std::future::Future::poll(delay.as_mut(), context).is_ready() {
                this.poll_delay = None;
                continue;
            }
            return Poll::Pending;
        }
    }
}

#[cfg(test)]
impl MemberTurnJoinSubscription {
    fn read_next_now(&mut self) -> anyhow::Result<Option<ExecuteStreamFrame>> {
        match self.registry.read_subscription_frame(
            &self.record_id,
            &self.stream_id,
            self.next_frame_sequence,
        )? {
            SubscriptionFrameRead::Frame(frame) => {
                self.next_frame_sequence += 1;
                Ok(Some(*frame))
            }
            SubscriptionFrameRead::Pending | SubscriptionFrameRead::Finished => Ok(None),
        }
    }
}

enum SubscriptionFrameRead {
    Frame(Box<ExecuteStreamFrame>),
    Pending,
    Finished,
}

impl MemberTurnJoinReservation {
    pub(crate) fn span_id(&self) -> &str {
        &self.record.span_id
    }

    pub(crate) fn stream_id(&self) -> &str {
        &self.record.stream_id
    }

    pub(crate) fn state(&self) -> &MemberTurnJoinStateV1 {
        &self.record.state
    }

    pub(crate) fn record_id(&self) -> &str {
        &self.record.record_id
    }
}

impl std::fmt::Debug for MemberTurnJoinRegistry {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter
            .debug_struct("MemberTurnJoinRegistry")
            .field("state_root_path", &self.inner.state_root_path)
            .field("key_id", &self.inner.key_id)
            .finish_non_exhaustive()
    }
}

impl MemberTurnJoinRegistry {
    pub(crate) fn open(state_root: &Path) -> anyhow::Result<Self> {
        let state_root_path = validate_absolute_path(state_root)?;
        let state_root_directory = TrustedDirectory::open_absolute(&state_root_path, false)?;
        let state_root_stat = state_root_directory.stat()?;
        validate_directory_stat(&state_root_stat, false)?;

        let registry = state_root_directory.open_or_create_directory(REGISTRY_DIRECTORY)?;
        registry.enforce_private_mode()?;
        state_root_directory.sync()?;
        let lock_file = registry.open_or_create_file(REGISTRY_LOCK_FILE)?;
        let lock = lock_file.lock_exclusive()?;
        reconcile_recognized_temps(&registry)?;

        let format = MemberTurnJoinFormatV1 {
            schema_version: 1,
            format: "substrate.world-service.member-turn-join.v1".to_string(),
        };
        create_or_exact_join_json(&registry, FORMAT_FILE, &format)?;
        let records = registry.open_or_create_directory(RECORDS_DIRECTORY)?;
        records.enforce_private_mode()?;
        reconcile_record_tree_temps(&records)?;
        let (key_id, digest_key) = open_or_initialize_digest_key(&registry, &records)?;
        validate_registry_layout(&registry)?;
        let registry_stat = registry.stat()?;
        validate_directory_stat(&registry_stat, true)?;
        let records_stat = records.stat()?;
        validate_directory_stat(&records_stat, true)?;
        registry.sync()?;
        state_root_directory.sync()?;
        drop(lock);

        let opened = Self {
            inner: Arc::new(MemberTurnJoinRegistryInner {
                state_root_path,
                state_root_device: state_root_stat.st_dev as u64,
                state_root_inode: state_root_stat.st_ino as u64,
                registry_device: registry_stat.st_dev as u64,
                registry_inode: registry_stat.st_ino as u64,
                records_device: records_stat.st_dev as u64,
                records_inode: records_stat.st_ino as u64,
                registry,
                records,
                digest_key,
                key_id,
                acceptance_namespace: MemberTurnAcceptanceNamespace::default(),
                subscribers: Mutex::new(HashMap::new()),
                state_notifiers: Mutex::new(HashMap::new()),
            }),
        };
        opened.recover_and_validate_existing_records()?;
        Ok(opened)
    }

    pub(crate) fn acceptance_namespace(&self) -> MemberTurnAcceptanceNamespace {
        self.inner.acceptance_namespace.clone()
    }

    pub(crate) fn join_existing(
        &self,
        request: &MemberTurnSubmitRequestV1,
    ) -> Result<Option<MemberTurnJoinReservation>, MemberTurnJoinError> {
        self.revalidate_root().map_err(join_unsafe)?;
        let stable_key = MemberTurnStableKeyV1::from_request(request).map_err(join_unsafe)?;
        let record_id = stable_key_id(&self.inner.digest_key, &stable_key).map_err(join_unsafe)?;
        let acceptance = request
            .acceptance_context
            .as_ref()
            .ok_or_else(|| join_unsafe("E2 request omitted acceptance_context"))?;
        let message_id = acceptance
            .message_id
            .as_ref()
            .ok_or_else(|| join_unsafe("E2 acceptance_context omitted message_id"))?;
        let acceptance_namespace = self.inner.acceptance_namespace.lock()?;
        if acceptance_namespace.contains(&acceptance.proposed_acceptance_record_id) {
            return Err(MemberTurnJoinError::RequestConflict);
        }

        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        let shard = self
            .inner
            .records
            .open_directory_if_present(&record_id[..2])
            .map_err(join_unsafe)?;
        let record_directory = match shard {
            Some(shard) => shard
                .open_directory_if_present(&record_id[2..])
                .map_err(join_unsafe)?,
            None => None,
        };
        let Some(record_directory) = record_directory else {
            if self
                .record_by_acceptance_locked(&acceptance.proposed_acceptance_record_id)
                .map_err(join_unsafe)?
                .is_some()
            {
                return Err(MemberTurnJoinError::RequestConflict);
            }
            self.revalidate_root().map_err(join_unsafe)?;
            return Ok(None);
        };

        reconcile_record_directory_temps(&record_directory).map_err(join_unsafe)?;
        let mut record = read_record(&record_directory, &record_id).map_err(join_unsafe)?;
        let prior_revision = record.record_revision;
        verify_request_digest(&self.inner.digest_key, request, &record.request_digest_hex)?;
        if record.stable_key != stable_key
            || record.acceptance_record_id != acceptance.proposed_acceptance_record_id
            || record.message_id != *message_id
            || self
                .record_identity_collision_locked(&record)
                .map_err(join_unsafe)?
                .is_some()
        {
            return Err(MemberTurnJoinError::RequestConflict);
        }
        adopt_one_published_frame(&record_directory, &mut record).map_err(join_unsafe)?;
        record_directory.sync().map_err(join_unsafe)?;
        validate_frame_journal(&record_directory, &record).map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        drop(_lock);
        drop(acceptance_namespace);
        if record.record_revision != prior_revision {
            self.notify_state_change(&record_id)?;
        }
        Ok(Some(MemberTurnJoinReservation {
            registry: self.clone(),
            record,
        }))
    }

    pub(crate) fn reserve(
        &self,
        request: &MemberTurnSubmitRequestV1,
    ) -> Result<MemberTurnJoinReservation, MemberTurnJoinError> {
        self.revalidate_root().map_err(join_unsafe)?;
        let stable_key = MemberTurnStableKeyV1::from_request(request).map_err(join_unsafe)?;
        let record_id = stable_key_id(&self.inner.digest_key, &stable_key).map_err(join_unsafe)?;
        let request_digest =
            request_digest(&self.inner.digest_key, request).map_err(join_unsafe)?;
        let acceptance = request
            .acceptance_context
            .as_ref()
            .ok_or_else(|| join_unsafe("E2 request omitted acceptance_context"))?;
        let message_id = acceptance
            .message_id
            .as_ref()
            .ok_or_else(|| join_unsafe("E2 acceptance_context omitted message_id"))?;
        let acceptance_namespace = self.inner.acceptance_namespace.lock()?;
        if acceptance_namespace.contains(&acceptance.proposed_acceptance_record_id) {
            return Err(MemberTurnJoinError::RequestConflict);
        }

        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        reconcile_recognized_temps(&self.inner.registry).map_err(join_unsafe)?;
        reconcile_record_tree_temps(&self.inner.records).map_err(join_unsafe)?;

        let shard = self.record_shard(&record_id).map_err(join_unsafe)?;
        let record_name = &record_id[2..];
        if let Some(record_directory) = shard
            .open_directory_if_present(record_name)
            .map_err(join_unsafe)?
        {
            reconcile_record_directory_temps(&record_directory).map_err(join_unsafe)?;
            let record = read_record(&record_directory, &record_id).map_err(join_unsafe)?;
            verify_request_digest(&self.inner.digest_key, request, &record.request_digest_hex)?;
            if record.stable_key != stable_key
                || record.acceptance_record_id != acceptance.proposed_acceptance_record_id
                || record.message_id != *message_id
            {
                return Err(MemberTurnJoinError::RequestConflict);
            }
            if self
                .record_identity_collision_locked(&record)
                .map_err(join_unsafe)?
                .is_some()
            {
                return Err(MemberTurnJoinError::RequestConflict);
            }
            return Ok(MemberTurnJoinReservation {
                registry: self.clone(),
                record,
            });
        }

        if count_record_directories(&self.inner.records).map_err(join_unsafe)?
            >= MAX_DURABLE_RECORDS
        {
            return Err(MemberTurnJoinError::CapacityExhausted);
        }
        let record = MemberTurnJoinRecordV1 {
            schema_version: 1,
            record_revision: 1,
            record_id: record_id.clone(),
            stable_key,
            request_digest_hex: request_digest,
            span_id: format!("spn_{}", uuid::Uuid::now_v7()),
            stream_id: format!("rts_{}", uuid::Uuid::now_v7()),
            acceptance_record_id: acceptance.proposed_acceptance_record_id.clone(),
            message_id: message_id.clone(),
            state: MemberTurnJoinStateV1::Reserved,
            last_frame_sequence: 0,
            retained_bytes: 0,
            frame_chain_sha256: "0".repeat(64),
        };
        record.validate(&record_id).map_err(join_unsafe)?;
        if self
            .record_identity_collision_locked(&record)
            .map_err(join_unsafe)?
            .is_some()
        {
            return Err(MemberTurnJoinError::RequestConflict);
        }
        let temp_record_name = format!(".{record_name}--{}.tmp", uuid::Uuid::now_v7().simple());
        let record_directory = shard
            .create_directory(&temp_record_name)
            .map_err(join_unsafe)?;
        let frames = record_directory
            .create_directory(FRAMES_DIRECTORY)
            .map_err(join_unsafe)?;
        let _leader_lock = record_directory
            .create_file(LEADER_LOCK_FILE)
            .map_err(join_unsafe)?;
        publish_json_no_replace(&record_directory, RECORD_FILE, &record).map_err(join_unsafe)?;
        frames.sync().map_err(join_unsafe)?;
        record_directory.sync().map_err(join_unsafe)?;
        shard
            .rename_directory_no_replace(&temp_record_name, &record_directory, record_name)
            .map_err(join_unsafe)?;
        shard.sync().map_err(join_unsafe)?;
        self.inner.records.sync().map_err(join_unsafe)?;
        self.inner.registry.sync().map_err(join_unsafe)?;
        let published_directory = shard.open_directory(record_name).map_err(join_unsafe)?;
        let readback = read_record(&published_directory, &record_id).map_err(join_unsafe)?;
        if readback != record {
            return Err(join_unsafe("member-turn reservation readback changed"));
        }
        self.revalidate_root().map_err(join_unsafe)?;
        Ok(MemberTurnJoinReservation {
            registry: self.clone(),
            record,
        })
    }

    fn record_shard(&self, record_id: &str) -> anyhow::Result<TrustedDirectory> {
        if record_id.len() != 64 || !is_lower_hex(record_id) {
            anyhow::bail!("member-turn record ID is invalid");
        }
        let shard = self
            .inner
            .records
            .open_or_create_directory(&record_id[..2])?;
        shard.enforce_private_mode()?;
        Ok(shard)
    }

    pub(crate) fn try_become_leader(
        &self,
        reservation: &MemberTurnJoinReservation,
    ) -> Result<Option<MemberTurnJoinLeader>, MemberTurnJoinError> {
        self.ensure_owned_reservation(reservation)?;
        let record_directory = self
            .open_record_directory(reservation.record_id())
            .map_err(join_unsafe)?;
        let leader_file = record_directory
            .open_file(LEADER_LOCK_FILE)
            .map_err(join_unsafe)?;
        let Some(leader_lock) = leader_file
            .try_lock_exclusive_owned()
            .map_err(join_unsafe)?
        else {
            return Ok(None);
        };
        let current = self.load_record(reservation.record_id())?;
        match current.state {
            MemberTurnJoinStateV1::LaunchingCallEntered { .. } => {
                let updated = self.transition_record(reservation.record_id(), |record| {
                    if !matches!(
                        record.state,
                        MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                    ) {
                        anyhow::bail!("member-turn call boundary state changed");
                    }
                    record.state = MemberTurnJoinStateV1::LaunchIndeterminate {
                        reason_code: "orphaned_post_call_execution_v1".to_string(),
                    };
                    Ok(())
                })?;
                self.close_subscribers(&updated.record_id)?;
                return Ok(None);
            }
            MemberTurnJoinStateV1::Started => {
                let recovery = ExecuteStreamFrame::Error {
                    frame_identity: RuntimeFrameIdentityV1 {
                        schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                        stream_id: current.stream_id,
                        frame_sequence: current.last_frame_sequence + 1,
                    },
                    message: "member turn outcome is indeterminate after leader loss".to_string(),
                };
                self.publish_frame(reservation.record_id(), &recovery, true)?;
                return Ok(None);
            }
            MemberTurnJoinStateV1::Completed { .. }
            | MemberTurnJoinStateV1::FailedBeforeLaunch { .. }
            | MemberTurnJoinStateV1::LaunchIndeterminate { .. } => return Ok(None),
            MemberTurnJoinStateV1::Reserved | MemberTurnJoinStateV1::LaunchingPreCall { .. } => {}
        }
        let leader_instance_id = format!("wsi_{}", uuid::Uuid::now_v7());
        let mut token = [0u8; 32];
        rand::rngs::OsRng.fill_bytes(&mut token);
        let leader_token_digest = lowercase_hex(&Sha256::digest(token));
        let updated = self.transition_record(reservation.record_id(), |record| {
            if !matches!(
                record.state,
                MemberTurnJoinStateV1::Reserved | MemberTurnJoinStateV1::LaunchingPreCall { .. }
            ) {
                anyhow::bail!("member-turn pre-call state changed during leader election");
            }
            record.state = MemberTurnJoinStateV1::LaunchingPreCall {
                leader_instance_id: leader_instance_id.clone(),
                leader_token_digest: leader_token_digest.clone(),
            };
            Ok(())
        })?;
        if !matches!(
            updated.state,
            MemberTurnJoinStateV1::LaunchingPreCall {
                leader_instance_id: ref stored_instance,
                leader_token_digest: ref stored_token,
            } if stored_instance == &leader_instance_id && stored_token == &leader_token_digest
        ) {
            return Ok(None);
        }
        Ok(Some(MemberTurnJoinLeader {
            reservation: MemberTurnJoinReservation {
                registry: self.clone(),
                record: updated,
            },
            leader_instance_id,
            leader_token_digest,
            _lock: leader_lock,
        }))
    }

    pub(crate) fn recover_orphaned(
        &self,
        reservation: &MemberTurnJoinReservation,
    ) -> Result<MemberTurnJoinStateV1, MemberTurnJoinError> {
        self.ensure_owned_reservation(reservation)?;
        let record_directory = self
            .open_record_directory(reservation.record_id())
            .map_err(join_unsafe)?;
        let leader_file = record_directory
            .open_file(LEADER_LOCK_FILE)
            .map_err(join_unsafe)?;
        let Some(_recovery_lock) = leader_file
            .try_lock_exclusive_owned()
            .map_err(join_unsafe)?
        else {
            return Ok(self.load_record(reservation.record_id())?.state);
        };
        let current = self.reconcile_published_frame(reservation.record_id())?;
        if current.stream_id != reservation.stream_id()
            || current.request_digest_hex != reservation.record.request_digest_hex
        {
            return Err(MemberTurnJoinError::RequestConflict);
        }
        let recovered = match &current.state {
            MemberTurnJoinStateV1::LaunchingPreCall { .. } => {
                self.transition_record(reservation.record_id(), |record| {
                    if !matches!(record.state, MemberTurnJoinStateV1::LaunchingPreCall { .. }) {
                        anyhow::bail!("member-turn PreCall recovery state changed");
                    }
                    record.state = MemberTurnJoinStateV1::Reserved;
                    Ok(())
                })?
            }
            MemberTurnJoinStateV1::LaunchingCallEntered { .. } => {
                let updated = self.transition_record(reservation.record_id(), |record| {
                    if !matches!(
                        record.state,
                        MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                    ) {
                        anyhow::bail!("member-turn CallEntered recovery state changed");
                    }
                    record.state = MemberTurnJoinStateV1::LaunchIndeterminate {
                        reason_code: "orphaned_post_call_execution_v1".to_string(),
                    };
                    Ok(())
                })?;
                self.close_subscribers(reservation.record_id())?;
                updated
            }
            MemberTurnJoinStateV1::Started => {
                let recovery = ExecuteStreamFrame::Error {
                    frame_identity: RuntimeFrameIdentityV1 {
                        schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                        stream_id: current.stream_id.clone(),
                        frame_sequence: current.last_frame_sequence + 1,
                    },
                    message: "member turn outcome is indeterminate after leader loss".to_string(),
                };
                self.publish_frame(reservation.record_id(), &recovery, true)?
            }
            MemberTurnJoinStateV1::Reserved
            | MemberTurnJoinStateV1::Completed { .. }
            | MemberTurnJoinStateV1::FailedBeforeLaunch { .. }
            | MemberTurnJoinStateV1::LaunchIndeterminate { .. } => current,
        };
        Ok(recovered.state)
    }

    pub(crate) fn subscribe(
        &self,
        reservation: &MemberTurnJoinReservation,
        after_frame_sequence: u64,
    ) -> Result<MemberTurnJoinSubscription, MemberTurnJoinError> {
        self.ensure_owned_reservation(reservation)?;
        self.subscribe_record(
            reservation.record_id(),
            reservation.stream_id(),
            after_frame_sequence,
        )
    }

    pub(crate) async fn await_launch_outcome(
        &self,
        reservation: &MemberTurnJoinReservation,
    ) -> Result<MemberTurnJoinStateV1, MemberTurnJoinError> {
        self.ensure_owned_reservation(reservation)?;
        let mut state_changes = {
            let mut notifiers =
                self.inner.state_notifiers.lock().map_err(|_| {
                    join_unsafe("member-turn state notifier registry lock poisoned")
                })?;
            notifiers
                .entry(reservation.record_id().to_string())
                .or_insert_with(|| watch::channel(0).0)
                .subscribe()
        };
        loop {
            let record = self.load_record(reservation.record_id())?;
            if record.stream_id != reservation.stream_id()
                || record.request_digest_hex != reservation.record.request_digest_hex
            {
                return Err(MemberTurnJoinError::RequestConflict);
            }
            match record.state {
                MemberTurnJoinStateV1::Reserved
                | MemberTurnJoinStateV1::LaunchingPreCall { .. }
                | MemberTurnJoinStateV1::LaunchingCallEntered { .. } => {
                    match tokio::time::timeout(DURABLE_STATE_POLL_INTERVAL, state_changes.changed())
                        .await
                    {
                        Ok(Ok(())) | Err(_) => {}
                        Ok(Err(_)) => {
                            return Err(join_unsafe("member-turn state notifier closed"));
                        }
                    }
                }
                MemberTurnJoinStateV1::FailedBeforeLaunch {
                    error_code,
                    http_status,
                } => {
                    return Err(MemberTurnJoinError::FailedBeforeLaunch {
                        error_code,
                        http_status,
                    })
                }
                MemberTurnJoinStateV1::LaunchIndeterminate { .. }
                    if record.last_frame_sequence == 0 =>
                {
                    return Err(MemberTurnJoinError::LaunchIndeterminate)
                }
                state => return Ok(state),
            }
        }
    }

    pub(crate) fn subscribe_for_replay(
        &self,
        acceptance_record_id: &str,
        stream_id: &str,
        after_frame_sequence: u64,
    ) -> Result<Option<MemberTurnJoinSubscription>, MemberTurnJoinError> {
        let Some(record) = self
            .record_by_acceptance(acceptance_record_id)
            .map_err(join_unsafe)?
        else {
            return Ok(None);
        };
        if record.stream_id != stream_id {
            return Err(MemberTurnJoinError::RequestConflict);
        }
        self.subscribe_record(&record.record_id, stream_id, after_frame_sequence)
            .map(Some)
    }

    pub(crate) fn claims_acceptance_record_id(
        &self,
        acceptance_record_id: &str,
    ) -> Result<bool, MemberTurnJoinError> {
        self.record_by_acceptance(acceptance_record_id)
            .map(|record| record.is_some())
            .map_err(join_unsafe)
    }

    fn subscribe_record(
        &self,
        record_id: &str,
        stream_id: &str,
        after_frame_sequence: u64,
    ) -> Result<MemberTurnJoinSubscription, MemberTurnJoinError> {
        self.revalidate_root().map_err(join_unsafe)?;
        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        let directory = self.open_record_directory(record_id).map_err(join_unsafe)?;
        let record = read_record(&directory, record_id).map_err(join_unsafe)?;
        if record.stream_id != stream_id || after_frame_sequence > record.last_frame_sequence {
            return Err(MemberTurnJoinError::RequestConflict);
        }
        if matches!(
            record.state,
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ) && record.last_frame_sequence == 0
        {
            return Err(MemberTurnJoinError::LaunchIndeterminate);
        }
        if matches!(
            record.state,
            MemberTurnJoinStateV1::FailedBeforeLaunch { .. }
        ) {
            let MemberTurnJoinStateV1::FailedBeforeLaunch {
                error_code,
                http_status,
            } = record.state
            else {
                unreachable!("matched FailedBeforeLaunch")
            };
            return Err(MemberTurnJoinError::FailedBeforeLaunch {
                error_code,
                http_status,
            });
        }
        validate_frame_journal(&directory, &record).map_err(join_unsafe)?;
        let (sender, receiver) = mpsc::channel(1);
        if !matches!(
            record.state,
            MemberTurnJoinStateV1::Completed { .. }
                | MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ) {
            let mut subscribers = self
                .inner
                .subscribers
                .lock()
                .map_err(|_| join_unsafe("member-turn subscriber registry lock poisoned"))?;
            let active = subscribers.entry(record_id.to_string()).or_default();
            active.retain(|sender| !sender.is_closed());
            if active.len() >= MAX_LIVE_SUBSCRIBERS_PER_RECORD {
                return Err(MemberTurnJoinError::CapacityExhausted);
            }
            active.push(sender);
        }
        let subscription = MemberTurnJoinSubscription {
            registry: self.clone(),
            record_id: record_id.to_string(),
            stream_id: stream_id.to_string(),
            next_frame_sequence: after_frame_sequence + 1,
            receiver,
            poll_delay: None,
            local_notifications_closed: false,
            finished: false,
        };
        self.revalidate_root().map_err(join_unsafe)?;
        Ok(subscription)
    }

    fn read_subscription_frame(
        &self,
        record_id: &str,
        stream_id: &str,
        sequence: u64,
    ) -> anyhow::Result<SubscriptionFrameRead> {
        self.revalidate_root()?;
        let lock_file = self.inner.registry.open_file(REGISTRY_LOCK_FILE)?;
        let _lock = lock_file.lock_exclusive()?;
        let directory = self.open_record_directory(record_id)?;
        let record = read_record(&directory, record_id)?;
        if record.stream_id != stream_id || sequence == 0 {
            anyhow::bail!("member-turn subscription identity changed");
        }
        let result = if sequence <= record.last_frame_sequence {
            let frames_directory = directory.open_directory(FRAMES_DIRECTORY)?;
            let bytes = frames_directory
                .open_file(&frame_file_name(sequence))?
                .read_all()?;
            let frame: ExecuteStreamFrame = serde_json::from_slice(&bytes)?;
            if frame.canonical_ndjson_bytes().map_err(anyhow::Error::msg)? != bytes
                || frame_sequence(&frame) != sequence
                || frame_stream_id(&frame) != record.stream_id
                || (sequence == 1
                    && !matches!(
                        &frame,
                        ExecuteStreamFrame::Start { span_id, .. } if span_id == &record.span_id
                    ))
                || (sequence > 1 && matches!(frame, ExecuteStreamFrame::Start { .. }))
            {
                anyhow::bail!("member-turn subscription frame changed");
            }
            SubscriptionFrameRead::Frame(Box::new(frame))
        } else {
            if sequence != record.last_frame_sequence.saturating_add(1) {
                anyhow::bail!("member-turn subscription cursor is invalid");
            }
            if matches!(
                record.state,
                MemberTurnJoinStateV1::Completed { .. }
                    | MemberTurnJoinStateV1::FailedBeforeLaunch { .. }
                    | MemberTurnJoinStateV1::LaunchIndeterminate { .. }
            ) {
                SubscriptionFrameRead::Finished
            } else {
                SubscriptionFrameRead::Pending
            }
        };
        self.revalidate_root()?;
        Ok(result)
    }

    fn transition_record(
        &self,
        record_id: &str,
        transition: impl FnOnce(&mut MemberTurnJoinRecordV1) -> anyhow::Result<()>,
    ) -> Result<MemberTurnJoinRecordV1, MemberTurnJoinError> {
        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        let directory = self.open_record_directory(record_id).map_err(join_unsafe)?;
        let mut record = read_record(&directory, record_id).map_err(join_unsafe)?;
        transition(&mut record).map_err(join_unsafe)?;
        record.record_revision = record
            .record_revision
            .checked_add(1)
            .ok_or_else(|| join_unsafe("member-turn record revision exhausted"))?;
        record.validate(record_id).map_err(join_unsafe)?;
        replace_json(&directory, RECORD_FILE, &record).map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        self.notify_state_change(record_id)?;
        Ok(record)
    }

    fn load_record(&self, record_id: &str) -> Result<MemberTurnJoinRecordV1, MemberTurnJoinError> {
        self.revalidate_root().map_err(join_unsafe)?;
        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        let directory = self.open_record_directory(record_id).map_err(join_unsafe)?;
        let record = read_record(&directory, record_id).map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        Ok(record)
    }

    fn reconcile_published_frame(
        &self,
        record_id: &str,
    ) -> Result<MemberTurnJoinRecordV1, MemberTurnJoinError> {
        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        let directory = self.open_record_directory(record_id).map_err(join_unsafe)?;
        reconcile_record_directory_temps(&directory).map_err(join_unsafe)?;
        let mut record = read_record(&directory, record_id).map_err(join_unsafe)?;
        let prior_revision = record.record_revision;
        adopt_one_published_frame(&directory, &mut record).map_err(join_unsafe)?;
        directory.sync().map_err(join_unsafe)?;
        validate_frame_journal(&directory, &record).map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        drop(_lock);
        if record.record_revision != prior_revision {
            self.notify_state_change(record_id)?;
        }
        Ok(record)
    }

    fn publish_frame(
        &self,
        record_id: &str,
        frame: &ExecuteStreamFrame,
        recovery_indeterminate: bool,
    ) -> Result<MemberTurnJoinRecordV1, MemberTurnJoinError> {
        let lock_file = self
            .inner
            .registry
            .open_file(REGISTRY_LOCK_FILE)
            .map_err(join_unsafe)?;
        let _lock = lock_file.lock_exclusive().map_err(join_unsafe)?;
        self.revalidate_root().map_err(join_unsafe)?;
        let directory = self.open_record_directory(record_id).map_err(join_unsafe)?;
        let frames_directory = directory
            .open_directory(FRAMES_DIRECTORY)
            .map_err(join_unsafe)?;
        let mut record = read_record(&directory, record_id).map_err(join_unsafe)?;
        let sequence = frame_sequence(frame);
        let expected = record
            .last_frame_sequence
            .checked_add(1)
            .ok_or_else(|| join_unsafe("member-turn frame sequence exhausted"))?;
        if sequence != expected
            || frame_stream_id(frame) != record.stream_id
            || (sequence == 1 && !matches!(frame, ExecuteStreamFrame::Start { .. }))
            || (sequence > 1 && matches!(frame, ExecuteStreamFrame::Start { .. }))
        {
            return Err(join_unsafe("member-turn durable frame identity changed"));
        }
        if let ExecuteStreamFrame::Start { span_id, .. } = frame {
            if span_id != &record.span_id
                || !matches!(
                    record.state,
                    MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                )
            {
                return Err(join_unsafe("member-turn Start does not join CallEntered"));
            }
        } else if !matches!(
            record.state,
            MemberTurnJoinStateV1::Started | MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ) {
            return Err(join_unsafe(
                "member-turn post-Start frame has invalid state",
            ));
        }
        let bytes = frame.canonical_ndjson_bytes().map_err(join_unsafe)?;
        let retained_bytes = record
            .retained_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| join_unsafe("member-turn retained byte count overflow"))?;
        if sequence > 65_536 || retained_bytes > 64 * 1024 * 1024 {
            return Err(join_unsafe(
                "member-turn durable frame capacity is exhausted",
            ));
        }
        let frame_name = frame_file_name(sequence);
        publish_bytes_no_replace(&frames_directory, &frame_name, &bytes).map_err(join_unsafe)?;
        record.last_frame_sequence = sequence;
        record.retained_bytes = retained_bytes;
        record.frame_chain_sha256 =
            next_frame_chain_hash(&record.frame_chain_sha256, &bytes).map_err(join_unsafe)?;
        record.state = if frame_is_terminal(frame) {
            if recovery_indeterminate {
                MemberTurnJoinStateV1::LaunchIndeterminate {
                    reason_code: "orphaned_started_execution_v1".to_string(),
                }
            } else {
                MemberTurnJoinStateV1::Completed {
                    terminal_frame_sequence: sequence,
                }
            }
        } else {
            MemberTurnJoinStateV1::Started
        };
        record.record_revision = record
            .record_revision
            .checked_add(1)
            .ok_or_else(|| join_unsafe("member-turn record revision exhausted"))?;
        record.validate(record_id).map_err(join_unsafe)?;
        if let Err(publication_error) = replace_json(&directory, RECORD_FILE, &record) {
            let recovered = (|| -> anyhow::Result<MemberTurnJoinRecordV1> {
                reconcile_record_directory_temps(&directory)?;
                let mut durable = read_record(&directory, record_id)?;
                adopt_one_published_frame(&directory, &mut durable)?;
                directory.sync()?;
                validate_frame_journal(&directory, &durable)?;
                if durable.last_frame_sequence != sequence {
                    anyhow::bail!("member-turn frame metadata recovery changed sequence");
                }
                Ok(durable)
            })()
            .map_err(|recovery_error| {
                join_unsafe(format!(
                    "member-turn frame metadata publication failed ({publication_error:#}); recovery failed ({recovery_error:#})"
                ))
            })?;
            record = recovered;
        }
        self.revalidate_root().map_err(join_unsafe)?;
        drop(_lock);
        self.deliver_durable_frame(record_id, frame.clone())?;
        self.notify_state_change(record_id)?;
        Ok(record)
    }

    fn deliver_durable_frame(
        &self,
        record_id: &str,
        frame: ExecuteStreamFrame,
    ) -> Result<(), MemberTurnJoinError> {
        let terminal = frame_is_terminal(&frame);
        let mut subscribers = self
            .inner
            .subscribers
            .lock()
            .map_err(|_| join_unsafe("member-turn subscriber registry lock poisoned"))?;
        if let Some(active) = subscribers.get_mut(record_id) {
            active.retain(|sender| match sender.try_send(()) {
                Ok(()) | Err(mpsc::error::TrySendError::Full(())) => true,
                Err(mpsc::error::TrySendError::Closed(())) => false,
            });
        }
        if terminal {
            subscribers.remove(record_id);
        }
        Ok(())
    }

    fn close_subscribers(&self, record_id: &str) -> Result<(), MemberTurnJoinError> {
        self.inner
            .subscribers
            .lock()
            .map_err(|_| join_unsafe("member-turn subscriber registry lock poisoned"))?
            .remove(record_id);
        Ok(())
    }

    fn notify_state_change(&self, record_id: &str) -> Result<(), MemberTurnJoinError> {
        if let Some(notifier) = self
            .inner
            .state_notifiers
            .lock()
            .map_err(|_| join_unsafe("member-turn state notifier registry lock poisoned"))?
            .get(record_id)
        {
            notifier.send_modify(|revision| *revision = revision.wrapping_add(1));
        }
        Ok(())
    }

    fn ensure_owned_reservation(
        &self,
        reservation: &MemberTurnJoinReservation,
    ) -> Result<(), MemberTurnJoinError> {
        if !Arc::ptr_eq(&self.inner, &reservation.registry.inner) {
            return Err(join_unsafe(
                "member-turn reservation belongs to another registry",
            ));
        }
        Ok(())
    }

    fn open_record_directory(&self, record_id: &str) -> anyhow::Result<TrustedDirectory> {
        if record_id.len() != 64 || !is_lower_hex(record_id) {
            anyhow::bail!("member-turn record ID is invalid");
        }
        self.inner
            .records
            .open_directory(&record_id[..2])?
            .open_directory(&record_id[2..])
    }

    fn record_by_acceptance(
        &self,
        acceptance_record_id: &str,
    ) -> anyhow::Result<Option<MemberTurnJoinRecordV1>> {
        self.revalidate_root()?;
        let lock_file = self.inner.registry.open_file(REGISTRY_LOCK_FILE)?;
        let _lock = lock_file.lock_exclusive()?;
        self.revalidate_root()?;
        let record = self.record_by_acceptance_locked(acceptance_record_id)?;
        self.revalidate_root()?;
        Ok(record)
    }

    fn record_by_acceptance_locked(
        &self,
        acceptance_record_id: &str,
    ) -> anyhow::Result<Option<MemberTurnJoinRecordV1>> {
        let mut found = None;
        for shard_name in self.inner.records.entries()? {
            let shard = self.inner.records.open_directory(&shard_name)?;
            for record_name in shard.entries()? {
                let record_id = format!("{shard_name}{record_name}");
                let record = read_record(&shard.open_directory(&record_name)?, &record_id)?;
                if record.acceptance_record_id == acceptance_record_id {
                    if found.is_some() {
                        anyhow::bail!("member-turn acceptance identity is not unique");
                    }
                    found = Some(record);
                }
            }
        }
        Ok(found)
    }

    fn record_identity_collision_locked(
        &self,
        candidate: &MemberTurnJoinRecordV1,
    ) -> anyhow::Result<Option<&'static str>> {
        for shard_name in self.inner.records.entries()? {
            let shard = self.inner.records.open_directory(&shard_name)?;
            for record_name in shard.entries()? {
                let record_id = format!("{shard_name}{record_name}");
                let record = read_record(&shard.open_directory(&record_name)?, &record_id)?;
                if record.record_id == candidate.record_id {
                    continue;
                }
                if record.acceptance_record_id == candidate.acceptance_record_id {
                    return Ok(Some("acceptance_record_id"));
                }
                if record.stream_id == candidate.stream_id {
                    return Ok(Some("stream_id"));
                }
                if record.span_id == candidate.span_id {
                    return Ok(Some("span_id"));
                }
            }
        }
        Ok(None)
    }

    fn recover_and_validate_existing_records(&self) -> anyhow::Result<()> {
        let lock_file = self.inner.registry.open_file(REGISTRY_LOCK_FILE)?;
        let _lock = lock_file.lock_exclusive()?;
        self.revalidate_root()?;
        reconcile_recognized_temps(&self.inner.registry)?;
        reconcile_record_tree_temps(&self.inner.records)?;
        let mut acceptance_record_ids = HashSet::new();
        let mut stream_ids = HashSet::new();
        let mut span_ids = HashSet::new();
        for shard_entry in self.inner.records.entries()? {
            if shard_entry.len() != 2 || !is_lower_hex(&shard_entry) {
                anyhow::bail!("member-turn record shard name is invalid");
            }
            let shard = self.inner.records.open_directory(&shard_entry)?;
            for record_name in shard.entries()? {
                if record_name.len() != 62 || !is_lower_hex(&record_name) {
                    anyhow::bail!("member-turn record directory name is invalid");
                }
                let record_id = format!("{shard_entry}{record_name}");
                let record_directory = shard.open_directory(&record_name)?;
                validate_record_layout(&record_directory)?;
                let mut record = read_record(&record_directory, &record_id)?;
                let expected_id = stable_key_id(&self.inner.digest_key, &record.stable_key)?;
                if expected_id != record_id {
                    anyhow::bail!("member-turn record key HMAC mismatch");
                }
                if !acceptance_record_ids.insert(record.acceptance_record_id.clone())
                    || !stream_ids.insert(record.stream_id.clone())
                    || !span_ids.insert(record.span_id.clone())
                {
                    anyhow::bail!("member-turn durable identity is not globally unique");
                }
                let leader_file = record_directory.open_file(LEADER_LOCK_FILE)?;
                let Some(_recovery_leader_lock) = leader_file.try_lock_exclusive_owned()? else {
                    validate_frame_journal(&record_directory, &record)?;
                    continue;
                };
                self.revalidate_root()?;
                adopt_one_published_frame(&record_directory, &mut record)?;
                self.revalidate_root()?;
                validate_frame_journal(&record_directory, &record)?;
                match record.state {
                    MemberTurnJoinStateV1::LaunchingPreCall { .. } => {
                        let mut recovered = record;
                        recovered.record_revision = recovered
                            .record_revision
                            .checked_add(1)
                            .ok_or_else(|| anyhow::anyhow!("member-turn revision exhausted"))?;
                        recovered.state = MemberTurnJoinStateV1::Reserved;
                        replace_json(&record_directory, RECORD_FILE, &recovered)?;
                        self.revalidate_root()?;
                    }
                    MemberTurnJoinStateV1::LaunchingCallEntered { .. } => {
                        let mut recovered = record;
                        recovered.record_revision = recovered
                            .record_revision
                            .checked_add(1)
                            .ok_or_else(|| anyhow::anyhow!("member-turn revision exhausted"))?;
                        recovered.state = MemberTurnJoinStateV1::LaunchIndeterminate {
                            reason_code: "restart_after_call_entered_v1".to_string(),
                        };
                        replace_json(&record_directory, RECORD_FILE, &recovered)?;
                        self.revalidate_root()?;
                    }
                    MemberTurnJoinStateV1::Started => {
                        drop(_lock);
                        let error_frame = ExecuteStreamFrame::Error {
                            frame_identity: RuntimeFrameIdentityV1 {
                                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                                stream_id: record.stream_id.clone(),
                                frame_sequence: record.last_frame_sequence + 1,
                            },
                            message: "member turn outcome is indeterminate after service restart"
                                .to_string(),
                        };
                        self.publish_frame(&record_id, &error_frame, true)
                            .map_err(anyhow::Error::new)?;
                        return self.recover_and_validate_existing_records();
                    }
                    _ => {}
                }
            }
        }
        self.revalidate_root()?;
        Ok(())
    }

    fn revalidate_root(&self) -> anyhow::Result<()> {
        let reopened = TrustedDirectory::open_absolute(&self.inner.state_root_path, false)?;
        let stat = reopened.stat()?;
        validate_directory_stat(&stat, false)?;
        if stat.st_dev != self.inner.state_root_device || stat.st_ino != self.inner.state_root_inode
        {
            anyhow::bail!("member-turn state root was replaced");
        }
        let registry = reopened.open_directory(REGISTRY_DIRECTORY)?;
        let registry_stat = registry.stat()?;
        validate_directory_stat(&registry_stat, true)?;
        if registry_stat.st_dev != self.inner.registry_device
            || registry_stat.st_ino != self.inner.registry_inode
        {
            anyhow::bail!("member-turn registry directory was replaced");
        }
        let records = registry.open_directory(RECORDS_DIRECTORY)?;
        let records_stat = records.stat()?;
        validate_directory_stat(&records_stat, true)?;
        if records_stat.st_dev != self.inner.records_device
            || records_stat.st_ino != self.inner.records_inode
        {
            anyhow::bail!("member-turn records directory was replaced");
        }
        Ok(())
    }
}

impl MemberTurnJoinLeader {
    pub(crate) fn mark_call_entered(&mut self) -> Result<(), MemberTurnJoinError> {
        let leader_instance_id = self.leader_instance_id.clone();
        let leader_token_digest = self.leader_token_digest.clone();
        let updated = self.reservation.registry.transition_record(
            self.reservation.record_id(),
            |record| match &record.state {
                MemberTurnJoinStateV1::LaunchingPreCall {
                    leader_instance_id: stored_instance,
                    leader_token_digest: stored_token,
                } if stored_instance == &leader_instance_id
                    && stored_token == &leader_token_digest =>
                {
                    record.state = MemberTurnJoinStateV1::LaunchingCallEntered {
                        leader_instance_id: leader_instance_id.clone(),
                        leader_token_digest: leader_token_digest.clone(),
                    };
                    Ok(())
                }
                _ => anyhow::bail!("member-turn leader lost PreCall authority"),
            },
        )?;
        self.reservation.record = updated;
        Ok(())
    }

    pub(crate) fn publish_frame(
        &mut self,
        frame: &ExecuteStreamFrame,
    ) -> Result<(), MemberTurnJoinError> {
        let updated =
            self.reservation
                .registry
                .publish_frame(self.reservation.record_id(), frame, false)?;
        self.reservation.record = updated;
        Ok(())
    }

    pub(crate) fn fail_before_launch(
        &mut self,
        error_code: &str,
        http_status: u16,
    ) -> Result<(), MemberTurnJoinError> {
        if error_code.trim().is_empty() || !(400..600).contains(&http_status) {
            return Err(join_unsafe("member-turn prelaunch failure is invalid"));
        }
        let leader_instance_id = self.leader_instance_id.clone();
        let leader_token_digest = self.leader_token_digest.clone();
        let updated = self.reservation.registry.transition_record(
            self.reservation.record_id(),
            |record| match &record.state {
                MemberTurnJoinStateV1::LaunchingPreCall {
                    leader_instance_id: stored_instance,
                    leader_token_digest: stored_token,
                } if stored_instance == &leader_instance_id
                    && stored_token == &leader_token_digest =>
                {
                    record.state = MemberTurnJoinStateV1::FailedBeforeLaunch {
                        error_code: error_code.to_string(),
                        http_status,
                    };
                    Ok(())
                }
                _ => anyhow::bail!("member-turn leader cannot publish prelaunch failure"),
            },
        )?;
        self.reservation.record = updated;
        self.reservation
            .registry
            .close_subscribers(self.reservation.record_id())?;
        Ok(())
    }

    pub(crate) fn mark_launch_indeterminate(
        &mut self,
        reason_code: &str,
    ) -> Result<(), MemberTurnJoinError> {
        if reason_code.trim().is_empty() {
            return Err(join_unsafe("member-turn indeterminate reason is empty"));
        }
        let current = self
            .reservation
            .registry
            .reconcile_published_frame(self.reservation.record_id())?;
        match current.state {
            MemberTurnJoinStateV1::LaunchingCallEntered { .. } => {
                let reason_code = reason_code.to_string();
                let updated = self.reservation.registry.transition_record(
                    self.reservation.record_id(),
                    |record| {
                        if !matches!(
                            record.state,
                            MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                        ) {
                            anyhow::bail!("member-turn call boundary state changed");
                        }
                        record.state = MemberTurnJoinStateV1::LaunchIndeterminate {
                            reason_code: reason_code.clone(),
                        };
                        Ok(())
                    },
                )?;
                self.reservation.record = updated;
                self.reservation
                    .registry
                    .close_subscribers(self.reservation.record_id())?;
            }
            MemberTurnJoinStateV1::Started => {
                let error = ExecuteStreamFrame::Error {
                    frame_identity: RuntimeFrameIdentityV1 {
                        schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                        stream_id: current.stream_id,
                        frame_sequence: current.last_frame_sequence + 1,
                    },
                    message: "member turn outcome is indeterminate after launch".to_string(),
                };
                let updated = self.reservation.registry.publish_frame(
                    self.reservation.record_id(),
                    &error,
                    true,
                )?;
                self.reservation.record = updated;
            }
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
            | MemberTurnJoinStateV1::Completed { .. } => {}
            _ => {
                return Err(join_unsafe(
                    "member-turn cannot become indeterminate before call",
                ))
            }
        }
        Ok(())
    }
}

fn open_or_initialize_digest_key(
    registry: &TrustedDirectory,
    records: &TrustedDirectory,
) -> anyhow::Result<(String, [u8; 32])> {
    match registry.open_file_if_present(KEY_FILE)? {
        Some(file) => {
            let envelope: MemberTurnDigestKeyEnvelopeV1 = decode_exact_json(&file.read_all()?)?;
            if envelope.schema_version != 1 || !envelope.key_id.starts_with("mtk_") {
                anyhow::bail!("member-turn digest key envelope is invalid");
            }
            use base64::Engine as _;
            let bytes = base64::engine::general_purpose::STANDARD
                .decode(&envelope.secret_key_base64)
                .map_err(|_| anyhow::anyhow!("member-turn digest key is not canonical base64"))?;
            if bytes.len() != 32
                || base64::engine::general_purpose::STANDARD.encode(&bytes)
                    != envelope.secret_key_base64
            {
                anyhow::bail!("member-turn digest key length or encoding is invalid");
            }
            let mut key = [0u8; 32];
            key.copy_from_slice(&bytes);
            Ok((envelope.key_id, key))
        }
        None => {
            if !records.entries()?.is_empty() {
                anyhow::bail!("member-turn digest key is missing while records exist");
            }
            let mut key = [0u8; 32];
            rand::rngs::OsRng.fill_bytes(&mut key);
            use base64::Engine as _;
            let envelope = MemberTurnDigestKeyEnvelopeV1 {
                schema_version: 1,
                key_id: format!("mtk_{}", uuid::Uuid::now_v7()),
                secret_key_base64: base64::engine::general_purpose::STANDARD.encode(key),
            };
            publish_json_no_replace(registry, KEY_FILE, &envelope)?;
            Ok((envelope.key_id, key))
        }
    }
}

fn validate_record_layout(directory: &TrustedDirectory) -> anyhow::Result<()> {
    let mut entries = directory.entries()?;
    entries.sort();
    let expected = [
        FRAMES_DIRECTORY.to_string(),
        LEADER_LOCK_FILE.to_string(),
        RECORD_FILE.to_string(),
    ];
    if entries != expected {
        anyhow::bail!("member-turn record layout contains unknown entries");
    }
    directory.open_directory(FRAMES_DIRECTORY)?;
    directory.open_file(LEADER_LOCK_FILE)?;
    directory.open_file(RECORD_FILE)?;
    Ok(())
}

fn validate_registry_layout(registry: &TrustedDirectory) -> anyhow::Result<()> {
    let entries = registry.entries()?;
    let expected = [
        KEY_FILE.to_string(),
        FORMAT_FILE.to_string(),
        RECORDS_DIRECTORY.to_string(),
        REGISTRY_LOCK_FILE.to_string(),
    ];
    if entries != expected {
        anyhow::bail!("member-turn registry contains unknown entries");
    }
    registry.open_file(KEY_FILE)?;
    registry.open_file(FORMAT_FILE)?;
    registry.open_file(REGISTRY_LOCK_FILE)?;
    registry.open_directory(RECORDS_DIRECTORY)?;
    Ok(())
}

fn reconcile_record_tree_temps(records: &TrustedDirectory) -> anyhow::Result<()> {
    for shard_name in records.entries()? {
        if shard_name.len() != 2 || !is_lower_hex(&shard_name) {
            anyhow::bail!("member-turn record shard name is invalid");
        }
        let shard = records.open_directory(&shard_name)?;
        for entry in shard.entries()? {
            if is_record_name(&entry) {
                let record = shard.open_directory(&entry)?;
                reconcile_record_directory_temps(&record)?;
                continue;
            }
            if is_record_temp_name(&entry) {
                remove_record_temp_directory(&shard, &entry)?;
                continue;
            }
            anyhow::bail!("member-turn record shard contains an unknown entry");
        }
    }
    Ok(())
}

fn reconcile_record_directory_temps(directory: &TrustedDirectory) -> anyhow::Result<()> {
    reconcile_recognized_temps(directory)?;
    if let Some(frames) = directory.open_directory_if_present(FRAMES_DIRECTORY)? {
        reconcile_recognized_temps(&frames)?;
    }
    Ok(())
}

fn remove_record_temp_directory(shard: &TrustedDirectory, name: &str) -> anyhow::Result<()> {
    let temp = shard.open_directory(name)?;
    reconcile_recognized_temps(&temp)?;
    if let Some(frames) = temp.open_directory_if_present(FRAMES_DIRECTORY)? {
        reconcile_recognized_temps(&frames)?;
        if !frames.entries()?.is_empty() {
            anyhow::bail!("member-turn reservation temp contains a published frame");
        }
        temp.remove_empty_directory(FRAMES_DIRECTORY, &frames)?;
    }
    for file_name in [LEADER_LOCK_FILE, RECORD_FILE] {
        if let Some(file) = temp.open_file_if_present(file_name)? {
            temp.unlink_verified_file(file_name, &file)?;
        }
    }
    if !temp.entries()?.is_empty() {
        anyhow::bail!("member-turn reservation temp contains unknown entries");
    }
    shard.remove_empty_directory(name, &temp)?;
    shard.sync()?;
    Ok(())
}

fn is_record_name(name: &str) -> bool {
    name.len() == 62 && is_lower_hex(name)
}

fn is_record_temp_name(name: &str) -> bool {
    let Some((record_name, nonce)) = name
        .strip_prefix('.')
        .and_then(|value| value.strip_suffix(".tmp"))
        .and_then(|value| value.rsplit_once("--"))
    else {
        return false;
    };
    is_record_name(record_name) && nonce.len() == 32 && is_lower_hex(nonce)
}

fn read_record(
    directory: &TrustedDirectory,
    record_id: &str,
) -> anyhow::Result<MemberTurnJoinRecordV1> {
    validate_record_layout(directory)?;
    let bytes = directory.open_file(RECORD_FILE)?.read_all()?;
    let record: MemberTurnJoinRecordV1 = decode_exact_json(&bytes)?;
    record.validate(record_id)?;
    Ok(record)
}

fn verify_request_digest(
    key: &[u8],
    request: &MemberTurnSubmitRequestV1,
    stored_hex: &str,
) -> Result<(), MemberTurnJoinError> {
    let stored = decode_lower_hex_32(stored_hex).map_err(join_unsafe)?;
    let canonical = canonical_request_bytes(request).map_err(join_unsafe)?;
    let mut mac = HmacSha256::new_from_slice(key)
        .map_err(|_| join_unsafe("member-turn HMAC key length is invalid"))?;
    mac.update(&(REQUEST_DIGEST_DOMAIN.len() as u64).to_be_bytes());
    mac.update(REQUEST_DIGEST_DOMAIN);
    mac.update(&(canonical.len() as u64).to_be_bytes());
    mac.update(&canonical);
    mac.verify_slice(&stored)
        .map_err(|_| MemberTurnJoinError::RequestConflict)
}

fn count_record_directories(records: &TrustedDirectory) -> anyhow::Result<usize> {
    let mut count = 0usize;
    for shard_name in records.entries()? {
        let shard = records.open_directory(&shard_name)?;
        count = count
            .checked_add(shard.entries()?.len())
            .ok_or_else(|| anyhow::anyhow!("member-turn record count overflow"))?;
    }
    Ok(count)
}

fn create_or_exact_join_json<T>(
    directory: &TrustedDirectory,
    name: &str,
    value: &T,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    let expected = serde_json::to_vec(value)?;
    match directory.open_file_if_present(name)? {
        Some(file) => {
            if file.read_all()? != expected {
                anyhow::bail!("member-turn durable format identity changed");
            }
        }
        None => publish_bytes_no_replace(directory, name, &expected)?,
    }
    Ok(())
}

fn replace_json<T>(directory: &TrustedDirectory, name: &str, value: &T) -> anyhow::Result<()>
where
    T: Serialize,
{
    let bytes = serde_json::to_vec(value)?;
    let temp_name = format!(".{name}--{}.tmp", uuid::Uuid::now_v7().simple());
    let mut temp = directory.create_file(&temp_name)?;
    temp.write_all(&bytes)?;
    temp.sync()?;
    directory.rename_replace(&temp_name, &temp, name)?;
    directory.sync()?;
    if directory.open_file(name)?.read_all()? != bytes {
        anyhow::bail!("member-turn durable replacement readback changed");
    }
    Ok(())
}

fn publish_json_no_replace<T>(
    directory: &TrustedDirectory,
    name: &str,
    value: &T,
) -> anyhow::Result<()>
where
    T: Serialize,
{
    publish_bytes_no_replace(directory, name, &serde_json::to_vec(value)?)
}

fn publish_bytes_no_replace(
    directory: &TrustedDirectory,
    name: &str,
    bytes: &[u8],
) -> anyhow::Result<()> {
    let temp_name = format!(".{name}--{}.tmp", uuid::Uuid::now_v7().simple());
    let mut temp = directory.create_file(&temp_name)?;
    temp.write_all(bytes)?;
    temp.sync()?;
    directory.rename_no_replace(&temp_name, &temp, name)?;
    directory.sync()?;
    if directory.open_file(name)?.read_all()? != bytes {
        anyhow::bail!("member-turn durable publication readback changed");
    }
    Ok(())
}

fn reconcile_recognized_temps(directory: &TrustedDirectory) -> anyhow::Result<()> {
    let mut removed = false;
    for entry in directory.entries()? {
        if !entry.starts_with('.') || !entry.ends_with(".tmp") {
            continue;
        }
        if !recognized_temp_name(&entry) {
            anyhow::bail!("member-turn durable directory contains an unknown temporary object");
        }
        let file = directory
            .open_file_if_present(&entry)?
            .ok_or_else(|| anyhow::anyhow!("member-turn temporary object disappeared"))?;
        directory.unlink_verified_file(&entry, &file)?;
        removed = true;
    }
    if removed {
        directory.sync()?;
    }
    Ok(())
}

fn frame_file_name(sequence: u64) -> String {
    format!("{sequence:020}.frame")
}

fn parse_frame_file_name(name: &str) -> anyhow::Result<u64> {
    let digits = name
        .strip_suffix(".frame")
        .filter(|digits| digits.len() == 20 && digits.bytes().all(|byte| byte.is_ascii_digit()))
        .ok_or_else(|| anyhow::anyhow!("member-turn frame filename is invalid"))?;
    let sequence = digits.parse::<u64>()?;
    if sequence == 0 || frame_file_name(sequence) != name {
        anyhow::bail!("member-turn frame filename is not canonical");
    }
    Ok(sequence)
}

fn frame_sequence(frame: &ExecuteStreamFrame) -> u64 {
    match frame {
        ExecuteStreamFrame::Start { frame_identity, .. }
        | ExecuteStreamFrame::Stdout { frame_identity, .. }
        | ExecuteStreamFrame::Stderr { frame_identity, .. }
        | ExecuteStreamFrame::Event { frame_identity, .. }
        | ExecuteStreamFrame::Exit { frame_identity, .. }
        | ExecuteStreamFrame::Error { frame_identity, .. } => frame_identity.frame_sequence,
    }
}

fn frame_stream_id(frame: &ExecuteStreamFrame) -> &str {
    match frame {
        ExecuteStreamFrame::Start { frame_identity, .. }
        | ExecuteStreamFrame::Stdout { frame_identity, .. }
        | ExecuteStreamFrame::Stderr { frame_identity, .. }
        | ExecuteStreamFrame::Event { frame_identity, .. }
        | ExecuteStreamFrame::Exit { frame_identity, .. }
        | ExecuteStreamFrame::Error { frame_identity, .. } => &frame_identity.stream_id,
    }
}

fn frame_is_terminal(frame: &ExecuteStreamFrame) -> bool {
    matches!(
        frame,
        ExecuteStreamFrame::Exit { .. } | ExecuteStreamFrame::Error { .. }
    )
}

fn next_frame_chain_hash(previous_hex: &str, bytes: &[u8]) -> anyhow::Result<String> {
    let previous = decode_lower_hex_32(previous_hex)?;
    let mut hasher = Sha256::new();
    hasher.update(previous);
    hasher.update((bytes.len() as u64).to_be_bytes());
    hasher.update(bytes);
    Ok(lowercase_hex(&hasher.finalize()))
}

fn validate_frame_journal(
    record_directory: &TrustedDirectory,
    record: &MemberTurnJoinRecordV1,
) -> anyhow::Result<()> {
    let frames_directory = record_directory.open_directory(FRAMES_DIRECTORY)?;
    let names = frames_directory.entries()?;
    if names.len() as u64 != record.last_frame_sequence {
        anyhow::bail!("member-turn frame journal has a gap or uncommitted entry");
    }
    let mut previous_was_terminal = false;
    let mut retained_bytes = 0u64;
    let mut chain = "0".repeat(64);
    for (index, name) in names.into_iter().enumerate() {
        let sequence = parse_frame_file_name(&name)?;
        if sequence != index as u64 + 1 {
            anyhow::bail!("member-turn frame journal sequence is not contiguous");
        }
        let bytes = frames_directory.open_file(&name)?.read_all()?;
        let frame: ExecuteStreamFrame = serde_json::from_slice(&bytes)?;
        let canonical = frame.canonical_ndjson_bytes().map_err(anyhow::Error::msg)?;
        if canonical != bytes
            || frame_sequence(&frame) != sequence
            || frame_stream_id(&frame) != record.stream_id
        {
            anyhow::bail!("member-turn frame journal bytes or identity changed");
        }
        if sequence == 1 {
            match &frame {
                ExecuteStreamFrame::Start { span_id, .. } if span_id == &record.span_id => {}
                _ => anyhow::bail!("member-turn frame journal does not begin with exact Start"),
            }
        } else if matches!(frame, ExecuteStreamFrame::Start { .. }) {
            anyhow::bail!("member-turn frame journal contains a second Start");
        }
        if previous_was_terminal {
            anyhow::bail!("member-turn frame journal contains a post-terminal frame");
        }
        retained_bytes = retained_bytes
            .checked_add(bytes.len() as u64)
            .ok_or_else(|| anyhow::anyhow!("member-turn retained bytes overflow"))?;
        chain = next_frame_chain_hash(&chain, &bytes)?;
        previous_was_terminal = frame_is_terminal(&frame);
    }
    if retained_bytes != record.retained_bytes || chain != record.frame_chain_sha256 {
        anyhow::bail!("member-turn frame journal commitment mismatch");
    }
    let state_requires_terminal_frame =
        matches!(record.state, MemberTurnJoinStateV1::Completed { .. })
            || (record.last_frame_sequence > 0
                && matches!(
                    record.state,
                    MemberTurnJoinStateV1::LaunchIndeterminate { .. }
                ));
    if previous_was_terminal != state_requires_terminal_frame {
        anyhow::bail!("member-turn terminal frame/state mismatch");
    }
    Ok(())
}

fn adopt_one_published_frame(
    record_directory: &TrustedDirectory,
    record: &mut MemberTurnJoinRecordV1,
) -> anyhow::Result<()> {
    let frames_directory = record_directory.open_directory(FRAMES_DIRECTORY)?;
    let names = frames_directory.entries()?;
    if names.len() as u64 == record.last_frame_sequence {
        return Ok(());
    }
    if names.len() as u64 != record.last_frame_sequence.saturating_add(1) {
        anyhow::bail!("member-turn frame journal has more than one unpublished frame");
    }
    let sequence = record
        .last_frame_sequence
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("member-turn frame sequence exhausted"))?;
    let name = frame_file_name(sequence);
    if names.last() != Some(&name) {
        anyhow::bail!("member-turn unpublished frame name is not contiguous");
    }
    let bytes = frames_directory.open_file(&name)?.read_all()?;
    let frame: ExecuteStreamFrame = serde_json::from_slice(&bytes)?;
    if frame.canonical_ndjson_bytes().map_err(anyhow::Error::msg)? != bytes
        || frame_sequence(&frame) != sequence
        || frame_stream_id(&frame) != record.stream_id
    {
        anyhow::bail!("member-turn unpublished frame bytes or identity changed");
    }
    match &frame {
        ExecuteStreamFrame::Start { span_id, .. }
            if sequence == 1
                && span_id == &record.span_id
                && matches!(
                    record.state,
                    MemberTurnJoinStateV1::LaunchingCallEntered { .. }
                ) => {}
        ExecuteStreamFrame::Start { .. } => {
            anyhow::bail!("member-turn unpublished Start has invalid boundary")
        }
        _ if sequence == 1 || !matches!(record.state, MemberTurnJoinStateV1::Started) => {
            anyhow::bail!("member-turn unpublished post-Start frame has invalid state")
        }
        _ => {}
    }
    record.last_frame_sequence = sequence;
    record.retained_bytes = record
        .retained_bytes
        .checked_add(bytes.len() as u64)
        .ok_or_else(|| anyhow::anyhow!("member-turn retained bytes overflow"))?;
    record.frame_chain_sha256 = next_frame_chain_hash(&record.frame_chain_sha256, &bytes)?;
    record.state = if frame_is_terminal(&frame) {
        let recovery_indeterminate = matches!(
            &frame,
            ExecuteStreamFrame::Error { message, .. }
                if message == "member turn outcome is indeterminate after service restart"
                    || message == "member turn outcome is indeterminate after leader loss"
                    || message == "member turn outcome is indeterminate after launch"
        );
        if recovery_indeterminate {
            MemberTurnJoinStateV1::LaunchIndeterminate {
                reason_code: "recovered_published_indeterminate_frame_v1".to_string(),
            }
        } else {
            MemberTurnJoinStateV1::Completed {
                terminal_frame_sequence: sequence,
            }
        }
    } else {
        MemberTurnJoinStateV1::Started
    };
    record.record_revision = record
        .record_revision
        .checked_add(1)
        .ok_or_else(|| anyhow::anyhow!("member-turn record revision exhausted"))?;
    record.validate(&record.record_id)?;
    replace_json(record_directory, RECORD_FILE, record)?;
    Ok(())
}

fn recognized_temp_name(name: &str) -> bool {
    let Some((target, nonce)) = name
        .strip_prefix('.')
        .and_then(|value| value.strip_suffix(".tmp"))
        .and_then(|value| value.rsplit_once("--"))
    else {
        return false;
    };
    (matches!(target, FORMAT_FILE | KEY_FILE | RECORD_FILE)
        || target
            .strip_suffix(".frame")
            .is_some_and(|digits| digits.len() == 20 && digits.bytes().all(|b| b.is_ascii_digit())))
        && nonce.len() == 32
        && is_lower_hex(nonce)
}

fn decode_exact_json<T>(bytes: &[u8]) -> anyhow::Result<T>
where
    T: for<'de> Deserialize<'de> + Serialize,
{
    let value: T = serde_json::from_slice(bytes)?;
    if serde_json::to_vec(&value)? != bytes {
        anyhow::bail!("member-turn durable JSON is not byte-canonical");
    }
    Ok(value)
}

fn decode_lower_hex_32(value: &str) -> anyhow::Result<[u8; 32]> {
    if value.len() != 64 || !is_lower_hex(value) {
        anyhow::bail!("member-turn digest is not 64 lowercase hexadecimal characters");
    }
    let mut output = [0u8; 32];
    for (index, pair) in value.as_bytes().chunks_exact(2).enumerate() {
        output[index] = (hex_nibble(pair[0])? << 4) | hex_nibble(pair[1])?;
    }
    Ok(output)
}

fn hex_nibble(value: u8) -> anyhow::Result<u8> {
    match value {
        b'0'..=b'9' => Ok(value - b'0'),
        b'a'..=b'f' => Ok(value - b'a' + 10),
        _ => anyhow::bail!("invalid lowercase hexadecimal digit"),
    }
}

fn is_lower_hex(value: &str) -> bool {
    value
        .bytes()
        .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn join_unsafe(error: impl std::fmt::Display) -> MemberTurnJoinError {
    MemberTurnJoinError::Unsafe(error.to_string())
}

fn validate_absolute_path(path: &Path) -> anyhow::Result<PathBuf> {
    if !path.is_absolute() || path.as_os_str().as_bytes().contains(&0) {
        anyhow::bail!("member-turn state root must be an absolute NUL-free path");
    }
    if path
        .components()
        .any(|component| matches!(component, Component::ParentDir | Component::CurDir))
    {
        anyhow::bail!("member-turn state root contains dot traversal");
    }
    Ok(path.to_path_buf())
}

struct TrustedDirectory {
    file: File,
    device: u64,
    owner: libc::uid_t,
}

impl TrustedDirectory {
    fn open_absolute(path: &Path, private: bool) -> anyhow::Result<Self> {
        let mut current = owned_file(
            // SAFETY: the static root pathname and open flags are valid.
            unsafe {
                libc::open(
                    c"/".as_ptr(),
                    libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                )
            },
            "open member-turn physical root",
        )?;
        for component_value in path.components() {
            let Component::Normal(component_value) = component_value else {
                continue;
            };
            let component = c_string(component_value.as_bytes())?;
            let observed = fstatat_nofollow(current.as_raw_fd(), &component)?;
            if observed.st_mode & libc::S_IFMT == libc::S_IFLNK {
                anyhow::bail!("member-turn state-root path contains a symlink");
            }
            current = openat_file(
                current.as_raw_fd(),
                &component,
                libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                0,
            )?;
        }
        let stat = fstat(current.as_raw_fd())?;
        validate_directory_stat(&stat, private)?;
        Ok(Self {
            file: current,
            device: stat.st_dev as u64,
            owner: stat.st_uid,
        })
    }

    fn stat(&self) -> anyhow::Result<libc::stat> {
        fstat(self.file.as_raw_fd())
    }

    fn enforce_private_mode(&self) -> anyhow::Result<()> {
        let stat = self.stat()?;
        validate_directory_stat(&stat, true)
    }

    fn open_or_create_directory(&self, name: &str) -> anyhow::Result<Self> {
        match self.open_directory_if_present(name)? {
            Some(directory) => Ok(directory),
            None => self.create_directory(name),
        }
    }

    fn create_directory(&self, name: &str) -> anyhow::Result<Self> {
        let name = component(name)?;
        // SAFETY: the parent descriptor and validated component are live.
        if unsafe { libc::mkdirat(self.file.as_raw_fd(), name.as_ptr(), PRIVATE_DIRECTORY_MODE) }
            != 0
        {
            return Err(std::io::Error::last_os_error()).context("create member-turn directory");
        }
        self.sync()?;
        self.open_directory_cstr(&name)
    }

    fn open_directory_if_present(&self, name: &str) -> anyhow::Result<Option<Self>> {
        match self.entry_stat(name)? {
            None => Ok(None),
            Some(stat) if stat.st_mode & libc::S_IFMT == libc::S_IFDIR => {
                Ok(Some(self.open_directory(name)?))
            }
            Some(_) => anyhow::bail!("member-turn directory component has unsafe type"),
        }
    }

    fn open_directory(&self, name: &str) -> anyhow::Result<Self> {
        self.open_directory_cstr(&component(name)?)
    }

    fn open_directory_cstr(&self, name: &CStr) -> anyhow::Result<Self> {
        let file = openat_file(
            self.file.as_raw_fd(),
            name,
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )?;
        let stat = fstat(file.as_raw_fd())?;
        validate_directory_stat(&stat, true)?;
        if stat.st_dev as u64 != self.device || stat.st_uid != self.owner {
            anyhow::bail!("member-turn directory crossed device or owner boundary");
        }
        Ok(Self {
            file,
            device: self.device,
            owner: self.owner,
        })
    }

    fn open_or_create_file(&self, name: &str) -> anyhow::Result<TrustedFile> {
        match self.open_file_if_present(name)? {
            Some(file) => Ok(file),
            None => self.create_file(name),
        }
    }

    fn create_file(&self, name: &str) -> anyhow::Result<TrustedFile> {
        let file = openat_file(
            self.file.as_raw_fd(),
            &component(name)?,
            libc::O_RDWR | libc::O_CREAT | libc::O_EXCL | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            PRIVATE_FILE_MODE,
        )?;
        TrustedFile::validated(file, self.device, self.owner)
    }

    fn open_file_if_present(&self, name: &str) -> anyhow::Result<Option<TrustedFile>> {
        match self.entry_stat(name)? {
            None => Ok(None),
            Some(stat) if stat.st_mode & libc::S_IFMT == libc::S_IFREG => {
                Ok(Some(self.open_file(name)?))
            }
            Some(_) => anyhow::bail!("member-turn file component has unsafe type"),
        }
    }

    fn open_file(&self, name: &str) -> anyhow::Result<TrustedFile> {
        let file = openat_file(
            self.file.as_raw_fd(),
            &component(name)?,
            libc::O_RDWR | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )?;
        TrustedFile::validated(file, self.device, self.owner)
    }

    fn entry_stat(&self, name: &str) -> anyhow::Result<Option<libc::stat>> {
        let name = component(name)?;
        let mut stat = MaybeUninit::<libc::stat>::uninit();
        // SAFETY: descriptor/name are live and stat points to writable storage.
        if unsafe {
            libc::fstatat(
                self.file.as_raw_fd(),
                name.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        } != 0
        {
            let error = std::io::Error::last_os_error();
            if error.kind() == std::io::ErrorKind::NotFound {
                return Ok(None);
            }
            return Err(error).context("inspect member-turn entry");
        }
        // SAFETY: fstatat initialized stat on success.
        Ok(Some(unsafe { stat.assume_init() }))
    }

    fn entries(&self) -> anyhow::Result<Vec<String>> {
        let duplicate = openat_file(
            self.file.as_raw_fd(),
            c".",
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
            0,
        )?;
        let fd = std::os::fd::IntoRawFd::into_raw_fd(duplicate);
        // SAFETY: descriptor ownership transfers to fdopendir on success.
        let stream = unsafe { libc::fdopendir(fd) };
        if stream.is_null() {
            // SAFETY: fdopendir did not consume fd on failure.
            unsafe { libc::close(fd) };
            return Err(std::io::Error::last_os_error()).context("enumerate member-turn directory");
        }
        let mut names = Vec::new();
        loop {
            // SAFETY: errno is thread-local. POSIX requires clearing it before
            // readdir so a null return can be distinguished from end-of-stream.
            unsafe { *libc::__errno_location() = 0 };
            // SAFETY: stream is live and owned here.
            let entry = unsafe { libc::readdir(stream) };
            if entry.is_null() {
                // SAFETY: errno remains thread-local and readdir just returned.
                let errno = unsafe { *libc::__errno_location() };
                if errno != 0 {
                    // SAFETY: close the sole owned stream before returning.
                    unsafe { libc::closedir(stream) };
                    return Err(std::io::Error::from_raw_os_error(errno))
                        .context("enumerate member-turn directory");
                }
                break;
            }
            // SAFETY: d_name is NUL-terminated within dirent.
            let name = unsafe { CStr::from_ptr((*entry).d_name.as_ptr()) };
            if matches!(name.to_bytes(), b"." | b"..") {
                continue;
            }
            let name = std::str::from_utf8(name.to_bytes())
                .map_err(|_| anyhow::anyhow!("member-turn entry name is not UTF-8"))?
                .to_string();
            let stat = self
                .entry_stat(&name)?
                .ok_or_else(|| anyhow::anyhow!("member-turn entry disappeared"))?;
            if stat.st_dev as u64 != self.device || stat.st_uid != self.owner {
                // SAFETY: close sole owned directory stream before returning.
                unsafe { libc::closedir(stream) };
                anyhow::bail!("member-turn entry crossed device or owner boundary");
            }
            names.push(name);
        }
        // SAFETY: stream is live and owned here.
        if unsafe { libc::closedir(stream) } != 0 {
            return Err(std::io::Error::last_os_error())
                .context("close member-turn directory enumeration");
        }
        names.sort();
        Ok(names)
    }

    fn rename_no_replace(
        &self,
        source: &str,
        expected: &TrustedFile,
        destination: &str,
    ) -> anyhow::Result<()> {
        self.verify_file_identity(source, expected)?;
        let source = component(source)?;
        let destination = component(destination)?;
        // SAFETY: descriptors and component pointers are valid for renameat2.
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                self.file.as_raw_fd(),
                source.as_ptr(),
                self.file.as_raw_fd(),
                destination.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error())
                .context("publish member-turn file without replacement");
        }
        self.verify_file_identity(destination.to_str()?, expected)?;
        Ok(())
    }

    fn rename_replace(
        &self,
        source: &str,
        expected: &TrustedFile,
        destination: &str,
    ) -> anyhow::Result<()> {
        self.verify_file_identity(source, expected)?;
        let source = component(source)?;
        let destination = component(destination)?;
        // SAFETY: descriptor and component pointers are valid for renameat.
        if unsafe {
            libc::renameat(
                self.file.as_raw_fd(),
                source.as_ptr(),
                self.file.as_raw_fd(),
                destination.as_ptr(),
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error())
                .context("replace member-turn durable file");
        }
        self.verify_file_identity(destination.to_str()?, expected)?;
        Ok(())
    }

    fn rename_directory_no_replace(
        &self,
        source: &str,
        expected: &TrustedDirectory,
        destination: &str,
    ) -> anyhow::Result<()> {
        self.verify_directory_identity(source, expected)?;
        let source = component(source)?;
        let destination = component(destination)?;
        // SAFETY: descriptors and component pointers are valid for renameat2.
        let result = unsafe {
            libc::syscall(
                libc::SYS_renameat2,
                self.file.as_raw_fd(),
                source.as_ptr(),
                self.file.as_raw_fd(),
                destination.as_ptr(),
                libc::RENAME_NOREPLACE,
            )
        };
        if result != 0 {
            return Err(std::io::Error::last_os_error())
                .context("publish member-turn directory without replacement");
        }
        self.verify_directory_identity(destination.to_str()?, expected)
    }

    fn verify_file_identity(&self, name: &str, expected: &TrustedFile) -> anyhow::Result<()> {
        let opened = self.open_file(name)?;
        if opened.device != expected.device || opened.inode != expected.inode {
            anyhow::bail!("member-turn publication file identity changed");
        }
        Ok(())
    }

    fn verify_directory_identity(
        &self,
        name: &str,
        expected: &TrustedDirectory,
    ) -> anyhow::Result<()> {
        let opened = self.open_directory(name)?;
        let opened_stat = opened.stat()?;
        let expected_stat = expected.stat()?;
        if opened_stat.st_dev != expected_stat.st_dev || opened_stat.st_ino != expected_stat.st_ino
        {
            anyhow::bail!("member-turn publication directory identity changed");
        }
        Ok(())
    }

    fn unlink_file(&self, name: &str) -> anyhow::Result<()> {
        // SAFETY: descriptor and validated name select a non-directory entry.
        if unsafe { libc::unlinkat(self.file.as_raw_fd(), component(name)?.as_ptr(), 0) } != 0 {
            return Err(std::io::Error::last_os_error()).context("unlink member-turn temp");
        }
        Ok(())
    }

    fn unlink_verified_file(&self, name: &str, expected: &TrustedFile) -> anyhow::Result<()> {
        self.verify_file_identity(name, expected)?;
        self.unlink_file(name)
    }

    fn remove_empty_directory(
        &self,
        name: &str,
        expected: &TrustedDirectory,
    ) -> anyhow::Result<()> {
        self.verify_directory_identity(name, expected)?;
        // SAFETY: descriptor and validated name select the verified empty directory.
        if unsafe {
            libc::unlinkat(
                self.file.as_raw_fd(),
                component(name)?.as_ptr(),
                libc::AT_REMOVEDIR,
            )
        } != 0
        {
            return Err(std::io::Error::last_os_error())
                .context("remove member-turn temporary directory");
        }
        Ok(())
    }

    fn sync(&self) -> anyhow::Result<()> {
        self.file.sync_all().context("sync member-turn directory")
    }
}

struct TrustedFile {
    file: File,
    device: u64,
    inode: u64,
}

impl TrustedFile {
    fn validated(file: File, device: u64, owner: libc::uid_t) -> anyhow::Result<Self> {
        let stat = fstat(file.as_raw_fd())?;
        if stat.st_mode & libc::S_IFMT != libc::S_IFREG
            || stat.st_dev as u64 != device
            || stat.st_uid != owner
            || stat.st_nlink != 1
            || stat.st_mode & 0o077 != 0
        {
            anyhow::bail!("member-turn trusted file has unsafe owner/type/links/mode");
        }
        Ok(Self {
            file,
            device,
            inode: stat.st_ino as u64,
        })
    }

    fn write_all(&mut self, bytes: &[u8]) -> anyhow::Result<()> {
        self.file
            .write_all(bytes)
            .context("write member-turn durable file")
    }

    fn read_all(&self) -> anyhow::Result<Vec<u8>> {
        let mut bytes = Vec::new();
        (&self.file)
            .read_to_end(&mut bytes)
            .context("read member-turn durable file")?;
        Ok(bytes)
    }

    fn sync(&self) -> anyhow::Result<()> {
        self.file
            .sync_all()
            .context("sync member-turn durable file")
    }

    fn lock_exclusive(&self) -> anyhow::Result<TrustedFileLock<'_>> {
        // SAFETY: descriptor is live and flock accepts LOCK_EX.
        if unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_EX) } != 0 {
            return Err(std::io::Error::last_os_error()).context("lock member-turn registry");
        }
        Ok(TrustedFileLock { file: self })
    }

    fn try_lock_exclusive_owned(&self) -> anyhow::Result<Option<TrustedOwnedFileLock>> {
        let file = self
            .file
            .try_clone()
            .context("duplicate member-turn leader lock descriptor")?;
        // SAFETY: descriptor is live and flock accepts LOCK_EX|LOCK_NB.
        if unsafe { libc::flock(file.as_raw_fd(), libc::LOCK_EX | libc::LOCK_NB) } == 0 {
            return Ok(Some(TrustedOwnedFileLock { file }));
        }
        let error = std::io::Error::last_os_error();
        if error.raw_os_error() == Some(libc::EWOULDBLOCK) {
            return Ok(None);
        }
        Err(error).context("acquire member-turn leader lock")
    }
}

struct TrustedFileLock<'a> {
    file: &'a TrustedFile,
}

impl Drop for TrustedFileLock<'_> {
    fn drop(&mut self) {
        // SAFETY: descriptor is live for the lifetime of this guard.
        unsafe { libc::flock(self.file.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

struct TrustedOwnedFileLock {
    file: File,
}

impl Drop for TrustedOwnedFileLock {
    fn drop(&mut self) {
        // SAFETY: descriptor is live and owns this flock.
        unsafe { libc::flock(self.file.as_raw_fd(), libc::LOCK_UN) };
    }
}

fn validate_directory_stat(stat: &libc::stat, private: bool) -> anyhow::Result<()> {
    // SAFETY: geteuid has no preconditions.
    let effective_uid = unsafe { libc::geteuid() };
    if stat.st_mode & libc::S_IFMT != libc::S_IFDIR
        || stat.st_uid != effective_uid
        || stat.st_mode & 0o022 != 0
        || (private && stat.st_mode & 0o077 != 0)
    {
        anyhow::bail!("member-turn directory has unsafe owner/type/mode");
    }
    Ok(())
}

fn component(value: &str) -> anyhow::Result<CString> {
    if value.is_empty()
        || matches!(value, "." | "..")
        || value.as_bytes().contains(&b'/')
        || value.as_bytes().contains(&0)
    {
        anyhow::bail!("member-turn name is not one safe path component");
    }
    CString::new(value).map_err(Into::into)
}

fn c_string(value: &[u8]) -> anyhow::Result<CString> {
    CString::new(value).map_err(Into::into)
}

fn openat_file(
    parent: RawFd,
    name: &CStr,
    flags: libc::c_int,
    mode: libc::mode_t,
) -> anyhow::Result<File> {
    // SAFETY: descriptor/name are live and mode is used only with O_CREAT.
    owned_file(
        unsafe { libc::openat(parent, name.as_ptr(), flags, mode as libc::c_uint) },
        "open member-turn trusted entry",
    )
}

fn owned_file(fd: RawFd, operation: &str) -> anyhow::Result<File> {
    if fd < 0 {
        return Err(std::io::Error::last_os_error()).context(operation.to_string());
    }
    // SAFETY: fd is newly returned and ownership transfers exactly once.
    Ok(unsafe { File::from_raw_fd(fd) })
}

fn fstat(fd: RawFd) -> anyhow::Result<libc::stat> {
    let mut stat = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: stat points to valid writable storage.
    if unsafe { libc::fstat(fd, stat.as_mut_ptr()) } != 0 {
        return Err(std::io::Error::last_os_error()).context("stat member-turn entry");
    }
    // SAFETY: fstat initialized stat on success.
    Ok(unsafe { stat.assume_init() })
}

fn fstatat_nofollow(parent: RawFd, name: &CStr) -> anyhow::Result<libc::stat> {
    let mut stat = MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptor/name are live and stat points to writable storage.
    if unsafe {
        libc::fstatat(
            parent,
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(std::io::Error::last_os_error()).context("inspect member-turn path");
    }
    // SAFETY: fstatat initialized stat on success.
    Ok(unsafe { stat.assume_init() })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::runtime_replay::RuntimeReplayRegistry;
    use base64::Engine as _;
    use std::os::unix::fs::PermissionsExt as _;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Barrier,
    };
    use transport_api_types::{
        AuthorityObjectKindV1, AuthorityObjectRefV1, DispatchPolicyCommitmentRefCarrierV1,
        DispatchPolicySnapshotCarrierV1, OpaqueAuthorityCommitmentV1, PolicySnapshotV3,
        PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
        RetainedTurnPolicyCommitmentSubjectV1, WorldBindingRefV1, WorldFsDenyEnforcementV3,
        WorldWorkAcceptanceContextV1,
    };

    fn request() -> MemberTurnSubmitRequestV1 {
        let snapshot = PolicySnapshotV3 {
            schema_version: 3,
            net_allowed: Vec::new(),
            world_fs: PolicySnapshotWorldFsV3 {
                host_visible: false,
                fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: true },
                deny_enforcement: Some(WorldFsDenyEnforcementV3::Strict),
                caged_required: true,
                discover: None,
                read: None,
                write: PolicySnapshotWorldFsWriteV3 {
                    enabled: false,
                    allow_list: Vec::new(),
                    deny_list: Vec::new(),
                },
            },
        }
        .canonicalize()
        .expect("canonical snapshot");
        let bytes = serde_json::to_vec(&snapshot).expect("snapshot bytes");
        let message_id = "wwm_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string();
        MemberTurnSubmitRequestV1 {
            schema_version: 1,
            orchestration_session_id: "sess_join_fixture".to_string(),
            participant_id: "worker_join_fixture".to_string(),
            orchestrator_participant_id: "orchestrator_join_fixture".to_string(),
            backend_id: "cli:codex".to_string(),
            run_id: "run_join_fixture".to_string(),
            world_id: "world_join_fixture".to_string(),
            world_generation: 1,
            prompt: "private prompt".to_string(),
            policy_snapshot_carrier: Some(DispatchPolicySnapshotCarrierV1 {
                schema_version: 1,
                immutable_worker_cap_ref: DispatchPolicyCommitmentRefCarrierV1 {
                    authority_store_id: "authority-store-join".to_string(),
                    commitment_id: "dpc_018f0f3a-9b2c-7def-8abc-0123456789ad".to_string(),
                    exact_linkage_hash: "a".repeat(64),
                },
                immutable_worker_cap_created_revision: 7,
                immutable_worker_cap_application_revision: 9,
                subject: RetainedTurnPolicyCommitmentSubjectV1 {
                    retained_participant_id: "worker_join_fixture".to_string(),
                    active_run_id: "run_join_fixture".to_string(),
                    message_id: Some(message_id.clone()),
                },
                orchestration_session_id: "sess_join_fixture".to_string(),
                caller_participant_id: "orchestrator_join_fixture".to_string(),
                caller_backend_id: "cli:codex".to_string(),
                target_backend_id: "cli:codex".to_string(),
                target_world: WorldBindingRefV1 {
                    world_id: "world_join_fixture".to_string(),
                    world_generation: 1,
                },
                policy_snapshot_bytes_base64: base64::engine::general_purpose::STANDARD
                    .encode(&bytes),
                policy_snapshot_byte_length: bytes.len() as u64,
                policy_snapshot_ref: AuthorityObjectRefV1 {
                    ref_id: "ao_0123456789abcdef0123456789abcdef".to_string(),
                    object_kind: AuthorityObjectKindV1::Policy,
                    schema_version: 1,
                    commitment: OpaqueAuthorityCommitmentV1::CanonicalSha256 {
                        digest_hex: "b".repeat(64),
                    },
                },
                policy_snapshot_hash: lowercase_hex(&Sha256::digest(&bytes)),
                policy_snapshot_revision: "policy-revision-join".to_string(),
                reason: Some("authenticated retained turn".to_string()),
            }),
            acceptance_context: Some(WorldWorkAcceptanceContextV1 {
                schema_version: 1,
                proposed_acceptance_record_id: "wwa_018f0f3a-9b2c-7def-8abc-0123456789ab"
                    .to_string(),
                request_id: "run_join_fixture".to_string(),
                message_id: Some(message_id),
                caller_backend_id: "cli:codex".to_string(),
                host_transition_correlation: None,
            }),
        }
    }

    fn non_e2_start(stream_id: &str) -> ExecuteStreamFrame {
        ExecuteStreamFrame::Start {
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: stream_id.to_string(),
                frame_sequence: 1,
            },
            span_id: format!("spn_{stream_id}"),
        }
    }

    #[test]
    fn volatile_and_durable_acceptance_ownership_conflicts_in_both_orders() {
        let volatile_first_root = tempfile::tempdir().expect("volatile-first root");
        let durable =
            MemberTurnJoinRegistry::open(volatile_first_root.path()).expect("open registry");
        let replay = RuntimeReplayRegistry::with_durable_e2(Some(durable.clone()));
        let volatile_request = request();
        let acceptance_id = &volatile_request
            .acceptance_context
            .as_ref()
            .expect("acceptance context")
            .proposed_acceptance_record_id;
        let _publisher = replay
            .begin(acceptance_id, &non_e2_start("rts_volatile_first"))
            .expect("volatile replay owns acceptance first");
        assert!(matches!(
            durable.reserve(&volatile_request),
            Err(MemberTurnJoinError::RequestConflict)
        ));

        let durable_first_root = tempfile::tempdir().expect("durable-first root");
        let durable =
            MemberTurnJoinRegistry::open(durable_first_root.path()).expect("open registry");
        let replay = RuntimeReplayRegistry::with_durable_e2(Some(durable.clone()));
        let request = request();
        let acceptance_id = &request
            .acceptance_context
            .as_ref()
            .expect("acceptance context")
            .proposed_acceptance_record_id;
        durable
            .reserve(&request)
            .expect("durable E2 owns acceptance first");
        let error = match replay.begin(acceptance_id, &non_e2_start("rts_durable_first")) {
            Ok(_) => panic!("volatile replay must reject durable E2 ownership"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("owned by durable E2 authority"));
    }

    #[test]
    fn concurrent_volatile_and_durable_acceptance_claims_have_one_winner() {
        for iteration in 0..32 {
            let root = tempfile::tempdir().expect("acceptance race root");
            let durable = Arc::new(
                MemberTurnJoinRegistry::open(root.path()).expect("open acceptance race registry"),
            );
            let replay = Arc::new(RuntimeReplayRegistry::with_durable_e2(Some(
                durable.as_ref().clone(),
            )));
            let request = request();
            let acceptance_id = request
                .acceptance_context
                .as_ref()
                .expect("acceptance context")
                .proposed_acceptance_record_id
                .clone();
            let barrier = Arc::new(Barrier::new(2));

            let durable_task = {
                let durable = durable.clone();
                let request = request.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    durable.reserve(&request).is_ok()
                })
            };
            let volatile_task = {
                let replay = replay.clone();
                let barrier = barrier.clone();
                std::thread::spawn(move || {
                    barrier.wait();
                    replay
                        .begin(
                            &acceptance_id,
                            &non_e2_start(&format!("rts_acceptance_race_{iteration}")),
                        )
                        .is_ok()
                })
            };

            let durable_won = durable_task.join().expect("durable claimant");
            let volatile_won = volatile_task.join().expect("volatile claimant");
            assert_ne!(
                durable_won, volatile_won,
                "exactly one acceptance namespace claimant must win iteration {iteration}"
            );
        }
    }

    #[test]
    fn stable_key_excludes_changed_material_that_the_request_hmac_conflicts() {
        let key = [0x5au8; 32];
        let request = request();
        request.validate().expect("valid request");
        let stable = MemberTurnStableKeyV1::from_request(&request).expect("stable key");
        let stable_id = stable_key_id(&key, &stable).expect("stable key ID");
        let digest = request_digest(&key, &request).expect("request digest");

        let mut changed = request;
        changed.prompt = "changed private prompt".to_string();
        let changed_stable = MemberTurnStableKeyV1::from_request(&changed).expect("stable key");
        assert_eq!(
            stable_id,
            stable_key_id(&key, &changed_stable).expect("changed stable key ID")
        );
        assert_ne!(
            digest,
            request_digest(&key, &changed).expect("changed request digest")
        );
    }

    #[test]
    fn request_hmac_covers_every_top_level_identity_and_nested_authority_envelope() {
        let key = [0x5au8; 32];
        let original = request();
        let original_digest = request_digest(&key, &original).expect("original request digest");
        let mut mutations = Vec::new();

        let mut changed = original.clone();
        changed.orchestration_session_id = "sess_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .orchestration_session_id = changed.orchestration_session_id.clone();
        mutations.push(("orchestration_session_id", changed));

        let mut changed = original.clone();
        changed.participant_id = "worker_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .subject
            .retained_participant_id = changed.participant_id.clone();
        mutations.push(("participant_id", changed));

        let mut changed = original.clone();
        changed.orchestrator_participant_id = "orchestrator_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .caller_participant_id = changed.orchestrator_participant_id.clone();
        mutations.push(("orchestrator_participant_id", changed));

        let mut changed = original.clone();
        changed.backend_id = "cli:claude-code".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .target_backend_id = changed.backend_id.clone();
        mutations.push(("backend_id", changed));

        let mut changed = original.clone();
        changed.run_id = "run_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .subject
            .active_run_id = changed.run_id.clone();
        changed
            .acceptance_context
            .as_mut()
            .expect("acceptance")
            .request_id = changed.run_id.clone();
        mutations.push(("run_id", changed));

        let mut changed = original.clone();
        changed.world_id = "world_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .target_world
            .world_id = changed.world_id.clone();
        mutations.push(("world_id", changed));

        let mut changed = original.clone();
        changed.world_generation += 1;
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .target_world
            .world_generation = changed.world_generation;
        mutations.push(("world_generation", changed));

        let mut changed = original.clone();
        changed.prompt = "changed private prompt".to_string();
        mutations.push(("prompt", changed));

        let mut changed = original.clone();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .immutable_worker_cap_ref
            .exact_linkage_hash = "c".repeat(64);
        mutations.push(("policy_snapshot_carrier", changed));

        let mut changed = original.clone();
        changed
            .acceptance_context
            .as_mut()
            .expect("acceptance")
            .proposed_acceptance_record_id = "wwa_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string();
        mutations.push(("acceptance_context", changed));

        for (field, changed) in mutations {
            changed
                .validate()
                .unwrap_or_else(|error| panic!("{field} mutation must remain valid: {error}"));
            assert_ne!(
                original_digest,
                request_digest(&key, &changed)
                    .unwrap_or_else(|error| panic!("digest {field} mutation: {error}")),
                "request HMAC omitted {field}"
            );
        }
    }

    #[test]
    fn durable_reservation_exact_joins_and_metadata_never_contains_prompt() {
        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let request = request();
        let first = registry.reserve(&request).expect("first reservation");
        let retry = registry.reserve(&request).expect("exact retry");
        assert_eq!(first.span_id(), retry.span_id());
        assert_eq!(first.stream_id(), retry.stream_id());
        assert_eq!(first.record_id(), retry.record_id());

        let mut changed = request;
        changed.prompt = "changed private prompt".to_string();
        assert!(matches!(
            registry.reserve(&changed),
            Err(MemberTurnJoinError::RequestConflict)
        ));

        let record = root
            .path()
            .join(REGISTRY_DIRECTORY)
            .join(RECORDS_DIRECTORY)
            .join(&first.record_id()[..2])
            .join(&first.record_id()[2..])
            .join(RECORD_FILE);
        let metadata = std::fs::read(record).expect("read durable record metadata");
        assert!(!String::from_utf8_lossy(&metadata).contains("private prompt"));
        assert!(!String::from_utf8_lossy(&metadata).contains("changed private prompt"));
        let key_bytes = std::fs::read(root.path().join(REGISTRY_DIRECTORY).join(KEY_FILE))
            .expect("read private key envelope");
        let key_envelope: MemberTurnDigestKeyEnvelopeV1 =
            decode_exact_json(&key_bytes).expect("decode private key envelope");
        assert!(!String::from_utf8_lossy(&metadata).contains(&key_envelope.secret_key_base64));
    }

    #[test]
    fn existing_only_join_is_hmac_exact_and_never_creates_a_reservation() {
        let root = tempfile::tempdir().expect("member-turn existing-only root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let request = request();
        assert!(registry
            .join_existing(&request)
            .expect("read-only lookup")
            .is_none());
        assert_eq!(
            count_record_directories(&registry.inner.records).expect("count records"),
            0,
            "read-only lookup must not reserve or create durable retry authority"
        );

        let reserved = registry.reserve(&request).expect("publish reservation");
        let joined = registry
            .join_existing(&request)
            .expect("exact existing lookup")
            .expect("durable record exists");
        assert_eq!(joined.record_id(), reserved.record_id());
        assert_eq!(joined.span_id(), reserved.span_id());
        assert_eq!(joined.stream_id(), reserved.stream_id());

        let mut changed = request;
        changed.prompt = "changed private prompt".to_string();
        assert!(matches!(
            registry.join_existing(&changed),
            Err(MemberTurnJoinError::RequestConflict)
        ));
    }

    #[test]
    fn concurrent_distinct_stable_keys_cannot_share_one_acceptance_identity() {
        let root = tempfile::tempdir().expect("member-turn acceptance collision root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let first = request();
        let mut changed = first.clone();
        changed.orchestration_session_id = "sess_join_fixture_changed".to_string();
        changed
            .policy_snapshot_carrier
            .as_mut()
            .expect("carrier")
            .orchestration_session_id = changed.orchestration_session_id.clone();
        changed.validate().expect("changed request remains valid");

        let barrier = Arc::new(std::sync::Barrier::new(3));
        let workers = [first, changed].map(|request| {
            let registry = registry.clone();
            let barrier = Arc::clone(&barrier);
            std::thread::spawn(move || {
                barrier.wait();
                registry.reserve(&request)
            })
        });
        barrier.wait();
        let results = workers.map(|worker| worker.join().expect("join reservation worker"));
        let successful = results.iter().filter(|result| result.is_ok()).count();
        let conflicts = results
            .iter()
            .filter(|result| matches!(result, Err(MemberTurnJoinError::RequestConflict)))
            .count();
        assert_eq!(successful, 1, "only one acceptance owner may reserve");
        assert_eq!(conflicts, 1, "the competing stable key must conflict");

        let reservation = results
            .into_iter()
            .find_map(Result::ok)
            .expect("one durable acceptance owner");
        assert!(registry
            .try_become_leader(&reservation)
            .expect("elect the unique acceptance owner")
            .is_some());
    }

    #[test]
    fn startup_and_exact_join_reject_global_acceptance_stream_and_span_collisions() {
        for collision in ["acceptance", "stream", "span"] {
            let root = tempfile::tempdir().expect("member-turn identity collision root");
            let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
            let first_request = request();
            let first = registry.reserve(&first_request).expect("first reservation");
            let mut second_request = first_request.clone();
            second_request.orchestration_session_id = "sess_join_fixture_collision".to_string();
            second_request
                .policy_snapshot_carrier
                .as_mut()
                .expect("carrier")
                .orchestration_session_id = second_request.orchestration_session_id.clone();
            second_request
                .acceptance_context
                .as_mut()
                .expect("acceptance")
                .proposed_acceptance_record_id =
                "wwa_018f0f3a-9b2c-7def-8abc-0123456789ac".to_string();
            second_request
                .validate()
                .expect("second request remains valid");
            let second = registry
                .reserve(&second_request)
                .expect("second distinct reservation");

            let mut duplicate = second.record.clone();
            match collision {
                "acceptance" => {
                    duplicate.acceptance_record_id = first.record.acceptance_record_id.clone()
                }
                "stream" => duplicate.stream_id = first.record.stream_id.clone(),
                "span" => duplicate.span_id = first.record.span_id.clone(),
                _ => unreachable!(),
            }
            let lock_file = registry
                .inner
                .registry
                .open_file(REGISTRY_LOCK_FILE)
                .expect("open registry lock");
            let _lock = lock_file.lock_exclusive().expect("lock registry");
            let directory = registry
                .open_record_directory(second.record_id())
                .expect("open second record");
            replace_json(&directory, RECORD_FILE, &duplicate).expect("inject collision fixture");
            drop(_lock);

            assert!(
                registry.reserve(&first_request).is_err(),
                "exact join must reject a global {collision} collision"
            );
            drop(registry);
            assert!(
                MemberTurnJoinRegistry::open(root.path()).is_err(),
                "startup must reject a global {collision} collision"
            );
        }
    }

    fn start(reservation: &MemberTurnJoinReservation) -> ExecuteStreamFrame {
        ExecuteStreamFrame::Start {
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: reservation.stream_id().to_string(),
                frame_sequence: 1,
            },
            span_id: reservation.span_id().to_string(),
        }
    }

    fn terminal_error(
        reservation: &MemberTurnJoinReservation,
        sequence: u64,
    ) -> ExecuteStreamFrame {
        ExecuteStreamFrame::Error {
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: reservation.stream_id().to_string(),
                frame_sequence: sequence,
            },
            message: "safe terminal fixture".to_string(),
        }
    }

    #[test]
    fn call_entered_start_and_completed_frames_are_durable_before_delivery_and_replay() {
        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut subscription = registry.subscribe(&reservation, 0).expect("subscribe");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect leader")
            .expect("one leader");
        leader.mark_call_entered().expect("publish CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("publish real Start");
        leader
            .publish_frame(&terminal_error(&reservation, 2))
            .expect("publish terminal");
        assert!(matches!(
            subscription
                .read_next_now()
                .expect("read Start")
                .expect("replayed Start"),
            ExecuteStreamFrame::Start { .. }
        ));
        assert!(matches!(
            subscription
                .read_next_now()
                .expect("read terminal")
                .expect("replayed terminal"),
            ExecuteStreamFrame::Error { .. }
        ));
        drop(leader);
        drop(registry);

        let reopened = MemberTurnJoinRegistry::open(root.path()).expect("reopen registry");
        let retry = reopened.reserve(&request()).expect("exact retry");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::Completed {
                terminal_frame_sequence: 2
            }
        ));
        let mut replay = reopened.subscribe(&retry, 0).expect("completed replay");
        let replayed = [
            replay
                .read_next_now()
                .expect("read Start")
                .expect("Start after restart"),
            replay
                .read_next_now()
                .expect("read terminal")
                .expect("terminal after restart"),
        ];
        assert_eq!(
            replayed
                .iter()
                .map(|frame| frame.canonical_ndjson_bytes().expect("canonical frame"))
                .collect::<Vec<_>>(),
            vec![
                start(&reservation)
                    .canonical_ndjson_bytes()
                    .expect("canonical Start"),
                terminal_error(&reservation, 2)
                    .canonical_ndjson_bytes()
                    .expect("canonical terminal"),
            ]
        );
    }

    #[tokio::test]
    async fn durable_launch_outcome_wait_maps_prestart_failures_without_http_success() {
        let prelaunch_root = tempfile::tempdir().expect("prelaunch root");
        let registry = MemberTurnJoinRegistry::open(prelaunch_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        let waiting = {
            let registry = registry.clone();
            let reservation = reservation.clone();
            tokio::spawn(async move { registry.await_launch_outcome(&reservation).await })
        };
        leader
            .fail_before_launch("safe_prelaunch_failure_v1", 400)
            .expect("publish prelaunch failure");
        assert!(matches!(
            waiting.await.expect("join prelaunch waiter"),
            Err(MemberTurnJoinError::FailedBeforeLaunch {
                error_code,
                http_status: 400,
            }) if error_code == "safe_prelaunch_failure_v1"
        ));

        let indeterminate_root = tempfile::tempdir().expect("indeterminate root");
        let registry =
            MemberTurnJoinRegistry::open(indeterminate_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        let waiting = {
            let registry = registry.clone();
            let reservation = reservation.clone();
            tokio::spawn(async move { registry.await_launch_outcome(&reservation).await })
        };
        leader
            .mark_launch_indeterminate("run_control_failed_after_call_entered_v1")
            .expect("publish indeterminate outcome");
        assert!(matches!(
            waiting.await.expect("join indeterminate waiter"),
            Err(MemberTurnJoinError::LaunchIndeterminate)
        ));
    }

    #[tokio::test]
    async fn independently_opened_registry_observes_durable_launch_and_stream_publication() {
        use futures_util::StreamExt as _;
        use std::time::Duration;

        let root = tempfile::tempdir().expect("cross-instance join root");
        let publisher = MemberTurnJoinRegistry::open(root.path()).expect("open publisher");
        let joiner = MemberTurnJoinRegistry::open(root.path()).expect("open joiner");
        let publisher_reservation = publisher.reserve(&request()).expect("publisher reserve");
        let joiner_reservation = joiner.reserve(&request()).expect("joiner reserve");
        let mut leader = publisher
            .try_become_leader(&publisher_reservation)
            .expect("elect publisher")
            .expect("publisher leader");

        let outcome_waiter = {
            let joiner = joiner.clone();
            let reservation = joiner_reservation.clone();
            tokio::spawn(async move { joiner.await_launch_outcome(&reservation).await })
        };
        let mut stream = joiner
            .subscribe(&joiner_reservation, 0)
            .expect("subscribe through independent registry");
        let stream_waiter = tokio::spawn(async move { stream.next().await });
        tokio::task::yield_now().await;

        leader.mark_call_entered().expect("publish CallEntered");
        leader
            .publish_frame(&start(&publisher_reservation))
            .expect("publish durable Start");

        let state = tokio::time::timeout(Duration::from_secs(1), outcome_waiter)
            .await
            .expect("cross-instance state wait must observe durable publication")
            .expect("join outcome waiter")
            .expect("durable launch outcome");
        assert!(matches!(state, MemberTurnJoinStateV1::Started));
        let frame = tokio::time::timeout(Duration::from_secs(1), stream_waiter)
            .await
            .expect("cross-instance stream tail must observe durable publication")
            .expect("join stream waiter")
            .expect("durable Start frame");
        assert!(matches!(frame, ExecuteStreamFrame::Start { .. }));
    }

    #[test]
    fn startup_recovery_leaves_a_live_cross_instance_leader_unchanged() {
        let root = tempfile::tempdir().expect("live leader recovery root");
        let leader_registry = MemberTurnJoinRegistry::open(root.path()).expect("open leader");
        let reservation = leader_registry.reserve(&request()).expect("reserve");
        let leader = leader_registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("live leader");

        let recovering_registry =
            MemberTurnJoinRegistry::open(root.path()).expect("open concurrent registry");
        let observed = recovering_registry
            .reserve(&request())
            .expect("exact retry through concurrent registry");
        assert!(matches!(
            observed.state(),
            MemberTurnJoinStateV1::LaunchingPreCall { .. }
        ));
        assert!(recovering_registry
            .try_become_leader(&observed)
            .expect("live leader remains authoritative")
            .is_none());

        drop(leader);
    }

    #[test]
    fn restart_recovers_precall_but_never_relaunches_call_entered_or_started() {
        let precall_root = tempfile::tempdir().expect("PreCall root");
        let registry = MemberTurnJoinRegistry::open(precall_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        drop(leader);
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(precall_root.path()).expect("reopen PreCall");
        let retry = reopened.reserve(&request()).expect("retry PreCall");
        assert!(matches!(retry.state(), MemberTurnJoinStateV1::Reserved));
        assert!(reopened
            .try_become_leader(&retry)
            .expect("re-elect")
            .is_some());

        let call_root = tempfile::tempdir().expect("CallEntered root");
        let registry = MemberTurnJoinRegistry::open(call_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        drop(leader);
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(call_root.path()).expect("reopen CallEntered");
        let retry = reopened.reserve(&request()).expect("retry CallEntered");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        assert!(reopened
            .try_become_leader(&retry)
            .expect("no relaunch")
            .is_none());
        drop(reopened);
        let reopened_again =
            MemberTurnJoinRegistry::open(call_root.path()).expect("reopen indeterminate again");
        let retry_again = reopened_again
            .reserve(&request())
            .expect("retry zero-frame indeterminate after second restart");
        assert!(matches!(
            retry_again.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        assert!(reopened_again
            .try_become_leader(&retry_again)
            .expect("second restart remains non-launchable")
            .is_none());

        let started_root = tempfile::tempdir().expect("Started root");
        let registry = MemberTurnJoinRegistry::open(started_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("durable Start");
        drop(leader);
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(started_root.path()).expect("reopen Started");
        let retry = reopened.reserve(&request()).expect("retry Started");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        let mut replay = reopened.subscribe(&retry, 0).expect("indeterminate replay");
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read Start")
                .expect("real Start"),
            ExecuteStreamFrame::Start { .. }
        ));
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read Error")
                .expect("recovery Error"),
            ExecuteStreamFrame::Error { .. }
        ));
    }

    #[test]
    fn thirty_two_concurrent_exact_requests_elect_one_provider_leader() {
        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = Arc::new(MemberTurnJoinRegistry::open(root.path()).expect("open registry"));
        let ready = Arc::new(Barrier::new(32));
        let elected = Arc::new(AtomicUsize::new(0));
        let keep_leader_live = Arc::new(Barrier::new(32));
        let mut threads = Vec::new();
        for _ in 0..32 {
            let registry = registry.clone();
            let ready = ready.clone();
            let elected = elected.clone();
            let keep_leader_live = keep_leader_live.clone();
            threads.push(std::thread::spawn(move || {
                let reservation = registry.reserve(&request()).expect("exact reservation");
                ready.wait();
                let leader = registry
                    .try_become_leader(&reservation)
                    .expect("leader election");
                if leader.is_some() {
                    elected.fetch_add(1, Ordering::SeqCst);
                }
                keep_leader_live.wait();
                drop(leader);
                reservation.record_id().to_string()
            }));
        }
        let record_ids = threads
            .into_iter()
            .map(|thread| thread.join().expect("concurrent requester"))
            .collect::<Vec<_>>();
        assert_eq!(elected.load(Ordering::SeqCst), 1);
        assert!(record_ids.windows(2).all(|ids| ids[0] == ids[1]));
    }

    #[test]
    fn concurrent_changed_material_conflicts_before_leader_election() {
        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = Arc::new(MemberTurnJoinRegistry::open(root.path()).expect("open registry"));
        let ready = Arc::new(Barrier::new(32));
        let successes = Arc::new(AtomicUsize::new(0));
        let conflicts = Arc::new(AtomicUsize::new(0));
        let mut threads = Vec::new();
        for index in 0..32 {
            let registry = registry.clone();
            let ready = ready.clone();
            let successes = successes.clone();
            let conflicts = conflicts.clone();
            threads.push(std::thread::spawn(move || {
                let mut candidate = request();
                if index == 31 {
                    candidate.prompt = "conflicting private prompt".to_string();
                }
                ready.wait();
                match registry.reserve(&candidate) {
                    Ok(_) => {
                        successes.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(MemberTurnJoinError::RequestConflict) => {
                        conflicts.fetch_add(1, Ordering::SeqCst);
                    }
                    Err(error) => panic!("unexpected concurrent outcome: {error}"),
                }
            }));
        }
        for thread in threads {
            thread.join().expect("concurrent requester");
        }
        let success_count = successes.load(Ordering::SeqCst);
        let conflict_count = conflicts.load(Ordering::SeqCst);
        assert_eq!(success_count + conflict_count, 32);
        assert!(matches!((success_count, conflict_count), (31, 1) | (1, 31)));
    }

    #[test]
    fn restart_adopts_a_fully_published_start_and_never_relaunches() {
        use std::os::unix::fs::OpenOptionsExt as _;

        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        let bytes = start(&reservation)
            .canonical_ndjson_bytes()
            .expect("canonical Start");
        let frames = root
            .path()
            .join(REGISTRY_DIRECTORY)
            .join(RECORDS_DIRECTORY)
            .join(&reservation.record_id()[..2])
            .join(&reservation.record_id()[2..])
            .join(FRAMES_DIRECTORY);
        let frame_path = frames.join(frame_file_name(1));
        let mut frame_file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(PRIVATE_FILE_MODE)
            .open(&frame_path)
            .expect("publish frame fixture");
        frame_file.write_all(&bytes).expect("write Start fixture");
        frame_file.sync_all().expect("sync Start fixture");
        File::open(&frames)
            .expect("open frames directory")
            .sync_all()
            .expect("sync frames directory");
        drop(leader);
        drop(registry);

        let reopened = MemberTurnJoinRegistry::open(root.path()).expect("recover registry");
        let retry = reopened.reserve(&request()).expect("exact retry");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        assert!(reopened
            .try_become_leader(&retry)
            .expect("retry decision")
            .is_none());
        let mut replay = reopened.subscribe(&retry, 0).expect("recovery replay");
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read Start")
                .expect("replayed real Start"),
            ExecuteStreamFrame::Start { .. }
        ));
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read Error")
                .expect("replayed recovery Error"),
            ExecuteStreamFrame::Error { .. }
        ));
    }

    #[test]
    fn recognized_reservation_temp_is_removed_but_unknown_residue_fails_closed() {
        let root = tempfile::tempdir().expect("member-turn join root");
        let registry = MemberTurnJoinRegistry::open(root.path()).expect("open registry");
        let record_id = stable_key_id(
            &registry.inner.digest_key,
            &MemberTurnStableKeyV1::from_request(&request()).expect("stable key"),
        )
        .expect("record ID");
        drop(registry);
        let shard = root
            .path()
            .join(REGISTRY_DIRECTORY)
            .join(RECORDS_DIRECTORY)
            .join(&record_id[..2]);
        std::fs::create_dir(&shard).expect("create shard");
        std::fs::set_permissions(&shard, std::fs::Permissions::from_mode(0o700))
            .expect("private shard");
        let temp_name = format!(".{}--{}.tmp", &record_id[2..], "a".repeat(32));
        let temp = shard.join(&temp_name);
        std::fs::create_dir(&temp).expect("create recognized temp");
        std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o700))
            .expect("private temp");
        MemberTurnJoinRegistry::open(root.path()).expect("reconcile recognized temp");
        assert!(!temp.exists());

        std::fs::write(shard.join("unknown"), b"unsafe residue").expect("unknown residue");
        assert!(MemberTurnJoinRegistry::open(root.path()).is_err());
    }

    #[test]
    fn same_process_retry_recovers_released_leader_without_stalling() {
        let pre_call_root = tempfile::tempdir().expect("PreCall root");
        let registry = MemberTurnJoinRegistry::open(pre_call_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        drop(leader);
        assert!(matches!(
            registry
                .recover_orphaned(&reservation)
                .expect("recover released PreCall leader"),
            MemberTurnJoinStateV1::Reserved
        ));
        assert!(registry
            .try_become_leader(&reservation)
            .expect("retry leader decision")
            .is_some());

        let call_entered_root = tempfile::tempdir().expect("CallEntered root");
        let registry =
            MemberTurnJoinRegistry::open(call_entered_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        drop(leader);
        assert!(matches!(
            registry
                .recover_orphaned(&reservation)
                .expect("recover released CallEntered leader"),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        assert!(registry
            .try_become_leader(&reservation)
            .expect("post-call retry decision")
            .is_none());

        let started_root = tempfile::tempdir().expect("Started root");
        let registry = MemberTurnJoinRegistry::open(started_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("publish real Start");
        drop(leader);
        assert!(matches!(
            registry
                .recover_orphaned(&reservation)
                .expect("recover released Started leader"),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        let mut replay = registry
            .subscribe(&reservation, 0)
            .expect("subscribe recovered Started turn");
        assert!(matches!(
            replay.read_next_now().expect("read Start"),
            Some(ExecuteStreamFrame::Start { .. })
        ));
        assert!(matches!(
            replay.read_next_now().expect("read recovery Error"),
            Some(ExecuteStreamFrame::Error { .. })
        ));
    }

    #[test]
    fn live_frame_metadata_split_is_reconciled_before_indeterminate() {
        let start_root = tempfile::tempdir().expect("Start split root");
        let registry = MemberTurnJoinRegistry::open(start_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        publish_uncommitted_frame(start_root.path(), &reservation, &start(&reservation));
        leader
            .mark_launch_indeterminate("start_persistence_failed_v1")
            .expect("reconcile durable Start before indeterminate");
        let recovered = registry
            .load_record(reservation.record_id())
            .expect("load reconciled Start record");
        assert_eq!(recovered.last_frame_sequence, 2);
        assert!(matches!(
            recovered.state,
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));

        let event_root = tempfile::tempdir().expect("event split root");
        let registry = MemberTurnJoinRegistry::open(event_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("publish Start");
        let event = ExecuteStreamFrame::Stdout {
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: reservation.stream_id().to_string(),
                frame_sequence: 2,
            },
            chunk_b64: "b2s=".to_string(),
        };
        publish_uncommitted_frame(event_root.path(), &reservation, &event);
        leader
            .mark_launch_indeterminate("frame_persistence_failed_v1")
            .expect("reconcile durable event before indeterminate");
        let recovered = registry
            .load_record(reservation.record_id())
            .expect("load reconciled event record");
        assert_eq!(recovered.last_frame_sequence, 3);
        assert!(matches!(
            recovered.state,
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        validate_frame_journal(
            &registry
                .open_record_directory(reservation.record_id())
                .expect("open reconciled record"),
            &recovered,
        )
        .expect("reconciled journal remains valid");
    }

    #[test]
    fn exact_retry_on_already_open_registry_adopts_frame_metadata_split() {
        let root = tempfile::tempdir().expect("exact retry split root");
        let leader_registry = MemberTurnJoinRegistry::open(root.path()).expect("open leader");
        let reservation = leader_registry.reserve(&request()).expect("reserve");
        let mut leader = leader_registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        let retry_registry =
            MemberTurnJoinRegistry::open(root.path()).expect("open retry registry before split");

        publish_uncommitted_frame(root.path(), &reservation, &start(&reservation));
        let retry = retry_registry
            .join_existing(&request())
            .expect("exact production retry lookup")
            .expect("existing reservation");
        assert_eq!(retry.record.last_frame_sequence, 1);
        assert!(matches!(retry.state(), MemberTurnJoinStateV1::Started));
        let mut replay = retry_registry
            .subscribe(&retry, 0)
            .expect("subscribe exact retry");
        assert!(matches!(
            replay.read_next_now().expect("read durable Start"),
            Some(ExecuteStreamFrame::Start { .. })
        ));

        drop(leader);
        assert!(matches!(
            retry_registry
                .recover_orphaned(&retry)
                .expect("recover orphaned exact retry"),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
    }

    #[test]
    fn recognized_file_temps_require_verified_private_regular_inodes() {
        fn registry_temp(root: &Path) -> PathBuf {
            root.join(REGISTRY_DIRECTORY)
                .join(format!(".{FORMAT_FILE}--{}.tmp", "a".repeat(32)))
        }

        let regular_root = tempfile::tempdir().expect("regular temp root");
        let registry = MemberTurnJoinRegistry::open(regular_root.path()).expect("open registry");
        drop(registry);
        let regular = registry_temp(regular_root.path());
        std::fs::write(&regular, b"recognized private temp").expect("write regular temp");
        std::fs::set_permissions(&regular, std::fs::Permissions::from_mode(PRIVATE_FILE_MODE))
            .expect("secure regular temp");
        MemberTurnJoinRegistry::open(regular_root.path()).expect("remove verified regular temp");
        assert!(!regular.exists());

        let symlink_root = tempfile::tempdir().expect("symlink temp root");
        let registry = MemberTurnJoinRegistry::open(symlink_root.path()).expect("open registry");
        drop(registry);
        std::os::unix::fs::symlink(FORMAT_FILE, registry_temp(symlink_root.path()))
            .expect("create recognized-name symlink");
        assert!(MemberTurnJoinRegistry::open(symlink_root.path()).is_err());

        let hard_link_root = tempfile::tempdir().expect("hard-link temp root");
        let registry = MemberTurnJoinRegistry::open(hard_link_root.path()).expect("open registry");
        drop(registry);
        let outside = hard_link_root.path().join("hard-link-source");
        std::fs::write(&outside, b"recognized hard-link temp").expect("write hard-link source");
        std::fs::set_permissions(&outside, std::fs::Permissions::from_mode(PRIVATE_FILE_MODE))
            .expect("secure hard-link source");
        std::fs::hard_link(&outside, registry_temp(hard_link_root.path()))
            .expect("create recognized-name hard link");
        assert!(MemberTurnJoinRegistry::open(hard_link_root.path()).is_err());

        let fifo_root = tempfile::tempdir().expect("fifo temp root");
        let registry = MemberTurnJoinRegistry::open(fifo_root.path()).expect("open registry");
        drop(registry);
        let fifo = registry_temp(fifo_root.path());
        let fifo_c = CString::new(fifo.as_os_str().as_bytes()).expect("fifo path");
        // SAFETY: fifo_c is a valid NUL-terminated path and the parent is test-owned.
        assert_eq!(
            unsafe { libc::mkfifo(fifo_c.as_ptr(), PRIVATE_FILE_MODE) },
            0
        );
        assert!(MemberTurnJoinRegistry::open(fifo_root.path()).is_err());
    }

    fn publish_uncommitted_frame(
        root: &Path,
        reservation: &MemberTurnJoinReservation,
        frame: &ExecuteStreamFrame,
    ) {
        use std::os::unix::fs::OpenOptionsExt as _;

        let frames = root
            .join(REGISTRY_DIRECTORY)
            .join(RECORDS_DIRECTORY)
            .join(&reservation.record_id()[..2])
            .join(&reservation.record_id()[2..])
            .join(FRAMES_DIRECTORY);
        let bytes = frame
            .canonical_ndjson_bytes()
            .expect("canonical crash-boundary frame");
        let mut file = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .mode(PRIVATE_FILE_MODE)
            .open(frames.join(frame_file_name(frame_sequence(frame))))
            .expect("publish crash-boundary frame");
        file.write_all(&bytes).expect("write crash-boundary frame");
        file.sync_all().expect("sync crash-boundary frame");
        File::open(frames)
            .expect("open crash-boundary frames directory")
            .sync_all()
            .expect("sync crash-boundary frames directory");
    }

    #[test]
    fn every_launch_and_frame_publication_crash_boundary_recovers_fail_closed() {
        let before_reservation = tempfile::tempdir().expect("before-reservation root");
        let registry = MemberTurnJoinRegistry::open(before_reservation.path()).expect("open");
        let reservation = registry
            .reserve(&request())
            .expect("first writer after no reservation");
        assert!(matches!(
            reservation.state(),
            MemberTurnJoinStateV1::Reserved
        ));

        let after_reservation = tempfile::tempdir().expect("after-reservation root");
        let registry = MemberTurnJoinRegistry::open(after_reservation.path()).expect("open");
        let original = registry.reserve(&request()).expect("reserve");
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(after_reservation.path()).expect("reopen");
        let retry = reopened.reserve(&request()).expect("join reservation");
        assert_eq!(retry.span_id(), original.span_id());
        assert_eq!(retry.stream_id(), original.stream_id());
        assert!(matches!(retry.state(), MemberTurnJoinStateV1::Reserved));

        let after_precall = tempfile::tempdir().expect("PreCall root");
        let registry = MemberTurnJoinRegistry::open(after_precall.path()).expect("open");
        let reservation = registry.reserve(&request()).expect("reserve");
        let leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        drop(leader);
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(after_precall.path()).expect("recover PreCall");
        let retry = reopened.reserve(&request()).expect("retry PreCall");
        assert!(matches!(retry.state(), MemberTurnJoinStateV1::Reserved));

        for label in ["after_call_entered", "after_run_control_return"] {
            let root = tempfile::tempdir().expect("post-call root");
            let registry = MemberTurnJoinRegistry::open(root.path()).expect("open");
            let reservation = registry.reserve(&request()).expect("reserve");
            let mut leader = registry
                .try_become_leader(&reservation)
                .expect("elect")
                .expect("leader");
            leader.mark_call_entered().expect("CallEntered");
            drop(leader);
            drop(registry);
            let reopened = MemberTurnJoinRegistry::open(root.path()).expect("recover post-call");
            let retry = reopened.reserve(&request()).expect("retry post-call");
            assert!(
                matches!(
                    retry.state(),
                    MemberTurnJoinStateV1::LaunchIndeterminate { .. }
                ),
                "{label} must never authorize a relaunch"
            );
            assert!(reopened
                .try_become_leader(&retry)
                .expect("post-call retry decision")
                .is_none());
        }

        let after_start_rename = tempfile::tempdir().expect("Start rename root");
        let registry = MemberTurnJoinRegistry::open(after_start_rename.path()).expect("open");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        publish_uncommitted_frame(
            after_start_rename.path(),
            &reservation,
            &start(&reservation),
        );
        drop(leader);
        drop(registry);
        let reopened =
            MemberTurnJoinRegistry::open(after_start_rename.path()).expect("recover Start rename");
        let retry = reopened.reserve(&request()).expect("retry Start rename");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        let mut replay = reopened
            .subscribe(&retry, 0)
            .expect("replay recovered Start");
        assert!(matches!(
            replay.read_next_now().expect("read").expect("Start"),
            ExecuteStreamFrame::Start { .. }
        ));
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read")
                .expect("recovery Error"),
            ExecuteStreamFrame::Error { .. }
        ));

        let after_nonterminal_rename = tempfile::tempdir().expect("event rename root");
        let registry = MemberTurnJoinRegistry::open(after_nonterminal_rename.path()).expect("open");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("persist Start and Started metadata");
        let nonterminal = ExecuteStreamFrame::Stdout {
            frame_identity: RuntimeFrameIdentityV1 {
                schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
                stream_id: reservation.stream_id().to_string(),
                frame_sequence: 2,
            },
            chunk_b64: "b2s=".to_string(),
        };
        publish_uncommitted_frame(after_nonterminal_rename.path(), &reservation, &nonterminal);
        drop(leader);
        drop(registry);
        let reopened = MemberTurnJoinRegistry::open(after_nonterminal_rename.path())
            .expect("recover nonterminal rename");
        let retry = reopened
            .reserve(&request())
            .expect("retry nonterminal rename");
        assert!(matches!(
            retry.state(),
            MemberTurnJoinStateV1::LaunchIndeterminate { .. }
        ));
        let mut replay = reopened
            .subscribe(&retry, 0)
            .expect("replay event recovery");
        assert!(matches!(
            replay.read_next_now().expect("read").expect("Start"),
            ExecuteStreamFrame::Start { .. }
        ));
        assert_eq!(
            replay
                .read_next_now()
                .expect("read")
                .expect("event")
                .canonical_ndjson_bytes()
                .expect("canonical"),
            nonterminal.canonical_ndjson_bytes().expect("canonical")
        );
        assert!(matches!(
            replay
                .read_next_now()
                .expect("read")
                .expect("recovery Error"),
            ExecuteStreamFrame::Error { .. }
        ));

        for terminal_metadata_published in [false, true] {
            let root = tempfile::tempdir().expect("terminal root");
            let registry = MemberTurnJoinRegistry::open(root.path()).expect("open");
            let reservation = registry.reserve(&request()).expect("reserve");
            let mut leader = registry
                .try_become_leader(&reservation)
                .expect("elect")
                .expect("leader");
            leader.mark_call_entered().expect("CallEntered");
            leader
                .publish_frame(&start(&reservation))
                .expect("persist Start");
            let terminal = terminal_error(&reservation, 2);
            if terminal_metadata_published {
                leader.publish_frame(&terminal).expect("persist Completed");
            } else {
                publish_uncommitted_frame(root.path(), &reservation, &terminal);
            }
            drop(leader);
            drop(registry);
            let reopened = MemberTurnJoinRegistry::open(root.path()).expect("recover terminal");
            let retry = reopened.reserve(&request()).expect("retry terminal");
            assert!(matches!(
                retry.state(),
                MemberTurnJoinStateV1::Completed {
                    terminal_frame_sequence: 2
                }
            ));
            assert!(reopened
                .try_become_leader(&retry)
                .expect("completed retry decision")
                .is_none());
        }
    }

    #[test]
    fn trusted_root_permissions_symlinks_hardlinks_and_replacement_fail_closed() {
        let symlink_container = tempfile::tempdir().expect("symlink container");
        let physical = symlink_container.path().join("physical");
        std::fs::create_dir(&physical).expect("create physical root");
        std::fs::set_permissions(&physical, std::fs::Permissions::from_mode(0o700))
            .expect("secure physical root");
        let alias = symlink_container.path().join("alias");
        std::os::unix::fs::symlink(&physical, &alias).expect("create root symlink");
        assert!(MemberTurnJoinRegistry::open(&alias).is_err());

        let mode_root = tempfile::tempdir().expect("mode root");
        let registry = MemberTurnJoinRegistry::open(mode_root.path()).expect("open mode root");
        drop(registry);
        let key = mode_root.path().join(REGISTRY_DIRECTORY).join(KEY_FILE);
        std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o640))
            .expect("make key group-readable");
        assert!(MemberTurnJoinRegistry::open(mode_root.path()).is_err());

        let hardlink_root = tempfile::tempdir().expect("hard-link root");
        let registry = MemberTurnJoinRegistry::open(hardlink_root.path()).expect("open hard-link");
        drop(registry);
        let registry_path = hardlink_root.path().join(REGISTRY_DIRECTORY);
        std::fs::hard_link(
            registry_path.join(KEY_FILE),
            registry_path.join("digest-key-hardlink"),
        )
        .expect("create key hard link");
        assert!(MemberTurnJoinRegistry::open(hardlink_root.path()).is_err());

        let replacement_container = tempfile::tempdir().expect("replacement container");
        let state_root = replacement_container.path().join("state");
        let displaced = replacement_container.path().join("displaced");
        std::fs::create_dir(&state_root).expect("create state root");
        std::fs::set_permissions(&state_root, std::fs::Permissions::from_mode(0o700))
            .expect("secure state root");
        let registry = MemberTurnJoinRegistry::open(&state_root).expect("open replacement root");
        std::fs::rename(&state_root, &displaced).expect("displace state root");
        std::fs::create_dir(&state_root).expect("create replacement state root");
        std::fs::set_permissions(&state_root, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement root");
        assert!(matches!(
            registry.reserve(&request()),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
    }

    #[test]
    fn root_replacement_after_reservation_blocks_call_entered_and_frame_publication() {
        let container = tempfile::tempdir().expect("replacement container");
        let state_root = container.path().join("state");
        let displaced = container.path().join("displaced");
        std::fs::create_dir(&state_root).expect("create state root");
        std::fs::set_permissions(&state_root, std::fs::Permissions::from_mode(0o700))
            .expect("secure state root");
        let registry = MemberTurnJoinRegistry::open(&state_root).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");

        std::fs::rename(&state_root, &displaced).expect("displace state root");
        std::fs::create_dir(&state_root).expect("create replacement state root");
        std::fs::set_permissions(&state_root, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement state root");

        assert!(matches!(
            leader.mark_call_entered(),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
        let record_path = displaced
            .join(REGISTRY_DIRECTORY)
            .join(RECORDS_DIRECTORY)
            .join(&reservation.record_id()[..2])
            .join(&reservation.record_id()[2..]);
        let record: MemberTurnJoinRecordV1 = decode_exact_json(
            &std::fs::read(record_path.join(RECORD_FILE)).expect("read displaced record"),
        )
        .expect("decode displaced record");
        assert!(matches!(
            record.state,
            MemberTurnJoinStateV1::LaunchingPreCall { .. }
        ));
        assert_eq!(record.last_frame_sequence, 0);
        assert_eq!(
            std::fs::read_dir(record_path.join(FRAMES_DIRECTORY))
                .expect("read displaced frame journal")
                .count(),
            0,
            "no Start or terminal frame may be published after root replacement"
        );
        assert!(matches!(
            registry.reserve(&request()),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
    }

    #[test]
    fn registry_subtree_replacement_after_reservation_blocks_call_entered_and_frames() {
        let state_root = tempfile::tempdir().expect("state root");
        let registry = MemberTurnJoinRegistry::open(state_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        let registry_path = state_root.path().join(REGISTRY_DIRECTORY);
        let displaced = state_root.path().join("displaced-member-turn-join-v1");
        std::fs::rename(&registry_path, &displaced).expect("displace registry subtree");
        std::fs::create_dir(&registry_path).expect("create replacement registry subtree");
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement registry subtree");

        assert!(matches!(
            leader.mark_call_entered(),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
        let record_path = displaced
            .join(RECORDS_DIRECTORY)
            .join(&reservation.record_id()[..2])
            .join(&reservation.record_id()[2..]);
        let record: MemberTurnJoinRecordV1 = decode_exact_json(
            &std::fs::read(record_path.join(RECORD_FILE)).expect("read displaced record"),
        )
        .expect("decode displaced record");
        assert!(matches!(
            record.state,
            MemberTurnJoinStateV1::LaunchingPreCall { .. }
        ));
        assert_eq!(record.last_frame_sequence, 0);
        assert_eq!(
            std::fs::read_dir(record_path.join(FRAMES_DIRECTORY))
                .expect("read displaced frame journal")
                .count(),
            0,
            "no frame may be published through a displaced registry subtree"
        );
    }

    #[test]
    fn registry_subtree_replacement_after_call_entered_blocks_start_publication() {
        let state_root = tempfile::tempdir().expect("state root");
        let registry = MemberTurnJoinRegistry::open(state_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");

        let registry_path = state_root.path().join(REGISTRY_DIRECTORY);
        let displaced = state_root.path().join("displaced-member-turn-join-v1");
        std::fs::rename(&registry_path, &displaced).expect("displace registry subtree");
        std::fs::create_dir(&registry_path).expect("create replacement registry subtree");
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement registry subtree");

        assert!(matches!(
            leader.publish_frame(&start(&reservation)),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
        let record_path = displaced
            .join(RECORDS_DIRECTORY)
            .join(&reservation.record_id()[..2])
            .join(&reservation.record_id()[2..]);
        let record: MemberTurnJoinRecordV1 = decode_exact_json(
            &std::fs::read(record_path.join(RECORD_FILE)).expect("read displaced record"),
        )
        .expect("decode displaced record");
        assert!(matches!(
            record.state,
            MemberTurnJoinStateV1::LaunchingCallEntered { .. }
        ));
        assert_eq!(record.last_frame_sequence, 0);
        assert_eq!(
            std::fs::read_dir(record_path.join(FRAMES_DIRECTORY))
                .expect("read displaced frame journal")
                .count(),
            0,
            "real Start must not be written after registry-subtree replacement"
        );
    }

    #[test]
    fn registry_subtree_replacement_blocks_acceptance_claim_replay_and_live_polling() {
        let state_root = tempfile::tempdir().expect("state root");
        let registry = MemberTurnJoinRegistry::open(state_root.path()).expect("open registry");
        let reservation = registry.reserve(&request()).expect("reserve");
        let mut leader = registry
            .try_become_leader(&reservation)
            .expect("elect")
            .expect("leader");
        leader.mark_call_entered().expect("publish CallEntered");
        leader
            .publish_frame(&start(&reservation))
            .expect("publish real Start");
        let mut live = registry
            .subscribe_for_replay(
                &reservation.record.acceptance_record_id,
                reservation.stream_id(),
                0,
            )
            .expect("initial replay lookup")
            .expect("durable replay exists");
        assert!(matches!(
            live.read_next_now().expect("read durable Start"),
            Some(ExecuteStreamFrame::Start { .. })
        ));

        let registry_path = state_root.path().join(REGISTRY_DIRECTORY);
        let displaced = state_root.path().join("displaced-member-turn-join-v1");
        std::fs::rename(&registry_path, &displaced).expect("displace registry subtree");
        std::fs::create_dir(&registry_path).expect("create replacement registry subtree");
        std::fs::set_permissions(&registry_path, std::fs::Permissions::from_mode(0o700))
            .expect("secure replacement registry subtree");

        assert!(matches!(
            registry.claims_acceptance_record_id(&reservation.record.acceptance_record_id),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
        assert!(matches!(
            registry.subscribe_for_replay(
                &reservation.record.acceptance_record_id,
                reservation.stream_id(),
                0,
            ),
            Err(MemberTurnJoinError::Unsafe(_))
        ));
        assert!(live.read_next_now().is_err());
    }
}
