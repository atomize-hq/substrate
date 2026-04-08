use std::process::ExitStatus;
use std::sync::{Mutex, OnceLock};
use std::thread;
use std::time::Duration;

use substrate_common::agent_events::{AgentEvent, AgentEventKind, MessageEventKind};
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use uuid::Uuid;

/// Global sender storage so any component can publish agent events.
static AGENT_EVENT_SENDER: OnceLock<Mutex<Option<UnboundedSender<AgentEvent>>>> = OnceLock::new();
static ORCHESTRATION_SESSION_ID: OnceLock<String> = OnceLock::new();

#[cfg(test)]
static EVENT_TEST_GUARD: OnceLock<Mutex<()>> = OnceLock::new();

#[cfg(test)]
pub(crate) fn acquire_event_test_guard() -> std::sync::MutexGuard<'static, ()> {
    EVENT_TEST_GUARD
        .get_or_init(|| Mutex::new(()))
        .lock()
        .expect("event channel test guard poisoned")
}

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

pub(crate) fn orchestration_session_id() -> String {
    ORCHESTRATION_SESSION_ID
        .get_or_init(|| Uuid::now_v7().to_string())
        .clone()
}

pub(crate) fn publish_command_completion(command: &str, cmd_id: &str, status: &ExitStatus) {
    #[cfg(unix)]
    {
        use std::os::unix::process::ExitStatusExt;
        if status.signal().is_some() {
            return;
        }
    }

    if status.success() {
        let enabled = std::env::var("SUBSTRATE_COMMAND_SUCCESS_EVENTS")
            .map(|v| matches!(v.to_ascii_lowercase().as_str(), "1" | "true" | "yes"))
            .unwrap_or(false);
        if !enabled {
            return;
        }

        let mut event = AgentEvent::message(
            "shell",
            orchestration_session_id(),
            cmd_id.to_string(),
            MessageEventKind::TaskEnd,
            format!("Command `{command}` completed successfully"),
        );
        event.cmd_id = Some(cmd_id.to_string());
        let _ = publish_agent_event(event);
        return;
    }

    let code = status.code().unwrap_or(-1);
    let mut event = AgentEvent::message(
        "shell",
        orchestration_session_id(),
        cmd_id.to_string(),
        MessageEventKind::TaskEnd,
        format!("Command `{command}` exited with status {code}"),
    );
    event.cmd_id = Some(cmd_id.to_string());

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
    fn escape_line_breaks(raw: &str) -> String {
        raw.replace('\n', "\\n").replace('\r', "\\r")
    }

    if let Some(msg) = data.get("message").and_then(serde_json::Value::as_str) {
        return escape_line_breaks(msg);
    }

    if let Some(chunk) = data.get("chunk").and_then(serde_json::Value::as_str) {
        let stream = data
            .get("stream")
            .and_then(serde_json::Value::as_str)
            .unwrap_or("stdout");
        return escape_line_breaks(&format!("{}: {}", stream, chunk));
    }

    if data.is_null() {
        kind.to_string()
    } else {
        escape_line_breaks(&data.to_string())
    }
}

pub(crate) fn schedule_demo_events() {
    if agent_event_sender().is_none() {
        return;
    }

    let orchestration_session_id = orchestration_session_id();
    let run_id = Uuid::now_v7().to_string();

    let mut first = AgentEvent::message(
        "demo-agent",
        orchestration_session_id.clone(),
        run_id.clone(),
        MessageEventKind::TaskProgress,
        "Demo agent event #1".to_string(),
    );
    first.role = Some("member".to_string());
    let _ = publish_agent_event(first);

    let events = vec![
        (
            Duration::from_millis(400),
            "Demo agent event #2".to_string(),
        ),
        (
            Duration::from_millis(400),
            "Demo agent event #3".to_string(),
        ),
    ];

    thread::spawn(move || {
        for (delay, message) in events {
            thread::sleep(delay);
            let mut event = AgentEvent::message(
                "demo-agent",
                orchestration_session_id.clone(),
                run_id.clone(),
                MessageEventKind::TaskProgress,
                message,
            );
            event.role = Some("member".to_string());
            let _ = publish_agent_event(event);
        }
    });
}

pub(crate) fn schedule_demo_burst(agent_count: usize, events_per_agent: usize, delay: Duration) {
    if agent_event_sender().is_none() {
        return;
    }

    let agent_count = agent_count.clamp(1, 16);
    let events_per_agent = events_per_agent.clamp(1, 10_000);
    let orchestration_session_id = orchestration_session_id();
    let run_id = Uuid::now_v7().to_string();

    for agent_idx in 0..agent_count {
        let agent = format!("burst-{agent_idx:02}");
        thread::spawn({
            let agent_name = agent.clone();
            let orchestration_session_id = orchestration_session_id.clone();
            let run_id = run_id.clone();
            move || {
                for event_idx in 0..events_per_agent {
                    let message = format!("chunk #{event_idx:05}");
                    let is_stderr = event_idx % 20 == 0;
                    let mut event = AgentEvent::stream_chunk(
                        agent_name.as_str(),
                        orchestration_session_id.clone(),
                        run_id.clone(),
                        is_stderr,
                        message,
                    );
                    event.role = Some("member".to_string());
                    let _ = publish_agent_event(event);
                    if !delay.is_zero() {
                        thread::sleep(delay);
                    }
                }
                let mut event = AgentEvent::message(
                    agent_name.clone(),
                    orchestration_session_id,
                    run_id,
                    MessageEventKind::TaskEnd,
                    "burst complete",
                );
                event.role = Some("member".to_string());
                let _ = publish_agent_event(event);
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;
    use tokio::runtime::Runtime;

    fn exit_status_from_code(code: i32) -> ExitStatus {
        #[cfg(unix)]
        {
            ExitStatus::from_raw(code << 8)
        }

        #[cfg(windows)]
        {
            ExitStatus::from_raw(code as u32)
        }
    }

    #[test]
    fn schedule_demo_burst_emits_expected_events() {
        let _guard = super::acquire_event_test_guard();
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

    #[test]
    fn publish_command_completion_failure_emits_task_end_with_cmd_id() {
        let _guard = super::acquire_event_test_guard();
        let rt = Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();
            let cmd_id = "cmd-failure";
            let status = exit_status_from_code(7);

            publish_command_completion("false", cmd_id, &status);

            let event = rx.recv().await.expect("event");
            assert_eq!(event.kind, AgentEventKind::TaskEnd);
            assert_eq!(event.cmd_id.as_deref(), Some(cmd_id));
            assert_eq!(
                event
                    .data
                    .get("message")
                    .and_then(serde_json::Value::as_str),
                Some("Command `false` exited with status 7")
            );
            assert!(
                event.data.get("code").is_none(),
                "failed command completion must not be emitted as alert payload: {:?}",
                event.data
            );
        });
        clear_agent_event_sender();
    }

    #[test]
    fn publish_command_completion_success_emits_task_end_when_enabled() {
        let _guard = super::acquire_event_test_guard();
        let rt = Runtime::new().expect("runtime");
        rt.block_on(async {
            let mut rx = init_event_channel();
            let cmd_id = "cmd-success";
            let status = exit_status_from_code(0);
            std::env::set_var("SUBSTRATE_COMMAND_SUCCESS_EVENTS", "1");

            publish_command_completion("true", cmd_id, &status);

            let event = rx.recv().await.expect("event");
            assert_eq!(event.kind, AgentEventKind::TaskEnd);
            assert_eq!(event.cmd_id.as_deref(), Some(cmd_id));
            assert_eq!(
                event
                    .data
                    .get("message")
                    .and_then(serde_json::Value::as_str),
                Some("Command `true` completed successfully")
            );

            std::env::remove_var("SUBSTRATE_COMMAND_SUCCESS_EVENTS");
        });
        clear_agent_event_sender();
    }
}
