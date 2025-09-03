use std::env;
use std::sync::Mutex;
use lazy_static::lazy_static;
use uuid::Uuid;

lazy_static! {
    static ref SESSION_INFO: Mutex<SessionInfo> = Mutex::new(SessionInfo::from_env());
}

#[derive(Debug, Clone)]
pub struct SessionInfo {
    pub session_id: String,
    pub parent_span_id: Option<String>,
    pub world_id: Option<String>,
    pub agent_id: String,
    pub policy_id: String,
    pub trace_log: String,
}

impl SessionInfo {
    fn from_env() -> Self {
        Self {
            session_id: env::var("SUBSTRATE_SESSION_ID")
                .unwrap_or_else(|_| format!("{}", Uuid::now_v7())),
            parent_span_id: env::var("SUBSTRATE_PARENT_SPAN").ok(),
            world_id: env::var("SUBSTRATE_WORLD_ID").ok(),
            agent_id: env::var("SUBSTRATE_AGENT_ID")
                .unwrap_or_else(|_| "human".to_string()),
            policy_id: env::var("SUBSTRATE_POLICY_ID")
                .unwrap_or_else(|_| "default".to_string()),
            trace_log: env::var("SUBSTRATE_TRACE_LOG")
                .unwrap_or_else(|_| {
                    let home = env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
                    format!("{}/.substrate/trace.jsonl", home)
                }),
        }
    }
}

pub fn get_session_info() -> SessionInfo {
    SESSION_INFO.lock().unwrap().clone()
}

pub fn generate_span_id() -> String {
    format!("spn_{}", Uuid::now_v7())
}

pub fn inherit_correlation_env() -> Vec<(String, String)> {
    let info = get_session_info();
    let mut env_vars = vec![
        ("SUBSTRATE_SESSION_ID".to_string(), info.session_id),
        ("SUBSTRATE_AGENT_ID".to_string(), info.agent_id),
        ("SUBSTRATE_POLICY_ID".to_string(), info.policy_id),
        ("SUBSTRATE_TRACE_LOG".to_string(), info.trace_log),
    ];
    
    if let Some(world_id) = info.world_id {
        env_vars.push(("SUBSTRATE_WORLD_ID".to_string(), world_id));
    }
    
    env_vars
}