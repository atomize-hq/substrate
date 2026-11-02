use std::time::Duration;

#[derive(Debug, Clone)]
pub struct ClaudeCommandRequest {
    pub(crate) path: Vec<String>,
    pub(crate) args: Vec<String>,
    pub(crate) stdin: Option<Vec<u8>>,
    pub(crate) timeout: Option<Duration>,
}

impl ClaudeCommandRequest {
    pub fn root() -> Self {
        Self {
            path: Vec::new(),
            args: Vec::new(),
            stdin: None,
            timeout: None,
        }
    }

    pub fn new(path: impl IntoIterator<Item = impl Into<String>>) -> Self {
        Self {
            path: path.into_iter().map(Into::into).collect(),
            args: Vec::new(),
            stdin: None,
            timeout: None,
        }
    }

    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    pub fn args(mut self, args: impl IntoIterator<Item = impl Into<String>>) -> Self {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }

    pub fn stdin_bytes(mut self, bytes: Vec<u8>) -> Self {
        self.stdin = Some(bytes);
        self
    }

    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn argv(&self) -> Vec<String> {
        let mut out = Vec::new();
        out.extend(self.path.iter().cloned());
        out.extend(self.args.iter().cloned());
        out
    }
}
