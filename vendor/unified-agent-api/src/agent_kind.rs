use crate::AgentWrapperError;

pub(crate) fn validate_agent_kind(value: &str) -> Result<(), AgentWrapperError> {
    if value.is_empty() {
        return Err(AgentWrapperError::InvalidAgentKind {
            message: "agent kind must not be empty".to_string(),
        });
    }
    let mut chars = value.chars();
    let first = chars.next().unwrap_or_default();
    if !first.is_ascii_lowercase() {
        return Err(AgentWrapperError::InvalidAgentKind {
            message: "agent kind must start with a lowercase ASCII letter".to_string(),
        });
    }
    for ch in chars {
        if !(ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_') {
            return Err(AgentWrapperError::InvalidAgentKind {
                message: "agent kind must match ^[a-z][a-z0-9_]*$".to_string(),
            });
        }
    }
    Ok(())
}
