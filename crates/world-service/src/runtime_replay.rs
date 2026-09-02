use std::{
    collections::BTreeMap,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
};

use anyhow::{Context as _, Result};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use transport_api_types::{
    ExecuteStreamFrame, ExecuteStreamReplayRequestV1, WorldWorkAcceptanceContextV1,
};

use crate::member_turn_join::{MemberTurnAcceptanceNamespace, MemberTurnJoinRegistry};

const MAX_REPLAY_STREAMS: usize = 1_024;
const MAX_REPLAY_FRAMES_PER_STREAM: usize = 65_536;
const MAX_REPLAY_BYTES_PER_STREAM: usize = 64 * 1024 * 1024;
const MAX_REPLAY_SUBSCRIPTIONS_PER_STREAM: usize = 8;

#[derive(Clone, Default)]
pub(crate) struct RuntimeReplayRegistry {
    streams: Arc<Mutex<BTreeMap<String, Arc<Mutex<RuntimeReplayStreamState>>>>>,
    durable_e2: Option<MemberTurnJoinRegistry>,
    acceptance_namespace: MemberTurnAcceptanceNamespace,
}

struct RuntimeReplayStreamState {
    stream_id: String,
    frames: Vec<ExecuteStreamFrame>,
    retained_bytes: usize,
    producer_closed: bool,
    replay_unavailable: bool,
    active_subscriptions: usize,
    subscribers: Vec<UnboundedSender<ExecuteStreamFrame>>,
}

#[derive(Clone)]
pub(crate) struct RuntimeReplayPublisher {
    stream: Arc<Mutex<RuntimeReplayStreamState>>,
}

pub(crate) struct RuntimeReplaySubscription {
    receiver: UnboundedReceiver<ExecuteStreamFrame>,
    stream: Arc<Mutex<RuntimeReplayStreamState>>,
}

impl RuntimeReplayRegistry {
    pub(crate) fn with_durable_e2(durable_e2: Option<MemberTurnJoinRegistry>) -> Self {
        let acceptance_namespace = durable_e2
            .as_ref()
            .map(MemberTurnJoinRegistry::acceptance_namespace)
            .unwrap_or_default();
        Self {
            durable_e2,
            acceptance_namespace,
            streams: Arc::default(),
        }
    }

    pub(crate) fn begin_for_acceptance(
        &self,
        acceptance_context: Option<&WorldWorkAcceptanceContextV1>,
        start: &ExecuteStreamFrame,
    ) -> Result<Option<RuntimeReplayPublisher>> {
        acceptance_context
            .map(|context| self.begin(&context.proposed_acceptance_record_id, start))
            .transpose()
    }

    pub(crate) fn begin(
        &self,
        acceptance_record_id: &str,
        start: &ExecuteStreamFrame,
    ) -> Result<RuntimeReplayPublisher> {
        validate_acceptance_record_id(acceptance_record_id)?;
        let ExecuteStreamFrame::Start { frame_identity, .. } = start else {
            anyhow::bail!("runtime replay stream must begin with exact Start");
        };
        if frame_identity.frame_sequence != 1 {
            anyhow::bail!("runtime replay Start must use frame sequence 1");
        }
        let canonical = start
            .canonical_ndjson_bytes()
            .map_err(anyhow::Error::msg)
            .context("canonicalize runtime replay Start")?;
        let mut acceptance_namespace = self
            .acceptance_namespace
            .lock()
            .map_err(anyhow::Error::new)?;
        if acceptance_namespace.contains(acceptance_record_id) {
            anyhow::bail!("runtime replay acceptance identity is already registered");
        }
        if let Some(registry) = self.durable_e2.as_ref() {
            if registry
                .claims_acceptance_record_id(acceptance_record_id)
                .map_err(anyhow::Error::new)?
            {
                anyhow::bail!(
                    "runtime replay acceptance identity is owned by durable E2 authority"
                );
            }
        }
        let mut streams = self
            .streams
            .lock()
            .map_err(|_| anyhow::anyhow!("runtime replay registry lock poisoned"))?;
        if streams.contains_key(acceptance_record_id) {
            anyhow::bail!("runtime replay acceptance identity is already registered");
        }
        if streams.len() >= MAX_REPLAY_STREAMS {
            let evictable = streams.iter().find_map(|(record_id, stream)| {
                stream
                    .lock()
                    .ok()
                    .and_then(|state| state.producer_closed.then(|| record_id.clone()))
            });
            match evictable {
                Some(record_id) => {
                    streams.remove(&record_id);
                    acceptance_namespace.remove(&record_id);
                }
                None => anyhow::bail!("runtime replay registry capacity is exhausted"),
            }
        }
        let stream = Arc::new(Mutex::new(RuntimeReplayStreamState {
            stream_id: frame_identity.stream_id.clone(),
            frames: vec![start.clone()],
            retained_bytes: canonical.len(),
            producer_closed: false,
            replay_unavailable: false,
            active_subscriptions: 0,
            subscribers: Vec::new(),
        }));
        streams.insert(acceptance_record_id.to_string(), stream.clone());
        acceptance_namespace.insert(acceptance_record_id.to_string());
        Ok(RuntimeReplayPublisher { stream })
    }

    pub(crate) fn subscribe(
        &self,
        request: &ExecuteStreamReplayRequestV1,
    ) -> Result<RuntimeReplaySubscription> {
        request.validate().map_err(anyhow::Error::msg)?;
        let stream = self
            .streams
            .lock()
            .map_err(|_| anyhow::anyhow!("runtime replay registry lock poisoned"))?
            .get(&request.acceptance_record_id)
            .cloned()
            .ok_or_else(|| anyhow::anyhow!("exact runtime replay stream is unavailable"))?;
        let mut state = stream
            .lock()
            .map_err(|_| anyhow::anyhow!("runtime replay stream lock poisoned"))?;
        if state.replay_unavailable {
            anyhow::bail!("exact runtime replay stream is unavailable");
        }
        if state.stream_id != request.stream_id {
            anyhow::bail!("runtime replay stream identity mismatch");
        }
        state
            .subscribers
            .retain(|subscriber| !subscriber.is_closed());
        if state.active_subscriptions >= MAX_REPLAY_SUBSCRIPTIONS_PER_STREAM {
            anyhow::bail!("runtime replay subscription capacity is exhausted");
        }
        let durable_end = state.frames.last().map(frame_sequence).unwrap_or_default();
        if request.after_frame_sequence > durable_end {
            anyhow::bail!("runtime replay cursor is ahead of the producer journal");
        }
        let (tx, rx) = mpsc::unbounded_channel();
        for frame in state
            .frames
            .iter()
            .filter(|frame| frame_sequence(frame) > request.after_frame_sequence)
        {
            tx.send(frame.clone())
                .map_err(|_| anyhow::anyhow!("runtime replay subscriber closed during replay"))?;
        }
        if !state.producer_closed {
            state.subscribers.push(tx);
        }
        state.active_subscriptions += 1;
        drop(state);
        Ok(RuntimeReplaySubscription {
            receiver: rx,
            stream,
        })
    }
}

impl RuntimeReplaySubscription {
    #[cfg(test)]
    pub(crate) fn try_recv(
        &mut self,
    ) -> std::result::Result<ExecuteStreamFrame, mpsc::error::TryRecvError> {
        self.receiver.try_recv()
    }
}

impl futures_util::Stream for RuntimeReplaySubscription {
    type Item = ExecuteStreamFrame;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.receiver.poll_recv(cx)
    }
}

impl Drop for RuntimeReplaySubscription {
    fn drop(&mut self) {
        if let Ok(mut state) = self.stream.lock() {
            state.active_subscriptions = state.active_subscriptions.saturating_sub(1);
        }
    }
}

impl RuntimeReplayPublisher {
    pub(crate) fn publish(&self, frame: &ExecuteStreamFrame) -> Result<()> {
        let canonical = frame
            .canonical_ndjson_bytes()
            .map_err(anyhow::Error::msg)
            .context("canonicalize runtime replay frame")?;
        let mut state = self
            .stream
            .lock()
            .map_err(|_| anyhow::anyhow!("runtime replay stream lock poisoned"))?;
        if state.producer_closed {
            anyhow::bail!("runtime replay producer emitted a post-close frame");
        }
        let candidate_sequence = frame_sequence(frame);
        let expected_sequence = state
            .frames
            .last()
            .map(frame_sequence)
            .and_then(|sequence| sequence.checked_add(1))
            .ok_or_else(|| anyhow::anyhow!("runtime replay frame sequence exhausted"))?;
        if candidate_sequence != expected_sequence {
            anyhow::bail!("runtime replay producer emitted a gap or reorder");
        }
        if frame_stream_id(frame) != state.stream_id {
            anyhow::bail!("runtime replay producer changed stream identity");
        }
        let retained_bytes = state
            .retained_bytes
            .checked_add(canonical.len())
            .ok_or_else(|| anyhow::anyhow!("runtime replay retained byte count overflow"))?;
        if state.frames.len() >= MAX_REPLAY_FRAMES_PER_STREAM
            || retained_bytes > MAX_REPLAY_BYTES_PER_STREAM
        {
            state.producer_closed = true;
            state.replay_unavailable = true;
            state.subscribers.clear();
            anyhow::bail!("runtime replay stream capacity is exhausted");
        }
        state.retained_bytes = retained_bytes;
        state.frames.push(frame.clone());
        state
            .subscribers
            .retain(|subscriber| subscriber.send(frame.clone()).is_ok());
        state.producer_closed = matches!(
            frame,
            ExecuteStreamFrame::Exit { .. } | ExecuteStreamFrame::Error { .. }
        );
        if state.producer_closed {
            state.subscribers.clear();
        }
        Ok(())
    }
}

pub(crate) fn publish_replayable_frame(
    publisher: Option<&RuntimeReplayPublisher>,
    tx: &UnboundedSender<ExecuteStreamFrame>,
    frame: ExecuteStreamFrame,
) -> Result<()> {
    let replay_result = publisher.map_or(Ok(()), |publisher| publisher.publish(&frame));
    let _ = tx.send(frame);
    replay_result
}

fn validate_acceptance_record_id(value: &str) -> Result<()> {
    if value.trim() != value || !value.starts_with("wwa_") || value.len() == 4 {
        anyhow::bail!("runtime replay acceptance_record_id is invalid");
    }
    Ok(())
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

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::sync::mpsc;
    use transport_api_types::{
        ExecuteStreamFrame, ExecuteStreamReplayRequestV1, RuntimeEventIdentityV1,
        RuntimeFrameIdentityV1, RuntimeTerminalIdentityV1,
        RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
    };

    fn frame_identity(sequence: u64) -> RuntimeFrameIdentityV1 {
        RuntimeFrameIdentityV1 {
            schema_version: RUNTIME_FRAME_IDENTITY_SCHEMA_VERSION_V1,
            stream_id: "rts_replay_fixture".to_string(),
            frame_sequence: sequence,
        }
    }

    fn start() -> ExecuteStreamFrame {
        ExecuteStreamFrame::Start {
            frame_identity: frame_identity(1),
            span_id: "spn_replay_fixture".to_string(),
        }
    }

    fn exit() -> ExecuteStreamFrame {
        let event_identity = RuntimeEventIdentityV1 {
            event_id: "evt_replay_terminal".to_string(),
            event_sequence: 1,
        };
        ExecuteStreamFrame::Exit {
            frame_identity: frame_identity(3),
            terminal_identity: RuntimeTerminalIdentityV1::from(&event_identity),
            event_identity,
            exit: 0,
            span_id: "spn_replay_fixture".to_string(),
            scopes_used: Vec::new(),
            fs_diff: None,
            process_telemetry: Default::default(),
        }
    }

    fn request(after_frame_sequence: u64) -> ExecuteStreamReplayRequestV1 {
        ExecuteStreamReplayRequestV1 {
            schema_version: 1,
            acceptance_record_id: "wwa_replay_fixture".to_string(),
            stream_id: "rts_replay_fixture".to_string(),
            after_frame_sequence,
        }
    }

    #[test]
    fn replay_subscriber_receives_exact_backlog_and_continued_live_terminal() {
        let registry = RuntimeReplayRegistry::default();
        let publisher = registry
            .begin("wwa_replay_fixture", &start())
            .expect("begin replay stream");
        let stdout = ExecuteStreamFrame::Stdout {
            frame_identity: frame_identity(2),
            chunk_b64: "b25jZQ==".to_string(),
        };
        publisher.publish(&stdout).expect("publish stdout");
        let mut replay = registry
            .subscribe(&request(1))
            .expect("subscribe from Start");
        assert_eq!(
            replay
                .try_recv()
                .expect("replayed stdout")
                .canonical_ndjson_bytes()
                .expect("canonical replayed stdout"),
            stdout
                .canonical_ndjson_bytes()
                .expect("canonical original stdout")
        );
        publisher.publish(&exit()).expect("publish terminal");
        assert!(matches!(
            replay.try_recv(),
            Ok(ExecuteStreamFrame::Exit { .. })
        ));
        assert!(matches!(
            replay.try_recv(),
            Err(tokio::sync::mpsc::error::TryRecvError::Disconnected)
        ));
    }

    #[test]
    fn replay_rejects_absent_mismatched_ahead_gap_reorder_and_post_close_state() {
        let registry = RuntimeReplayRegistry::default();
        assert!(registry.subscribe(&request(0)).is_err());
        let publisher = registry
            .begin("wwa_replay_fixture", &start())
            .expect("begin replay stream");
        let mut mismatch = request(0);
        mismatch.stream_id = "rts_other".to_string();
        assert!(registry.subscribe(&mismatch).is_err());
        assert!(registry.subscribe(&request(2)).is_err());
        let gap = ExecuteStreamFrame::Stdout {
            frame_identity: frame_identity(3),
            chunk_b64: "Z2Fw".to_string(),
        };
        assert!(publisher.publish(&gap).is_err());
        let stdout = ExecuteStreamFrame::Stdout {
            frame_identity: frame_identity(2),
            chunk_b64: "b25jZQ==".to_string(),
        };
        publisher.publish(&stdout).expect("publish stdout");
        assert!(publisher.publish(&stdout).is_err());
        publisher.publish(&exit()).expect("publish terminal");
        assert!(publisher.publish(&stdout).is_err());
        assert!(registry.begin("wwa_replay_fixture", &start()).is_err());
    }

    #[test]
    fn replay_capacity_failure_never_suppresses_live_delivery() {
        let registry = RuntimeReplayRegistry::default();
        let publisher = registry
            .begin("wwa_replay_fixture", &start())
            .expect("begin bounded replay stream");
        publisher
            .stream
            .lock()
            .expect("lock bounded replay stream")
            .retained_bytes = MAX_REPLAY_BYTES_PER_STREAM;
        let stdout = ExecuteStreamFrame::Stdout {
            frame_identity: frame_identity(2),
            chunk_b64: "b25jZQ==".to_string(),
        };
        let (tx, mut rx) = mpsc::unbounded_channel();
        assert!(publish_replayable_frame(Some(&publisher), &tx, stdout.clone()).is_err());
        assert_eq!(
            rx.try_recv()
                .expect("live frame remains deliverable")
                .canonical_ndjson_bytes()
                .expect("canonical live frame"),
            stdout
                .canonical_ndjson_bytes()
                .expect("canonical expected live frame")
        );
        assert!(registry.subscribe(&request(0)).is_err());
    }

    #[test]
    fn replay_subscriptions_are_bounded_even_while_exact_stream_is_live() {
        const EXPECTED_SUBSCRIPTION_LIMIT: usize = 8;

        let registry = RuntimeReplayRegistry::default();
        let _publisher = registry
            .begin("wwa_replay_fixture", &start())
            .expect("begin bounded subscription stream");
        let subscriptions = (0..EXPECTED_SUBSCRIPTION_LIMIT)
            .map(|_| registry.subscribe(&request(0)).expect("bounded subscriber"))
            .collect::<Vec<_>>();
        assert!(registry.subscribe(&request(0)).is_err());
        drop(subscriptions);
        assert!(registry.subscribe(&request(0)).is_ok());
    }
}
