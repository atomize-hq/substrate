pub(super) enum SetupTokenProcess {
    Pipes {
        child: tokio::process::Child,
        stdin: Option<tokio::process::ChildStdin>,
    },
    #[cfg(unix)]
    Pty {
        child: Box<dyn portable_pty::Child + Send + Sync>,
        writer: Option<Box<dyn std::io::Write + Send>>,
    },
}
