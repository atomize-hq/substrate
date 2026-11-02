use crate::{AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperKind};

fn status_event(message: Option<String>) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(super::AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Status,
        channel: Some("status".to_string()),
        text: None,
        message,
        data: None,
    }
}

fn text_event(text: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(super::AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::TextOutput,
        channel: Some("assistant".to_string()),
        text: Some(text),
        message: None,
        data: None,
    }
}

fn unknown_event() -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(super::AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Unknown,
        channel: None,
        text: None,
        message: None,
        data: None,
    }
}

fn error_event(message: String) -> AgentWrapperEvent {
    AgentWrapperEvent {
        agent_kind: AgentWrapperKind(super::AGENT_KIND.to_string()),
        kind: AgentWrapperEventKind::Error,
        channel: Some("error".to_string()),
        text: None,
        message: Some(message),
        data: None,
    }
}

pub(super) fn map_run_json_event(event: opencode::OpencodeRunJsonEvent) -> Vec<AgentWrapperEvent> {
    match event {
        opencode::OpencodeRunJsonEvent::Text { text, .. } => vec![text_event(text)],
        opencode::OpencodeRunJsonEvent::StepStart { .. } => {
            vec![status_event(Some("step_start".to_string()))]
        }
        opencode::OpencodeRunJsonEvent::StepFinish { .. } => {
            vec![status_event(Some("step_finish".to_string()))]
        }
        opencode::OpencodeRunJsonEvent::Unknown { .. } => vec![unknown_event()],
        opencode::OpencodeRunJsonEvent::TerminalError { message, .. } => vec![error_event(message)],
    }
}
