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

    #[error("git repository open failure")]
    GitOpen { path: String, reason: String },

    #[error("git revision could not be resolved")]
    GitRevisionResolve {
        rev: String,
        repo_root: String,
        reason: String,
    },

    #[error("git revision did not resolve to a tree-compatible object")]
    GitRevisionObjectKind {
        rev: String,
        actual_kind: String,
        expected_kind: &'static str,
    },

    #[error("git object lookup failure")]
    GitObjectLookup { object_id: String, reason: String },

    #[error("git blob did not decode to utf-8 symlink target")]
    GitSymlinkTargetInvalidUtf8 { rev: String, path: RepoPath },

    #[error("symlink follow escaped the repository root")]
    SymlinkTargetEscape { path: RepoPath, target: String },

    #[error("symlink follow target does not exist")]
    SymlinkTargetDangling { path: RepoPath, target: String },

    #[error("symlink follow detected a loop")]
    SymlinkTargetLoop { path: RepoPath, target: String },

    #[error("symlink follow target is a directory")]
    SymlinkTargetDirectory { path: RepoPath, target: String },

    #[error("file exceeds configured max_file_bytes")]
    FileTooLarge {
        display_path: String,
        size_bytes: u64,
        max_file_bytes: u64,
    },

    #[error("blob not present in snapshot")]
    MissingBlob { path: RepoPath },
}
