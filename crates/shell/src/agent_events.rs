use std::sync::{Mutex, OnceLock};

use substrate_common::agent_events::AgentEvent;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};

/// Global sender storage so any component can publish agent events.
static AGENT_EVENT_SENDER: OnceLock<Mutex<Option<UnboundedSender<AgentEvent>>>> = OnceLock::new();

/// Initialise the global event channel and return the receiver for consumers.
pub(crate) fn init_event_channel() -> UnboundedReceiver<AgentEvent> {
    let (tx, rx) = mpsc::unbounded_channel::<AgentEvent>();
    let cell = AGENT_EVENT_SENDER.get_or_init(|| Mutex::new(None));

    if let Ok(mut guard) = cell.lock() {
        *guard = Some(tx);
    }

    rx
}

/// Publish an event to the shared channel. Returns `true` if the send succeeded.
pub(crate) fn publish_agent_event(event: AgentEvent) -> bool {
    let sender = AGENT_EVENT_SENDER
        .get()
        .and_then(|lock| lock.lock().ok().and_then(|guard| guard.as_ref().cloned()));

    if let Some(sender) = sender {
        sender.send(event).is_ok()
    } else {
        false
    }
}

/// Obtain a clone of the current sender if one has been initialised.
pub(crate) fn agent_event_sender() -> Option<UnboundedSender<AgentEvent>> {
    AGENT_EVENT_SENDER
        .get()
        .and_then(|lock| lock.lock().ok().and_then(|guard| guard.as_ref().cloned()))
}

/// Clear the stored sender, allowing the channel to be dropped cleanly.
pub(crate) fn clear_agent_event_sender() {
    if let Some(lock) = AGENT_EVENT_SENDER.get() {
        if let Ok(mut guard) = lock.lock() {
            guard.take();
        }
    }
}
