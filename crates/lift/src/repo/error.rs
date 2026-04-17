use crate::kernel::RepoPath;

pub(crate) type RepoResult<T> = Result<T, RepoError>;

#[derive(Debug, thiserror::Error, Clone, Eq, PartialEq)]
pub(crate) enum RepoError {
    #[error("repo start path does not exist")]
    StartPathNotFound { path: String },

    #[error("repo root could not be detected")]
    RootNotFound {
        start_path: String,
        markers: Vec<String>,
    },

    #[error("repo root is not a directory")]
    RootNotDirectory { path: String },

    #[error("repo I/O failure")]
    Io {
        op: &'static str,
        path: String,
        reason: String,
    },

    #[error("invalid root marker")]
    InvalidRootMarker { input: String },

    #[error("invalid repo-relative path derived from filesystem entry")]
    InvalidRepoPath {
        display_path: String,
        reason: String,
    },

    #[error("encountered non-utf8 filesystem path")]
    NonUtf8Path { display_path: String },

    #[error("ignore glob compile failure")]
    IgnoreGlobCompile { pattern: String, reason: String },

    #[error("snapshot source is not implemented")]
    UnsupportedSnapshotSource { source_name: &'static str },

    #[error("snapshot option is not implemented")]
    UnsupportedSnapshotOption { option_name: &'static str },

    #[error("file exceeds configured max_file_bytes")]
    FileTooLarge {
        display_path: String,
        size_bytes: u64,
        max_file_bytes: u64,
    },

    #[error("blob not present in snapshot")]
    MissingBlob { path: RepoPath },
}
