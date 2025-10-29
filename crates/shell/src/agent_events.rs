use std::process::ExitStatus;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use substrate_common::agent_events::{AgentEvent, AgentEventKind};
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

pub(crate) fn publish_command_completion(command: &str, status: &ExitStatus) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if status.signal().is_some() {
            return;
        }
    }

    let event = if status.success() {
        AgentEvent::message(
            "shell",
            AgentEventKind::TaskEnd,
            format!("Command `{command}` completed successfully"),
        )
    } else {
        let code = status.code().unwrap_or(-1);
        AgentEvent::message(
            "shell",
            AgentEventKind::Alert,
            format!("Command `{command}` exited with status {code}"),
        )
    };

    let _ = publish_agent_event(event);
}

pub(crate) fn format_event_line(event: &AgentEvent) -> String {
    let agent = if event.agent_id.is_empty() {
        "agent"
    } else {
        event.agent_id.as_str()
    };

    let message = extract_event_message(&event.kind, &event.data);
    format!("[{agent}] {message}")
}

fn extract_event_message(kind: &AgentEventKind, data: &serde_json::Value) -> String {
    if let Some(msg) = data.get("message").and_then(serde_json::Value::as_str) {
        return msg.to_string();
    }

    if let Some(chunk) = data.get("chunk").and_then(serde_json::Value::as_str) {
        let stream = data
            .get("stream")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("stdout");
        return format!("{}: {}", stream, chunk);
    }

    if data.is_null() {
        kind.to_string()
    } else {
        data.to_string()
    }
}

pub(crate) fn schedule_demo_events() {
    if agent_event_sender().is_none() {
        return;
    }

    let events = vec![
        (
            Duration::from_millis(300),
            "Demo agent event #1".to_string(),
        ),
        (
            Duration::from_millis(820),
            "Demo agent event #2".to_string(),
        ),
        (
            Duration::from_millis(1350),
            "Demo agent event #3".to_string(),
        ),
    ];

    thread::spawn(move || {
        for (delay, message) in events {
            thread::sleep(delay);
            let _ = publish_agent_event(AgentEvent::message(
                "demo",
                AgentEventKind::TaskProgress,
                message,
            ));
        }
    });
}
