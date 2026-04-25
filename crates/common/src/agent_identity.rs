pub fn derive_agent_backend_id(kind: &str, agent_id: &str) -> String {
    format!("{}:{}", kind.trim(), agent_id.trim())
}
