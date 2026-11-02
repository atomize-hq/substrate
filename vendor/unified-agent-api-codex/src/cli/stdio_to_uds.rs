use std::path::PathBuf;

/// Request for `codex stdio-to-uds <SOCKET_PATH>`.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StdioToUdsRequest {
    /// Path to the Unix domain socket to connect to.
    pub socket_path: PathBuf,
    /// Optional working directory override for the spawned process.
    pub working_dir: Option<PathBuf>,
}

impl StdioToUdsRequest {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            working_dir: None,
        }
    }

    /// Sets the working directory used to resolve the socket path.
    pub fn working_dir(mut self, dir: impl Into<PathBuf>) -> Self {
        self.working_dir = Some(dir.into());
        self
    }
}
