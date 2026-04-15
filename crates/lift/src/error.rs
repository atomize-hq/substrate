#[derive(Debug)]
pub enum LiftError {
    NotImplemented(&'static str),
    Cli(String),
}

impl std::fmt::Display for LiftError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NotImplemented(message) => write!(f, "not implemented: {message}"),
            Self::Cli(message) => write!(f, "{message}"),
        }
    }
}

impl std::error::Error for LiftError {}
