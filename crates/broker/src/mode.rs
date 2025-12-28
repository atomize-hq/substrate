use std::env;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PolicyMode {
    Disabled,
    Observe,
    Enforce,
}

impl PolicyMode {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Observe => "observe",
            Self::Enforce => "enforce",
        }
    }

    pub fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "disabled" => Some(Self::Disabled),
            "observe" => Some(Self::Observe),
            "enforce" => Some(Self::Enforce),
            _ => None,
        }
    }

    pub fn from_env() -> Self {
        env::var("SUBSTRATE_POLICY_MODE")
            .ok()
            .and_then(|raw| Self::parse_insensitive(&raw))
            .unwrap_or(Self::Observe)
    }
}
