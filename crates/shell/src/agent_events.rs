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

pub(crate) fn schedule_demo_burst(agent_count: usize, events_per_agent: usize, delay: Duration) {
    if agent_event_sender().is_none() {
        return;
    }

    let agent_count = agent_count.max(1).min(16);
    let events_per_agent = events_per_agent.max(1).min(10_000);

    for agent_idx in 0..agent_count {
        let agent = format!("burst-{agent_idx:02}");
        thread::spawn({
            let agent_name = agent.clone();
            move || {
                for event_idx in 0..events_per_agent {
                    let message = format!("chunk #{event_idx:05}");
                    let is_stderr = event_idx % 20 == 0;
                    let _ = publish_agent_event(AgentEvent::stream_chunk(
                        agent_name.as_str(),
                        is_stderr,
                        message,
                    ));
                    if !delay.is_zero() {
                        thread::sleep(delay);
                    }
                }
                let _ = publish_agent_event(AgentEvent::message(
                    agent_name.clone(),
                    AgentEventKind::TaskEnd,
                    "burst complete",
                ));
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use tokio::runtime::Runtime;

    #[test]
    fn schedule_demo_burst_emits_expected_events() {
        let rt = Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();
            schedule_demo_burst(2, 5, Duration::from_millis(0));
            let mut counts: HashMap<String, usize> = HashMap::new();
            let mut completions = 0;
            let expected_stream = 2 * 5;
            while completions < 2 || counts.values().sum::<usize>() < expected_stream {
                let ev = rx.recv().await.expect("event");
                match ev.kind {
                    AgentEventKind::PtyData => {
                        *counts.entry(ev.agent_id).or_default() += 1;
                    }
                    AgentEventKind::TaskEnd => {
                        completions += 1;
                    }
                    _ => {}
                }
            }
            assert!(
                counts.values().all(|&n| n == 5),
                "each agent should emit all chunks: {:?}",
                counts
            );
            assert_eq!(completions, 2);
        });
        clear_agent_event_sender();
    }
}
