use crate::execution::value_parse::parse_bool_flag;
use crate::execution::workspace;
use anyhow::{anyhow, Context, Result};
use serde::ser::SerializeMap;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::env;
use std::error::Error as StdError;
use std::fmt;
use std::fs;
#[cfg(target_os = "linux")]
use std::fs::{File, OpenOptions};
use std::io;
#[cfg(target_os = "linux")]
use std::io::Read as _;
#[cfg(target_os = "linux")]
use std::os::fd::{AsRawFd, FromRawFd};
#[cfg(target_os = "linux")]
use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};
use std::time::SystemTime;
use substrate_broker::validate_backend_id;
use substrate_common::paths as substrate_paths;
use substrate_common::WorldRootMode;
use transport_api_types::{
    AgentConnectionAuthV1, AgentConnectionBillingV1, AgentConnectionProfileV1,
    AgentHarnessFamilyV1, AgentSelectionSourceScopeV1, AgentSelectionSourceV1,
    AgentSelectionTupleV1,
};

pub(crate) const PROTECTED_EXCLUDES: [&str; 2] = [".git/**", ".substrate/**"];

#[derive(Debug, Clone, PartialEq, Eq)]
struct FileStatKey {
    exists: bool,
    mtime: Option<SystemTime>,
    size: Option<u64>,
}

impl FileStatKey {
    fn for_path(path: &Path) -> Result<Self> {
        match fs::metadata(path) {
            Ok(meta) => Ok(Self {
                exists: true,
                mtime: meta.modified().ok(),
                size: Some(meta.len()),
            }),
            Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(Self {
                exists: false,
                mtime: None,
                size: None,
            }),
            Err(err) => Err(err).with_context(|| format!("failed to stat {}", path.display())),
        }
    }
}

#[derive(Debug, Clone)]
struct ConfigPatchCacheEntry {
    workspace_root: Option<PathBuf>,
    global_path: PathBuf,
    workspace_path: Option<PathBuf>,
    global_stat: FileStatKey,
    workspace_stat: Option<FileStatKey>,
    global_patch: SubstrateConfigPatch,
    workspace_patch: Option<SubstrateConfigPatch>,
}

static CONFIG_PATCH_CACHE: OnceLock<Mutex<Option<ConfigPatchCacheEntry>>> = OnceLock::new();

pub(crate) fn invalidate_config_cache() {
    let cache = CONFIG_PATCH_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(mut guard) = cache.lock() {
        *guard = None;
    }
}

#[derive(Debug)]
pub(crate) struct UserError {
    message: String,
}

impl UserError {
    fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for UserError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl StdError for UserError {}

pub(crate) fn user_error(message: impl Into<String>) -> anyhow::Error {
    anyhow::Error::new(UserError::new(message))
}

pub(crate) fn is_user_error(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| cause.is::<UserError>())
}

/// A present configuration cannot safely enter installer default recovery.
#[derive(Debug)]
pub(crate) struct ProtectedSelectionConfigErrorV1;

impl fmt::Display for ProtectedSelectionConfigErrorV1 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(
            "refusing replacement of invalid or unverified agent selection/profile settings",
        )
    }
}

impl StdError for ProtectedSelectionConfigErrorV1 {}

pub(crate) fn is_protected_selection_config_error_v1(err: &anyhow::Error) -> bool {
    err.is::<ProtectedSelectionConfigErrorV1>()
}

pub(crate) const REPL_MAX_PTY_BUFFERED_LINES_MIN: i64 = 0;
pub(crate) const REPL_MAX_PTY_BUFFERED_LINES_MAX: i64 = 16_384;
pub(crate) const REPL_MAX_PTY_BUFFERED_LINES_DEFAULT: i64 = 2_048;

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SubstrateConfig {
    pub world: WorldConfig,
    pub policy: PolicyConfig,
    pub sync: SyncConfig,
    pub repl: ReplConfig,
    pub llm: LlmConfig,
    pub agents: AgentsConfig,
    #[serde(skip)]
    pub selection_sources: Vec<AgentSelectionSourceV1>,
    #[serde(skip)]
    pub selection_preference_source: Option<AgentSelectionSourceV1>,
    #[serde(skip)]
    pub selection_profiles: Vec<AgentConnectionProfileV1>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ReplExitCwdMode {
    #[default]
    Entered,
    LastWorld,
}

impl ReplExitCwdMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "entered" => Some(Self::Entered),
            "last_world" => Some(Self::LastWorld),
            _ => None,
        }
    }
}

impl<'de> Deserialize<'de> for ReplExitCwdMode {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let raw = String::deserialize(deserializer)?;
        Self::parse_insensitive(&raw).ok_or_else(|| {
            serde::de::Error::custom(format!(
                "invalid repl.exit_cwd '{}'; expected entered or last_world",
                raw
            ))
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct I64ClampInfo {
    pub provided: i64,
    pub effective: i64,
    pub min: i64,
    pub max: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ReplConfig {
    pub exit_cwd: ReplExitCwdMode,
    pub max_pty_buffered_lines: i64,
    #[serde(skip)]
    pub max_pty_buffered_lines_clamp: Option<I64ClampInfo>,
}

impl Default for ReplConfig {
    fn default() -> Self {
        Self {
            exit_cwd: ReplExitCwdMode::default(),
            max_pty_buffered_lines: REPL_MAX_PTY_BUFFERED_LINES_DEFAULT,
            max_pty_buffered_lines_clamp: None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmConfig {
    pub enabled: bool,
    pub gateway: LlmGatewayConfig,
    pub routing: LlmRoutingConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmGatewayConfig {
    pub enabled: bool,
    pub mode: LlmGatewayMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LlmGatewayMode {
    #[default]
    InWorld,
    HostOnly,
}

impl LlmGatewayMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "in_world" => Some(Self::InWorld),
            "host_only" => Some(Self::HostOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmRoutingConfig {
    pub default_backend: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldConfig {
    pub enabled: bool,
    pub anchor_mode: WorldRootMode,
    pub anchor_path: String,
    pub caged: bool,
    pub net: WorldNetConfig,
    pub env: WorldEnvConfig,
    pub deps: WorldDepsConfig,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            anchor_mode: WorldRootMode::Project,
            anchor_path: String::new(),
            caged: true,
            net: WorldNetConfig::default(),
            env: WorldEnvConfig::default(),
            deps: WorldDepsConfig::default(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldNetConfig {
    pub filter: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldEnvConfig {
    pub inherit_from_host: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldDepsConfig {
    pub enabled: Vec<String>,
    pub inventory_mode: WorldDepsInventoryMode,
    pub builtins: WorldDepsBuiltinsMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepsInventoryMode {
    #[default]
    Merged,
    WorkspaceOnly,
}

impl WorldDepsInventoryMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "merged" => Some(Self::Merged),
            "workspace_only" => Some(Self::WorkspaceOnly),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldDepsBuiltinsMode {
    #[default]
    Enabled,
    Disabled,
}

impl WorldDepsBuiltinsMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "enabled" => Some(Self::Enabled),
            "disabled" => Some(Self::Disabled),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub(crate) enum PolicyMode {
    Disabled,
    #[default]
    Observe,
    Enforce,
}

impl PolicyMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "disabled" => Some(Self::Disabled),
            "observe" => Some(Self::Observe),
            "enforce" => Some(Self::Enforce),
            _ => None,
        }
    }

    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            Self::Disabled => "disabled",
            Self::Observe => "observe",
            Self::Enforce => "enforce",
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyConfig {
    pub mode: PolicyMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncDirection {
    #[default]
    FromWorld,
    FromHost,
    Both,
}

impl SyncDirection {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "from_world" => Some(Self::FromWorld),
            "from_host" => Some(Self::FromHost),
            "both" => Some(Self::Both),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SyncConflictPolicy {
    #[default]
    PreferHost,
    PreferWorld,
    Abort,
}

impl SyncConflictPolicy {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "prefer_host" => Some(Self::PreferHost),
            "prefer_world" => Some(Self::PreferWorld),
            "abort" => Some(Self::Abort),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SyncConfig {
    pub auto_sync: bool,
    pub direction: SyncDirection,
    pub conflict_policy: SyncConflictPolicy,
    pub exclude: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsConfig {
    pub enabled: bool,
    pub defaults: AgentDefaultsConfig,
    pub hub: AgentHubConfig,
    pub toolbox: AgentToolboxConfig,
    pub connection_profiles: BTreeMap<String, AgentConnectionProfileConfigV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsConfig {
    pub execution: AgentDefaultsExecutionConfig,
    pub cli: AgentDefaultsCliConfig,
    pub selection: AgentSelectionTupleV1,
}

impl Default for AgentDefaultsConfig {
    fn default() -> Self {
        Self {
            execution: Default::default(),
            cli: Default::default(),
            selection: AgentSelectionTupleV1 {
                harness: AgentHarnessFamilyV1::Codex,
                provider: "openai".into(),
                connection_profile: "built_in:codex-subscription".into(),
                model: "codex-mini-latest".into(),
            },
        }
    }
}

/// User-authored profile fields; source provenance is assigned by the loader.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct AgentConnectionProfileConfigV1 {
    pub revision: String,
    pub provider: String,
    pub billing: AgentConnectionBillingV1,
    pub endpoint_identity: String,
    pub upstream_base_url: String,
    pub auth: AgentConnectionAuthV1,
}

impl AgentConnectionProfileConfigV1 {
    fn bind(&self, id: &str, source: &AgentSelectionSourceV1) -> AgentConnectionProfileV1 {
        AgentConnectionProfileV1 {
            id: id.into(),
            revision: self.revision.clone(),
            provider: self.provider.clone(),
            billing: self.billing,
            endpoint_identity: self.endpoint_identity.clone(),
            upstream_base_url: self.upstream_base_url.clone(),
            auth: self.auth.clone(),
            source: source.clone(),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsExecutionConfig {
    pub scope: AgentExecutionScope,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentExecutionScope {
    Host,
    #[default]
    World,
}

impl AgentExecutionScope {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "host" => Some(Self::Host),
            "world" => Some(Self::World),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsCliConfig {
    pub mode: AgentCliMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentCliMode {
    #[default]
    Persistent,
    PerRequest,
}

impl AgentCliMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "persistent" => Some(Self::Persistent),
            "per_request" => Some(Self::PerRequest),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentToolboxConfig {
    pub enabled: bool,
    pub bind: AgentToolboxBindConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentToolboxBindConfig {
    pub transport: AgentToolboxBindTransport,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum AgentToolboxBindTransport {
    #[default]
    Uds,
    Tcp,
}

impl AgentToolboxBindTransport {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "uds" => Some(Self::Uds),
            "tcp" => Some(Self::Tcp),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentHubConfig {
    pub orchestrator_agent_id: String,
    pub world_restart: AgentHubWorldRestartConfig,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentHubWorldRestartConfig {
    pub on_drift: WorldRestartOnDriftMode,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum WorldRestartOnDriftMode {
    #[default]
    AutoRestart,
    FailClosed,
}

impl WorldRestartOnDriftMode {
    pub(crate) fn parse_insensitive(raw: &str) -> Option<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "auto_restart" => Some(Self::AutoRestart),
            "fail_closed" => Some(Self::FailClosed),
            _ => None,
        }
    }
}

#[derive(Debug, Default, Clone)]
pub(crate) struct CliConfigOverrides {
    pub world_enabled: Option<bool>,
    pub anchor_mode: Option<WorldRootMode>,
    pub anchor_path: Option<String>,
    pub caged: Option<bool>,
}

#[cfg(target_os = "linux")]
#[allow(
    dead_code,
    reason = "E3-C source authoring precedes E3 runtime adoption"
)]
enum E3PinnedConfigPatchSourceV1<'authority> {
    WorkspaceAbsent,
    Global {
        opened: crate::execution::agent_runtime::host_session_authority::facade::OpenedBootstrapConfigSourceV1<'authority>,
        source_root: config_projection::CanonicalDirectoryV1,
        source_relative_path: String,
        source_bytes_sha256: Option<String>,
    },
    Workspace {
        source_root: config_projection::CanonicalDirectoryV1,
        held_source_root: File,
        source_relative_path: String,
        held_file: File,
        file_device_id: u64,
        file_inode: u64,
        file_byte_length: u64,
        source_bytes: Vec<u8>,
        source_bytes_sha256: String,
    },
}

#[cfg(target_os = "linux")]
#[allow(
    dead_code,
    reason = "E3-C source authoring precedes E3 runtime adoption"
)]
pub(super) struct E3EffectiveConfigResolutionSnapshotV1<'authority> {
    effective: SubstrateConfig,
    explain: ConfigExplainV1,
    global_patch: E3PinnedConfigPatchSourceV1<'authority>,
    workspace_patch: E3PinnedConfigPatchSourceV1<'authority>,
    workspace_selection: E3WorkspaceConfigSelectionGuardV1,
}

#[cfg(target_os = "linux")]
impl E3EffectiveConfigResolutionSnapshotV1<'_> {
    pub(super) fn effective_config(&self) -> &SubstrateConfig {
        &self.effective
    }

    pub(super) fn revalidate(&self) -> Result<()> {
        let E3PinnedConfigPatchSourceV1::Global { opened, .. } = &self.global_patch else {
            return Err(user_error("E3 global source posture is malformed"));
        };
        opened
            .revalidate()
            .map_err(|error| user_error(error.to_string()))?;
        self.workspace_selection.revalidate()
    }
}

#[cfg(target_os = "linux")]
#[derive(Debug, PartialEq, Eq)]
enum E3ConfigEntryObservationV1 {
    Missing,
    Present {
        device: u64,
        inode: u64,
        mode: u32,
        owner: u32,
        group: u32,
        // Directory contents unrelated to selection may change. File metadata may not.
        content: Option<(u64, u64, i64, i64, i64, i64)>,
    },
}

#[cfg(target_os = "linux")]
fn observe_e3_config_entry_v1(directory: &File, name: &str) -> Result<E3ConfigEntryObservationV1> {
    let name = std::ffi::CString::new(name)?;
    let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: the held descriptor, component and output pointer are valid.
    if unsafe {
        libc::fstatat(
            directory.as_raw_fd(),
            name.as_ptr(),
            stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        let error = io::Error::last_os_error();
        return if error.raw_os_error() == Some(libc::ENOENT) {
            Ok(E3ConfigEntryObservationV1::Missing)
        } else {
            Err(error).context("failed to inspect configuration selection entry")
        };
    }
    // SAFETY: fstatat initialized stat on success.
    let stat = unsafe { stat.assume_init() };
    if stat.st_mode & libc::S_IFMT == libc::S_IFLNK {
        return Err(user_error("symlink in configuration selection"));
    }
    Ok(E3ConfigEntryObservationV1::Present {
        device: stat.st_dev,
        inode: stat.st_ino,
        mode: stat.st_mode,
        owner: stat.st_uid,
        group: stat.st_gid,
        content: (stat.st_mode & libc::S_IFMT != libc::S_IFDIR).then_some((
            stat.st_nlink,
            stat.st_size as u64,
            stat.st_mtime,
            stat.st_mtime_nsec,
            stat.st_ctime,
            stat.st_ctime_nsec,
        )),
    })
}

#[cfg(target_os = "linux")]
fn open_e3_config_directory_v1(path: &Path) -> Result<File> {
    use std::os::unix::ffi::OsStrExt;
    if !path.is_absolute() {
        return Err(user_error(
            "configuration guard requires an absolute directory",
        ));
    }
    let mut held = File::open("/")?;
    for component in path.components() {
        match component {
            std::path::Component::RootDir => continue,
            std::path::Component::Normal(name) => {
                let name = std::ffi::CString::new(name.as_bytes())?;
                // SAFETY: held descriptor and name are valid. Every component rejects symlinks.
                let fd = unsafe {
                    libc::openat(
                        held.as_raw_fd(),
                        name.as_ptr(),
                        libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
                    )
                };
                if fd < 0 {
                    return Err(io::Error::last_os_error())
                        .context("untrusted configuration directory traversal");
                }
                // SAFETY: successful openat returned a new owned descriptor.
                held = unsafe { File::from_raw_fd(fd) };
            }
            _ => return Err(user_error("configuration directory is not normalized")),
        }
    }
    Ok(held)
}

#[cfg(target_os = "linux")]
struct E3ConfigDirectoryObservationV1 {
    path: PathBuf,
    held: File,
    identity: E3ConfigEntryObservationV1,
    entries: Vec<(&'static str, E3ConfigEntryObservationV1)>,
}

#[cfg(target_os = "linux")]
struct E3WorkspaceConfigSelectionGuardV1 {
    authenticated_workspace: config_projection::CanonicalDirectoryV1,
    held_workspace: File,
    start: PathBuf,
    directories: Vec<E3ConfigDirectoryObservationV1>,
    selected_root: Option<PathBuf>,
}

#[cfg(target_os = "linux")]
impl E3WorkspaceConfigSelectionGuardV1 {
    fn capture(cwd: &Path, expected: &config_projection::CanonicalDirectoryV1) -> Result<Self> {
        let held_workspace = open_e3_config_directory_v1(Path::new(&expected.physical_path))?;
        let meta = held_workspace.metadata()?;
        let identity = config_projection::DirectoryPhysicalIdentityV1::Linux {
            device_id: meta.dev(),
            inode: meta.ino(),
        };
        if identity != expected.physical_identity {
            return Err(user_error("authenticated workspace identity changed"));
        }
        let start = if cwd.is_absolute() {
            cwd.to_path_buf()
        } else {
            env::current_dir()?.join(cwd)
        };
        // Prove the start before selecting an ancestor; never use canonicalization fallback.
        let held_start = open_e3_config_directory_v1(&start)?;
        let canonical_start = cwd.canonicalize()?;
        if canonical_start != start || !start.starts_with(&expected.physical_path) {
            return Err(user_error(
                "configuration start is outside the authenticated workspace",
            ));
        }
        drop(held_start);
        let mut guard = Self {
            authenticated_workspace: expected.clone(),
            held_workspace,
            start,
            directories: Vec::new(),
            selected_root: None,
        };
        for path in guard.start.ancestors() {
            let held = open_e3_config_directory_v1(path)?;
            let identity = observe_e3_config_entry_v1(&held, ".")?;
            let substrate = observe_e3_config_entry_v1(&held, ".substrate")?;
            let is_directory = matches!(&substrate, E3ConfigEntryObservationV1::Present { mode, .. } if mode & libc::S_IFMT == libc::S_IFDIR);
            guard.directories.push(E3ConfigDirectoryObservationV1 {
                path: path.to_path_buf(),
                held,
                identity,
                entries: vec![(".substrate", substrate)],
            });
            if !is_directory {
                continue;
            }
            let substrate_path = path.join(".substrate");
            let held = open_e3_config_directory_v1(&substrate_path)?;
            let identity = observe_e3_config_entry_v1(&held, ".")?;
            if identity != guard.directories.last().unwrap().entries[0].1 {
                return Err(user_error(
                    "configuration control directory changed during selection",
                ));
            }
            let marker = observe_e3_config_entry_v1(&held, "workspace.yaml")?;
            let disabled = observe_e3_config_entry_v1(&held, "workspace.disabled")?;
            let selected = matches!(&marker, E3ConfigEntryObservationV1::Present { mode, .. } if mode & libc::S_IFMT == libc::S_IFREG)
                && disabled == E3ConfigEntryObservationV1::Missing;
            let mut entries = vec![("workspace.yaml", marker), ("workspace.disabled", disabled)];
            if selected {
                let legacy = observe_e3_config_entry_v1(&held, "settings.yaml")?;
                if legacy != E3ConfigEntryObservationV1::Missing {
                    return Err(user_error(
                        "unsupported workspace control entry is present: .substrate/settings.yaml",
                    ));
                }
                entries.push(("settings.yaml", legacy));
                guard.selected_root = Some(path.to_path_buf());
            }
            guard.directories.push(E3ConfigDirectoryObservationV1 {
                path: substrate_path,
                held,
                identity,
                entries,
            });
            if selected {
                break;
            }
        }
        guard.revalidate()?;
        Ok(guard)
    }

    fn revalidate(&self) -> Result<()> {
        let workspace =
            open_e3_config_directory_v1(Path::new(&self.authenticated_workspace.physical_path))?;
        let held = self.held_workspace.metadata()?;
        let named = workspace.metadata()?;
        if held.dev() != named.dev()
            || held.ino() != named.ino()
            || self.authenticated_workspace.physical_identity
                != (config_projection::DirectoryPhysicalIdentityV1::Linux {
                    device_id: named.dev(),
                    inode: named.ino(),
                })
        {
            return Err(user_error("authenticated workspace identity changed"));
        }
        for directory in &self.directories {
            let named = open_e3_config_directory_v1(&directory.path)?;
            if observe_e3_config_entry_v1(&named, ".")? != directory.identity
                || observe_e3_config_entry_v1(&directory.held, ".")? != directory.identity
            {
                return Err(user_error("configuration selection directory changed"));
            }
            for (name, observed) in &directory.entries {
                if &observe_e3_config_entry_v1(&directory.held, name)? != observed {
                    return Err(user_error("configuration selection source changed"));
                }
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CapturedSelectionConfigSourceV1 {
    Absent,
    Present(AgentSelectionSourceV1),
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SubstrateConfigPatch {
    #[serde(skip)]
    pub source_sha256: Option<String>,
    // None means caller-held composition; Absent is a source-owner observation.
    #[serde(skip)]
    captured_selection_source: Option<CapturedSelectionConfigSourceV1>,
    #[serde(skip_serializing_if = "WorldConfigPatch::is_empty")]
    pub world: WorldConfigPatch,
    #[serde(skip_serializing_if = "PolicyConfigPatch::is_empty")]
    pub policy: PolicyConfigPatch,
    #[serde(skip_serializing_if = "SyncConfigPatch::is_empty")]
    pub sync: SyncConfigPatch,
    #[serde(skip_serializing_if = "ReplConfigPatch::is_empty")]
    pub repl: ReplConfigPatch,
    #[serde(skip_serializing_if = "LlmConfigPatch::is_empty")]
    pub llm: LlmConfigPatch,
    #[serde(skip_serializing_if = "AgentsConfigPatch::is_empty")]
    pub agents: AgentsConfigPatch,
}

impl SubstrateConfigPatch {
    pub(crate) fn is_empty(&self) -> bool {
        self.world.is_empty()
            && self.policy.is_empty()
            && self.sync.is_empty()
            && self.repl.is_empty()
            && self.llm.is_empty()
            && self.agents.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_mode: Option<WorldRootMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub anchor_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caged: Option<bool>,
    #[serde(skip_serializing_if = "WorldNetConfigPatch::is_empty")]
    pub net: WorldNetConfigPatch,
    #[serde(skip_serializing_if = "WorldEnvConfigPatch::is_empty")]
    pub env: WorldEnvConfigPatch,
    #[serde(skip_serializing_if = "WorldDepsConfigPatch::is_empty")]
    pub deps: WorldDepsConfigPatch,
}

impl WorldConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.anchor_mode.is_none()
            && self.anchor_path.is_none()
            && self.caged.is_none()
            && self.net.is_empty()
            && self.env.is_empty()
            && self.deps.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldNetConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<bool>,
}

impl WorldNetConfigPatch {
    fn is_empty(&self) -> bool {
        self.filter.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldEnvConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inherit_from_host: Option<bool>,
}

impl WorldEnvConfigPatch {
    fn is_empty(&self) -> bool {
        self.inherit_from_host.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct WorldDepsConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inventory_mode: Option<WorldDepsInventoryMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub builtins: Option<WorldDepsBuiltinsMode>,
}

impl WorldDepsConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.inventory_mode.is_none() && self.builtins.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct PolicyConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<PolicyMode>,
}

impl PolicyConfigPatch {
    fn is_empty(&self) -> bool {
        self.mode.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct SyncConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_sync: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub direction: Option<SyncDirection>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conflict_policy: Option<SyncConflictPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exclude: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct ReplConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_cwd: Option<ReplExitCwdMode>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub max_pty_buffered_lines: Option<i64>,
}

impl ReplConfigPatch {
    fn is_empty(&self) -> bool {
        self.exit_cwd.is_none() && self.max_pty_buffered_lines.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "LlmGatewayConfigPatch::is_empty")]
    pub gateway: LlmGatewayConfigPatch,
    #[serde(skip_serializing_if = "LlmRoutingConfigPatch::is_empty")]
    pub routing: LlmRoutingConfigPatch,
}

impl LlmConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.gateway.is_empty() && self.routing.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmGatewayConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<LlmGatewayMode>,
}

impl LlmGatewayConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.mode.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct LlmRoutingConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_backend: Option<String>,
}

impl LlmRoutingConfigPatch {
    fn is_empty(&self) -> bool {
        self.default_backend.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentsConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "AgentDefaultsConfigPatch::is_empty")]
    pub defaults: AgentDefaultsConfigPatch,
    #[serde(skip_serializing_if = "AgentHubConfigPatch::is_empty")]
    pub hub: AgentHubConfigPatch,
    #[serde(skip_serializing_if = "AgentToolboxConfigPatch::is_empty")]
    pub toolbox: AgentToolboxConfigPatch,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub connection_profiles: BTreeMap<String, AgentConnectionProfileConfigV1>,
}

impl AgentsConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none()
            && self.connection_profiles.is_empty()
            && self.defaults.is_empty()
            && self.hub.is_empty()
            && self.toolbox.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection: Option<AgentSelectionTupleV1>,
    #[serde(skip_serializing_if = "AgentDefaultsExecutionConfigPatch::is_empty")]
    pub execution: AgentDefaultsExecutionConfigPatch,
    #[serde(skip_serializing_if = "AgentDefaultsCliConfigPatch::is_empty")]
    pub cli: AgentDefaultsCliConfigPatch,
}

impl AgentDefaultsConfigPatch {
    fn is_empty(&self) -> bool {
        self.selection.is_none() && self.execution.is_empty() && self.cli.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsExecutionConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scope: Option<AgentExecutionScope>,
}

impl AgentDefaultsExecutionConfigPatch {
    fn is_empty(&self) -> bool {
        self.scope.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentDefaultsCliConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<AgentCliMode>,
}

impl AgentDefaultsCliConfigPatch {
    fn is_empty(&self) -> bool {
        self.mode.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentHubConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub orchestrator_agent_id: Option<String>,
    #[serde(skip_serializing_if = "AgentHubWorldRestartConfigPatch::is_empty")]
    pub world_restart: AgentHubWorldRestartConfigPatch,
}

impl AgentHubConfigPatch {
    fn is_empty(&self) -> bool {
        self.orchestrator_agent_id.is_none() && self.world_restart.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentHubWorldRestartConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_drift: Option<WorldRestartOnDriftMode>,
}

impl AgentHubWorldRestartConfigPatch {
    fn is_empty(&self) -> bool {
        self.on_drift.is_none()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentToolboxConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enabled: Option<bool>,
    #[serde(skip_serializing_if = "AgentToolboxBindConfigPatch::is_empty")]
    pub bind: AgentToolboxBindConfigPatch,
}

impl AgentToolboxConfigPatch {
    fn is_empty(&self) -> bool {
        self.enabled.is_none() && self.bind.is_empty()
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(default, deny_unknown_fields)]
pub(crate) struct AgentToolboxBindConfigPatch {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transport: Option<AgentToolboxBindTransport>,
}

impl AgentToolboxBindConfigPatch {
    fn is_empty(&self) -> bool {
        self.transport.is_none()
    }
}

impl SyncConfigPatch {
    fn is_empty(&self) -> bool {
        self.auto_sync.is_none()
            && self.direction.is_none()
            && self.conflict_policy.is_none()
            && self.exclude.is_none()
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainV1 {
    pub kind: String,
    pub keys: OrderedExplainKeys,
}

impl ConfigExplainV1 {
    pub(crate) fn world_enabled_explain(&self) -> Option<&ConfigExplainKey> {
        self.keys.0.get("world.enabled")
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainKey {
    pub merge_strategy: String,
    pub sources: Vec<ConfigExplainSource>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct ConfigExplainSource {
    pub layer: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorldDisableSource {
    pub key: &'static str,
    pub layer: &'static str,
    pub value_display: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub flag: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path_display: Option<&'static str>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub(crate) struct WorldDisableAttribution {
    pub reason: &'static str,
    pub source: WorldDisableSource,
}

pub(crate) type DoctorDisableSource = WorldDisableSource;
pub(crate) type DoctorDisableAttribution = WorldDisableAttribution;

#[allow(dead_code)]
pub(crate) fn world_disable_attribution_message(
    world_enabled: bool,
    world_enabled_explain: Option<&ConfigExplainKey>,
) -> Option<&'static str> {
    world_disable_attribution(world_enabled, world_enabled_explain)
        .map(|attribution| attribution.reason)
}

pub(crate) fn world_disable_attribution(
    world_enabled: bool,
    world_enabled_explain: Option<&ConfigExplainKey>,
) -> Option<WorldDisableAttribution> {
    if world_enabled {
        return None;
    }

    let Some(explain) = world_enabled_explain else {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    };

    let Some(source) = explain.sources.as_slice().first() else {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    };

    if explain.sources.len() != 1 {
        return Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        });
    }

    match source.layer.as_str() {
        "cli_flag" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by CLI flag --no-world",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "cli_flag",
                value_display: false,
                flag: Some("--no-world"),
                env: None,
                path_display: None,
            },
        }),
        "override_env" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "override_env",
                value_display: false,
                flag: None,
                env: Some("SUBSTRATE_OVERRIDE_WORLD"),
                path_display: None,
            },
        }),
        "workspace_patch" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "workspace_patch",
                value_display: false,
                flag: None,
                env: None,
                path_display: Some("<workspace>/.substrate/workspace.yaml"),
            },
        }),
        "global_patch" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "global_patch",
                value_display: false,
                flag: None,
                env: None,
                path_display: Some("$SUBSTRATE_HOME/config.yaml"),
            },
        }),
        "default" => Some(WorldDisableAttribution {
            reason: "world isolation disabled by default config (world.enabled: false)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "default",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        }),
        _ => Some(WorldDisableAttribution {
            reason: "world isolation disabled by effective config (source unknown)",
            source: WorldDisableSource {
                key: "world.enabled",
                layer: "source_unknown",
                value_display: false,
                flag: None,
                env: None,
                path_display: None,
            },
        }),
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) struct OrderedExplainKeys(BTreeMap<String, ConfigExplainKey>);

impl OrderedExplainKeys {
    fn insert(&mut self, key: String, value: ConfigExplainKey) {
        self.0.insert(key, value);
    }

    #[cfg(test)]
    fn get(&self, key: &str) -> Option<&ConfigExplainKey> {
        self.0.get(key)
    }
}

impl Serialize for OrderedExplainKeys {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        // Deterministic bytes are required, but we also keep the output stable for simple
        // string-scanning consumers by ensuring global-layer entries serialize before
        // workspace-layer entries.
        let mut entries: Vec<_> = self.0.iter().collect();
        entries.sort_by(|(a_key, a_val), (b_key, b_val)| {
            explain_key_rank(a_val)
                .cmp(&explain_key_rank(b_val))
                .then_with(|| a_key.cmp(b_key))
        });

        let mut map = serializer.serialize_map(Some(entries.len()))?;
        for (key, value) in entries {
            map.serialize_entry(key, value)?;
        }
        map.end()
    }
}

fn explain_key_rank(key: &ConfigExplainKey) -> u8 {
    key.sources
        .iter()
        .map(|source| match source.layer.as_str() {
            "global_patch" => 0,
            "default" => 1,
            "workspace_patch" => 2,
            "override_env" => 3,
            "cli_flag" => 4,
            _ => 5,
        })
        .min()
        .unwrap_or(5)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum UpdateOp {
    Set,
    Append,
    Remove,
}

#[derive(Debug, Clone)]
pub(crate) struct ConfigUpdate {
    pub key: String,
    pub op: UpdateOp,
    pub value: String,
}

pub(crate) fn global_config_path() -> Result<PathBuf> {
    substrate_paths::config_file()
}

pub(crate) fn read_global_config_patch_or_empty() -> Result<(SubstrateConfigPatch, bool)> {
    let path = global_config_path()?;
    match fs::read_to_string(&path) {
        Ok(raw) => Ok((
            parse_captured_config_patch(&path, &raw, AgentSelectionSourceScopeV1::Global)?,
            true,
        )),
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            Ok((captured_absent_config_patch(), false))
        }
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

pub(crate) fn read_global_config_or_defaults() -> Result<(SubstrateConfig, bool)> {
    let (patch, existed) = read_global_config_patch_or_empty()?;
    let cfg = resolve_effective_from_layers(
        &patch,
        &global_config_path()?,
        None,
        &EnvOverrides::default(),
        &CliConfigOverrides::default(),
        false,
        false,
    )?
    .0;
    Ok((cfg, existed))
}

pub(crate) fn parse_config_yaml(path: &Path, raw: &str) -> Result<SubstrateConfig> {
    // Preserve raw provenance before serde can expand aliases. A failed scan
    // cannot establish that protected input is absent.
    validate_config_yaml_selection_paths_v1(path, raw)
        .map_err(|err| err.context(ProtectedSelectionConfigErrorV1))?;
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
        .context(ProtectedSelectionConfigErrorV1)
    })?;

    match &value {
        serde_yaml::Value::Null => return Ok(SubstrateConfig::default()),
        serde_yaml::Value::Mapping(map) if map.is_empty() => return Ok(SubstrateConfig::default()),
        _ => {}
    }

    // Match the raw guard's destination paths even in a malformed sequence
    // container, so a schema error cannot hide a protected selection.
    let mut pending = vec![(&value, ConfigYamlPathV1::Root)];
    let mut protected_input = false;
    while let Some((node, node_path)) = pending.pop() {
        if node_path == ConfigYamlPathV1::Strict {
            protected_input = true;
            break;
        }
        match node {
            serde_yaml::Value::Mapping(map) => {
                for (key, child) in map {
                    if let Some(key) = key.as_str() {
                        pending.push((child, config_yaml_child_path_v1(node_path, key)));
                    }
                }
            }
            serde_yaml::Value::Sequence(items) => {
                pending.extend(items.iter().map(|item| (item, node_path)));
            }
            serde_yaml::Value::Tagged(tagged) => pending.push((&tagged.value, node_path)),
            _ => {}
        }
    }
    let classify_error = |err: anyhow::Error| {
        if protected_input {
            err.context(ProtectedSelectionConfigErrorV1)
        } else {
            err
        }
    };
    let parsed: SubstrateConfig = serde_yaml::from_value(value).map_err(|err| {
        classify_error(user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        )))
    })?;
    validate_local_config(&parsed).map_err(classify_error)?;
    // This full parser's consumers read selected-home config.yaml. Workspace
    // settings retain their separately scoped patch parser and composition.
    for (id, input) in &parsed.agents.connection_profiles {
        if !id.starts_with("global:") {
            return Err(user_error(format!(
                "connection profile '{}' in {} cannot redefine another source namespace",
                id,
                path.display()
            ))
            .context(ProtectedSelectionConfigErrorV1));
        }
        AgentConnectionProfileV1::validate_fields(
            id,
            &input.revision,
            &input.provider,
            input.billing,
            &input.endpoint_identity,
            &input.upstream_base_url,
            &input.auth,
        )
        .map_err(|error| {
            user_error(format!(
                "invalid connection profile '{}' in {}: {}",
                id,
                path.display(),
                error
            ))
            .context(ProtectedSelectionConfigErrorV1)
        })?;
    }
    Ok(parsed)
}

// This fixed-path event graph retains references and annotations; it never
// constructs expanded YAML values or acquires a configuration source.
struct ConfigYamlNodeV1 {
    kind: ConfigYamlNodeKindV1,
    marked: bool,
    line: usize,
}

enum ConfigYamlNodeKindV1 {
    Scalar(String),
    Mapping(Vec<(usize, usize)>),
    Sequence(Vec<usize>),
    Alias(usize),
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ConfigYamlPathV1 {
    Root,
    Agents,
    Defaults,
    Strict,
    Other,
}

#[derive(Default)]
struct ConfigYamlGraphV1 {
    nodes: Vec<ConfigYamlNodeV1>,
    anchors: BTreeMap<usize, usize>,
}

impl ConfigYamlGraphV1 {
    fn read_node(
        &mut self,
        parser: &mut yaml_rust2::parser::Parser<std::str::Chars<'_>>,
        depth: usize,
    ) -> Result<usize, String> {
        use yaml_rust2::parser::Event;
        if depth > 256 {
            return Err("YAML nesting is too deep".into());
        }
        let (event, mark) = parser.next_token().map_err(|error| error.to_string())?;
        let (kind, anchor, tagged) = match event {
            Event::Scalar(value, _, anchor, tag) => {
                (ConfigYamlNodeKindV1::Scalar(value), anchor, tag.is_some())
            }
            Event::MappingStart(anchor, tag) => (
                ConfigYamlNodeKindV1::Mapping(Vec::new()),
                anchor,
                tag.is_some(),
            ),
            Event::SequenceStart(anchor, tag) => (
                ConfigYamlNodeKindV1::Sequence(Vec::new()),
                anchor,
                tag.is_some(),
            ),
            Event::Alias(anchor) => (ConfigYamlNodeKindV1::Alias(anchor), 0, false),
            _ => return Err(format!("expected YAML value at line {}", mark.line())),
        };
        let mapping = matches!(&kind, ConfigYamlNodeKindV1::Mapping(_));
        let sequence = matches!(&kind, ConfigYamlNodeKindV1::Sequence(_));
        let index = self.nodes.len();
        self.nodes.push(ConfigYamlNodeV1 {
            kind,
            marked: anchor != 0 || tagged,
            line: mark.line(),
        });
        if anchor != 0 {
            self.anchors.insert(anchor, index);
        }
        if mapping {
            let mut entries = Vec::new();
            while !matches!(
                &parser.peek().map_err(|error| error.to_string())?.0,
                Event::MappingEnd
            ) {
                let key = self.read_node(parser, depth + 1)?;
                let value = self.read_node(parser, depth + 1)?;
                entries.push((key, value));
            }
            parser.next_token().map_err(|error| error.to_string())?;
            self.nodes[index].kind = ConfigYamlNodeKindV1::Mapping(entries);
        } else if sequence {
            let mut items = Vec::new();
            while !matches!(
                &parser.peek().map_err(|error| error.to_string())?.0,
                Event::SequenceEnd
            ) {
                items.push(self.read_node(parser, depth + 1)?);
            }
            parser.next_token().map_err(|error| error.to_string())?;
            self.nodes[index].kind = ConfigYamlNodeKindV1::Sequence(items);
        }
        Ok(index)
    }

    fn key(
        &self,
        index: usize,
        visiting: &mut std::collections::BTreeSet<usize>,
    ) -> Result<(String, bool), String> {
        if !visiting.insert(index) || visiting.len() > 256 {
            return Err("recursive YAML key reference".into());
        }
        let node = &self.nodes[index];
        match &node.kind {
            ConfigYamlNodeKindV1::Scalar(value) => Ok((value.clone(), node.marked)),
            ConfigYamlNodeKindV1::Alias(anchor) => {
                let target = self
                    .anchors
                    .get(anchor)
                    .ok_or("unknown YAML key reference")?;
                let (key, _) = self.key(*target, visiting)?;
                Ok((key, true))
            }
            _ => Err(format!(
                "mapping keys must be scalar strings at line {}",
                node.line
            )),
        }
    }

    fn validate_node(
        &self,
        index: usize,
        path: ConfigYamlPathV1,
        tainted: bool,
        active: &mut std::collections::BTreeSet<usize>,
        seen: &mut std::collections::BTreeSet<(usize, ConfigYamlPathV1, bool)>,
    ) -> Result<(), String> {
        let node = &self.nodes[index];
        if path == ConfigYamlPathV1::Strict
            && (tainted || node.marked || matches!(&node.kind, ConfigYamlNodeKindV1::Alias(_)))
        {
            return Err(format!("selection/profile YAML provenance at line {} uses an anchor, alias, tag, merge or referenced parent", node.line));
        }
        if seen.contains(&(index, path, tainted)) {
            return Ok(());
        }
        if !active.insert(index) || active.len() > 256 {
            return Err("recursive YAML reference".into());
        }
        let inherited = tainted || node.marked;
        match &node.kind {
            ConfigYamlNodeKindV1::Alias(anchor) => {
                let target = self.anchors.get(anchor).ok_or("unknown YAML reference")?;
                // Inspect the reference under its destination path without
                // expanding/replacing the source graph or discarding origin.
                self.validate_node(*target, path, true, active, seen)?;
            }
            ConfigYamlNodeKindV1::Mapping(entries) => {
                let mut keys = std::collections::BTreeSet::new();
                for (key, value) in entries {
                    let (name, key_marked) =
                        self.key(*key, &mut std::collections::BTreeSet::new())?;
                    if !keys.insert(name.clone()) {
                        return Err(format!(
                            "duplicate YAML mapping key at line {}",
                            self.nodes[*key].line
                        ));
                    }
                    if name == "<<" {
                        if path == ConfigYamlPathV1::Strict {
                            return Err(format!(
                                "selection/profile YAML provenance at line {} uses a merge",
                                self.nodes[*key].line
                            ));
                        }
                        // A merge contributes at the current mapping path,
                        // including when its value is a sequence of references.
                        self.validate_node(*value, path, true, active, seen)?;
                    } else {
                        self.validate_node(
                            *value,
                            config_yaml_child_path_v1(path, &name),
                            inherited || key_marked,
                            active,
                            seen,
                        )?;
                    }
                }
            }
            ConfigYamlNodeKindV1::Sequence(items) => {
                for item in items {
                    self.validate_node(*item, path, inherited, active, seen)?;
                }
            }
            ConfigYamlNodeKindV1::Scalar(_) => {}
        }
        active.remove(&index);
        seen.insert((index, path, tainted));
        Ok(())
    }
}

fn config_yaml_child_path_v1(path: ConfigYamlPathV1, key: &str) -> ConfigYamlPathV1 {
    use ConfigYamlPathV1::{Agents, Defaults, Other, Root, Strict};
    match (path, key) {
        (Root, "agents") => Agents,
        (Agents, "defaults") => Defaults,
        (Agents, "connection_profiles") | (Defaults, "selection") | (Strict, _) => Strict,
        _ => Other,
    }
}

fn validate_config_yaml_selection_paths_v1(path: &Path, raw: &str) -> Result<()> {
    use yaml_rust2::parser::{Event, Parser};
    let validate = || -> Result<(), String> {
        let mut parser = Parser::new_from_str(raw);
        if parser.next_token().map_err(|error| error.to_string())?.0 != Event::StreamStart {
            return Err("expected YAML stream".into());
        }
        let next = parser.next_token().map_err(|error| error.to_string())?.0;
        if next == Event::StreamEnd {
            return Ok(());
        }
        if next != Event::DocumentStart {
            return Err("expected YAML document".into());
        }
        let mut graph = ConfigYamlGraphV1::default();
        let root = graph.read_node(&mut parser, 0)?;
        if parser.next_token().map_err(|error| error.to_string())?.0 != Event::DocumentEnd
            || parser.next_token().map_err(|error| error.to_string())?.0 != Event::StreamEnd
        {
            return Err("multiple YAML documents are forbidden".into());
        }
        graph.validate_node(
            root,
            ConfigYamlPathV1::Root,
            false,
            &mut std::collections::BTreeSet::new(),
            &mut std::collections::BTreeSet::new(),
        )
    };
    validate().map_err(|error| user_error(format!("invalid YAML in {}: {}", path.display(), error)))
}

pub(crate) fn parse_config_patch_yaml(path: &Path, raw: &str) -> Result<SubstrateConfigPatch> {
    validate_config_yaml_selection_paths_v1(path, raw)?;
    let value: serde_yaml::Value = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;

    match &value {
        serde_yaml::Value::Null => {
            return Ok(SubstrateConfigPatch {
                source_sha256: Some(format!("{:x}", Sha256::digest(raw.as_bytes()))),
                ..Default::default()
            })
        }
        serde_yaml::Value::Mapping(map) if map.is_empty() => {
            return Ok(SubstrateConfigPatch {
                source_sha256: Some(format!("{:x}", Sha256::digest(raw.as_bytes()))),
                ..Default::default()
            })
        }
        _ => {}
    }

    let mut parsed: SubstrateConfigPatch = serde_yaml::from_str(raw).map_err(|err| {
        user_error(format!(
            "invalid YAML in {}: {}",
            path.display(),
            err.to_string().trim()
        ))
    })?;
    parsed.source_sha256 = Some(format!("{:x}", Sha256::digest(raw.as_bytes())));
    validate_config_patch(&parsed)?;
    Ok(parsed)
}

/// Inputs captured once at the native invocation boundary; deliberately not printable.
pub(crate) struct CapturedConfigOverridesV1 {
    environment: EnvOverrides,
    cli: CliConfigOverrides,
}

impl CapturedConfigOverridesV1 {
    pub(crate) fn capture(cli: &CliConfigOverrides) -> Result<Self> {
        Ok(Self {
            environment: parse_env_overrides()?,
            cli: cli.clone(),
        })
    }
}

/// Fresh composition from caller-held sources. This function never discovers or opens paths.
pub(crate) fn resolve_effective_config_from_sources_v1(
    global_path: &Path,
    global_bytes: Option<&[u8]>,
    workspace_path: &Path,
    workspace_bytes: &[u8],
    overrides: &CapturedConfigOverridesV1,
    policy: &substrate_broker::Policy,
) -> Result<(SubstrateConfig, ConfigExplainV1)> {
    fn parse(path: &Path, bytes: &[u8]) -> Result<SubstrateConfigPatch> {
        let raw = std::str::from_utf8(bytes)
            .map_err(|_| user_error(format!("invalid UTF-8 in {}", path.display())))?;
        parse_config_patch_yaml(path, raw)
    }
    let global_patch = global_bytes
        .map(|bytes| parse(global_path, bytes))
        .transpose()?
        .unwrap_or_default();
    let workspace_patch = parse(workspace_path, workspace_bytes)?;
    let (config, explain) = resolve_effective_from_layers(
        &global_patch,
        global_path,
        Some((&workspace_patch, workspace_path)),
        &overrides.environment,
        &overrides.cli,
        true,
        true,
    )?;
    validate_config_against_policy(&config, policy)?;
    Ok((
        config,
        explain.ok_or_else(|| anyhow!("explicit config explanation missing"))?,
    ))
}

pub(crate) fn resolve_effective_config(
    cwd: &Path,
    cli: &CliConfigOverrides,
) -> Result<SubstrateConfig> {
    Ok(resolve_effective_config_with_explain(cwd, cli, false)?.0)
}

pub(crate) fn resolve_effective_config_with_explain(
    cwd: &Path,
    cli: &CliConfigOverrides,
    explain: bool,
) -> Result<(SubstrateConfig, Option<ConfigExplainV1>)> {
    let env_overrides = parse_env_overrides()?;
    let (global_patch, global_path, workspace_layer) = load_config_patch_layers_cached(cwd)?;
    let workspace_ref = workspace_layer
        .as_ref()
        .map(|(p, path)| (p, path.as_path()));

    let resolved = resolve_effective_from_layers(
        &global_patch,
        &global_path,
        workspace_ref,
        &env_overrides,
        cli,
        explain,
        true,
    )?;
    validate_config_against_effective_policy(cwd, &resolved.0)?;
    Ok(resolved)
}

#[allow(
    dead_code,
    reason = "A1.1e establishes the explicit-home entry point before A1.3 adopts it"
)]
pub(crate) fn resolve_effective_config_for_bootstrap_home(
    cwd: &Path,
    cli: &CliConfigOverrides,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
) -> Result<SubstrateConfig> {
    Ok(
        resolve_effective_config_with_explain_for_bootstrap_home(cwd, cli, bootstrap_home, false)?
            .0,
    )
}

pub(crate) fn resolve_effective_config_with_explain_for_bootstrap_home(
    cwd: &Path,
    cli: &CliConfigOverrides,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'_>,
    explain: bool,
) -> Result<(SubstrateConfig, Option<ConfigExplainV1>)> {
    let global_path = PathBuf::from(
        &bootstrap_home
            .identity()
            .map_err(|error| user_error(error.to_string()))?
            .physical_path,
    )
    .join("config.yaml");
    let global_patch = match bootstrap_home
        .read_config_yaml()
        .map_err(|error| user_error(error.to_string()))?
    {
        Some(bytes) => {
            let raw = std::str::from_utf8(&bytes)
                .map_err(|_| user_error(format!("invalid UTF-8 in {}", global_path.display())))?;
            let mut patch = parse_config_patch_yaml(&global_path, raw)?;
            // The trusted reader, rather than a second path lookup, owns this root.
            let identity = bootstrap_home
                .identity()
                .map_err(|error| user_error(error.to_string()))?;
            use crate::execution::agent_runtime::host_session_authority::schema::DirectoryPhysicalIdentityV1;
            let root_identity = match &identity.physical_identity {
                DirectoryPhysicalIdentityV1::Linux { device_id, inode } => {
                    format!("{}:{}:{}", device_id, inode, identity.physical_path)
                }
                DirectoryPhysicalIdentityV1::MacOs {
                    volume_uuid,
                    file_id,
                    case_sensitive,
                } => format!(
                    "{}:{}:{}:{}",
                    volume_uuid, file_id, case_sensitive, identity.physical_path
                ),
            };
            let hash = patch
                .source_sha256
                .clone()
                .ok_or_else(|| user_error("captured configuration hash missing"))?;
            let source = AgentSelectionSourceV1 {
                scope: AgentSelectionSourceScopeV1::Global,
                root_identity,
                relative_path: "config.yaml".into(),
                revision: hash.clone(),
                sha256: hash,
            };
            source.validate().map_err(user_error)?;
            patch.captured_selection_source =
                Some(CapturedSelectionConfigSourceV1::Present(source));
            patch
        }
        None => captured_absent_config_patch(),
    };
    let workspace_layer = load_workspace_config_patch(cwd)?;
    let workspace_ref = workspace_layer
        .as_ref()
        .map(|(patch, path)| (patch, path.as_path()));
    let resolved = resolve_effective_from_layers(
        &global_patch,
        &global_path,
        workspace_ref,
        &parse_env_overrides()?,
        cli,
        explain,
        true,
    )?;
    if resolved.0.llm.gateway.mode == LlmGatewayMode::HostOnly {
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            cwd,
            bootstrap_home,
        )?;
        validate_config_against_policy(&resolved.0, &policy)?;
    }
    Ok(resolved)
}

#[cfg(target_os = "linux")]
#[allow(
    dead_code,
    reason = "E3-C source authoring precedes E3 runtime adoption"
)]
fn open_e3_workspace_config_source_v1<'authority>(
    cwd: &Path,
    expected_workspace_root: &config_projection::CanonicalDirectoryV1,
) -> Result<(
    E3PinnedConfigPatchSourceV1<'authority>,
    E3WorkspaceConfigSelectionGuardV1,
)> {
    use sha2::{Digest, Sha256};
    use std::ffi::CString;

    let selection = E3WorkspaceConfigSelectionGuardV1::capture(cwd, expected_workspace_root)?;
    let Some(selected) = selection.selected_root.as_ref() else {
        return Ok((E3PinnedConfigPatchSourceV1::WorkspaceAbsent, selection));
    };
    let held_source_root = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(selected)
        .with_context(|| format!("failed to open exact workspace root {}", selected.display()))?;
    let root_metadata = held_source_root
        .metadata()
        .context("failed to inspect opened workspace root")?;
    let source_root = config_projection::CanonicalDirectoryV1 {
        physical_path: selected
            .to_str()
            .ok_or_else(|| user_error("workspace root is not UTF-8"))?
            .to_string(),
        physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
            device_id: root_metadata.dev(),
            inode: root_metadata.ino(),
        },
    };
    let substrate_name = CString::new(".substrate").expect("static component is valid");
    // SAFETY: the retained root descriptor and static component are valid; flags reject symlinks.
    let substrate_fd = unsafe {
        libc::openat(
            held_source_root.as_raw_fd(),
            substrate_name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if substrate_fd < 0 {
        return Err(anyhow!(
            "failed to open workspace .substrate directory: {}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: a successful openat returns a newly owned descriptor.
    let held_substrate = unsafe { File::from_raw_fd(substrate_fd) };
    let substrate_metadata = held_substrate
        .metadata()
        .context("failed to inspect workspace .substrate directory")?;
    if !substrate_metadata.is_dir() || substrate_metadata.dev() != root_metadata.dev() {
        return Err(user_error(
            "workspace .substrate entry is not an on-root directory",
        ));
    }

    for forbidden in ["settings.yaml", "workspace.disabled"] {
        let component = CString::new(forbidden).expect("static component is valid");
        let mut stat = std::mem::MaybeUninit::<libc::stat>::uninit();
        // SAFETY: the descriptor, component, and output pointer are valid.
        let result = unsafe {
            libc::fstatat(
                held_substrate.as_raw_fd(),
                component.as_ptr(),
                stat.as_mut_ptr(),
                libc::AT_SYMLINK_NOFOLLOW,
            )
        };
        if result == 0 {
            return Err(user_error(format!(
                "unsupported workspace control entry is present: .substrate/{forbidden}"
            )));
        }
        let error = io::Error::last_os_error();
        if error.kind() != io::ErrorKind::NotFound {
            return Err(error).with_context(|| {
                format!("failed to inspect workspace control entry .substrate/{forbidden}")
            });
        }
    }

    let file_name = CString::new("workspace.yaml").expect("static component is valid");
    // SAFETY: the retained directory descriptor and static component are valid.
    let file_fd = unsafe {
        libc::openat(
            held_substrate.as_raw_fd(),
            file_name.as_ptr(),
            libc::O_RDONLY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if file_fd < 0 {
        return Err(anyhow!(
            "failed to open workspace configuration: {}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: a successful openat returns a newly owned descriptor.
    let held_file = unsafe { File::from_raw_fd(file_fd) };
    let before = held_file
        .metadata()
        .context("failed to inspect workspace configuration")?;
    if !before.is_file() || before.dev() != root_metadata.dev() || before.nlink() != 1 {
        return Err(user_error(
            "workspace configuration is not a single-linked on-root regular file",
        ));
    }
    let mut source_bytes = Vec::new();
    (&held_file)
        .read_to_end(&mut source_bytes)
        .context("failed to read workspace configuration")?;
    let after = held_file
        .metadata()
        .context("failed to re-inspect workspace configuration")?;
    if before.dev() != after.dev()
        || before.ino() != after.ino()
        || before.len() != after.len()
        || before.modified().ok() != after.modified().ok()
        || usize::try_from(after.len()).ok() != Some(source_bytes.len())
    {
        return Err(user_error(
            "workspace configuration changed while it was read",
        ));
    }
    let mut named_stat = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptor, component, and output pointer are valid.
    if unsafe {
        libc::fstatat(
            held_substrate.as_raw_fd(),
            file_name.as_ptr(),
            named_stat.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(anyhow!(
            "workspace configuration disappeared: {}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: fstatat initialized the output on success.
    let named_stat = unsafe { named_stat.assume_init() };
    if named_stat.st_dev != before.dev()
        || named_stat.st_ino != before.ino()
        || named_stat.st_size < 0
        || named_stat.st_size as u64 != before.len()
    {
        return Err(user_error("workspace configuration named identity changed"));
    }
    if std::str::from_utf8(&source_bytes).is_err() {
        return Err(user_error(format!(
            "invalid UTF-8 in {}/.substrate/workspace.yaml",
            source_root.physical_path
        )));
    }
    let source_bytes_sha256 = format!("{:x}", Sha256::digest(&source_bytes));
    selection.revalidate()?;
    Ok((
        E3PinnedConfigPatchSourceV1::Workspace {
            source_root,
            held_source_root,
            source_relative_path: ".substrate/workspace.yaml".to_string(),
            held_file,
            file_device_id: before.dev(),
            file_inode: before.ino(),
            file_byte_length: before.len(),
            source_bytes,
            source_bytes_sha256,
        },
        selection,
    ))
}

#[cfg(target_os = "linux")]
#[allow(
    dead_code,
    reason = "E3-C source authoring precedes E3 runtime adoption"
)]
pub(super) fn resolve_e3_effective_config_source_v1<'authority>(
    cwd: &Path,
    bootstrap_home: &crate::execution::agent_runtime::OpenedBootstrapHomeV1<'authority>,
    expected_workspace_root: &config_projection::CanonicalDirectoryV1,
) -> Result<E3EffectiveConfigResolutionSnapshotV1<'authority>> {
    use sha2::{Digest, Sha256};

    let opened = bootstrap_home
        .open_e3_config_source()
        .map_err(|error| user_error(error.to_string()))?;
    let physical_root_path = opened
        .physical_root_path()
        .map_err(|error| user_error(error.to_string()))?;
    let held_root = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW)
        .open(physical_root_path)
        .with_context(|| format!("failed to open accepted home {physical_root_path}"))?;
    let root_metadata = held_root
        .metadata()
        .context("failed to inspect accepted home")?;
    let source_root = config_projection::CanonicalDirectoryV1 {
        physical_path: physical_root_path.to_string(),
        physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
            device_id: root_metadata.dev(),
            inode: root_metadata.ino(),
        },
    };
    opened
        .validate_e3_public_root_identity(&source_root)
        .map_err(|error| user_error(error.to_string()))?;
    let global_path = PathBuf::from(&source_root.physical_path).join("config.yaml");
    let global_patch_value = match opened.source_bytes() {
        Some(bytes) => {
            let raw = std::str::from_utf8(bytes)
                .map_err(|_| user_error(format!("invalid UTF-8 in {}", global_path.display())))?;
            parse_config_patch_yaml(&global_path, raw)?
        }
        None => SubstrateConfigPatch::default(),
    };
    let global_patch = E3PinnedConfigPatchSourceV1::Global {
        source_bytes_sha256: opened
            .source_bytes()
            .map(|bytes| format!("{:x}", Sha256::digest(bytes))),
        opened,
        source_root,
        source_relative_path: "config.yaml".to_string(),
    };

    let (workspace_patch, workspace_selection) =
        open_e3_workspace_config_source_v1(cwd, expected_workspace_root)?;
    let workspace_layer = match &workspace_patch {
        E3PinnedConfigPatchSourceV1::Workspace {
            source_root,
            source_relative_path,
            source_bytes,
            ..
        } => {
            let path = PathBuf::from(&source_root.physical_path).join(source_relative_path);
            let raw = std::str::from_utf8(source_bytes)
                .map_err(|_| user_error(format!("invalid UTF-8 in {}", path.display())))?;
            Some((parse_config_patch_yaml(&path, raw)?, path))
        }
        E3PinnedConfigPatchSourceV1::WorkspaceAbsent => None,
        _ => return Err(user_error("E3 workspace source posture is malformed")),
    };

    let env_overrides = parse_env_overrides()?;
    let (effective, explain) = resolve_effective_from_layers(
        &global_patch_value,
        &global_path,
        workspace_layer
            .as_ref()
            .map(|(patch, path)| (patch, path.as_path())),
        &env_overrides,
        &CliConfigOverrides::default(),
        true,
        true,
    )?;
    let explain = explain.ok_or_else(|| user_error("E3 config explanation was not produced"))?;
    if effective.llm.gateway.mode == LlmGatewayMode::HostOnly {
        let policy = crate::execution::policy_model::resolve_effective_policy_for_bootstrap_home(
            cwd,
            bootstrap_home,
        )?;
        validate_config_against_policy(&effective, &policy)?;
    }
    let snapshot = E3EffectiveConfigResolutionSnapshotV1 {
        effective,
        explain,
        global_patch,
        workspace_patch,
        workspace_selection,
    };
    snapshot.revalidate()?;
    Ok(snapshot)
}

#[cfg(target_os = "linux")]
#[allow(
    dead_code,
    reason = "E3-C source authoring precedes E3 runtime adoption"
)]
pub(super) fn extract_e3_effective_config_source_v1(
    authority_store_id: &str,
    snapshot: &E3EffectiveConfigResolutionSnapshotV1<'_>,
) -> Result<config_projection::EffectiveSubstrateConfigSourceV1> {
    use config_projection::{
        ConfigProjectionCodecV1, E3ConfigExplainFileV1, E3ConfigExplainOriginKindV1,
        E3ConfigExplainOriginV1, E3EffectiveConfigInputV1, EffectiveSubstrateConfigSourceV1,
    };
    use sha2::{Digest, Sha256};
    use std::collections::BTreeMap;
    use std::ffi::CString;

    snapshot.revalidate()?;
    let (
        E3PinnedConfigPatchSourceV1::Global {
            opened,
            source_root: accepted_home,
            source_relative_path: global_relative_path,
            source_bytes_sha256: global_bytes_sha256,
        },
        E3PinnedConfigPatchSourceV1::Workspace {
            source_root: workspace_root,
            held_source_root,
            source_relative_path: workspace_relative_path,
            held_file,
            file_device_id,
            file_inode,
            file_byte_length,
            source_bytes,
            source_bytes_sha256: workspace_bytes_sha256,
        },
    ) = (&snapshot.global_patch, &snapshot.workspace_patch)
    else {
        return Err(user_error("E3 config source postures are malformed"));
    };
    if workspace_root != &snapshot.workspace_selection.authenticated_workspace {
        return Err(user_error(
            "selected workspace root does not match the projection workspace binding",
        ));
    }
    opened
        .revalidate()
        .map_err(|error| user_error(error.to_string()))?;
    opened
        .validate_e3_public_root_identity(accepted_home)
        .map_err(|error| user_error(error.to_string()))?;
    if global_relative_path != "config.yaml"
        || global_bytes_sha256.is_some() != opened.source_bytes().is_some()
        || global_bytes_sha256.as_deref()
            != opened
                .source_bytes()
                .map(|bytes| format!("{:x}", Sha256::digest(bytes)))
                .as_deref()
        || workspace_relative_path != ".substrate/workspace.yaml"
        || format!("{:x}", Sha256::digest(source_bytes)) != *workspace_bytes_sha256
    {
        return Err(user_error("E3 retained configuration source mismatch"));
    }
    let root_metadata = held_source_root
        .metadata()
        .context("failed to revalidate held workspace root")?;
    if root_metadata.dev()
        != match workspace_root.physical_identity {
            config_projection::DirectoryPhysicalIdentityV1::Linux { device_id, .. } => device_id,
        }
        || root_metadata.ino()
            != match workspace_root.physical_identity {
                config_projection::DirectoryPhysicalIdentityV1::Linux { inode, .. } => inode,
            }
    {
        return Err(user_error("held workspace root identity changed"));
    }
    let file_metadata = held_file
        .metadata()
        .context("failed to revalidate held workspace configuration")?;
    if file_metadata.dev() != *file_device_id
        || file_metadata.ino() != *file_inode
        || file_metadata.len() != *file_byte_length
        || usize::try_from(*file_byte_length).ok() != Some(source_bytes.len())
    {
        return Err(user_error("held workspace configuration identity changed"));
    }
    let substrate_name = CString::new(".substrate").expect("static component is valid");
    // SAFETY: the retained workspace root and static component are valid.
    let substrate_fd = unsafe {
        libc::openat(
            held_source_root.as_raw_fd(),
            substrate_name.as_ptr(),
            libc::O_RDONLY | libc::O_DIRECTORY | libc::O_CLOEXEC | libc::O_NOFOLLOW,
        )
    };
    if substrate_fd < 0 {
        return Err(anyhow!(
            "failed to re-open retained workspace .substrate directory: {}",
            io::Error::last_os_error()
        ));
    }
    // SAFETY: a successful openat returns a newly owned descriptor.
    let substrate = unsafe { File::from_raw_fd(substrate_fd) };
    let workspace_name = CString::new("workspace.yaml").expect("static component is valid");
    let mut current = std::mem::MaybeUninit::<libc::stat>::uninit();
    // SAFETY: descriptor, component, and output pointer are valid.
    if unsafe {
        libc::fstatat(
            substrate.as_raw_fd(),
            workspace_name.as_ptr(),
            current.as_mut_ptr(),
            libc::AT_SYMLINK_NOFOLLOW,
        )
    } != 0
    {
        return Err(user_error(
            "workspace configuration disappeared before E3 extraction",
        ));
    }
    // SAFETY: fstatat initialized the output on success.
    let current = unsafe { current.assume_init() };
    if current.st_dev != *file_device_id
        || current.st_ino != *file_inode
        || current.st_size < 0
        || current.st_size as u64 != *file_byte_length
    {
        return Err(user_error(
            "workspace configuration named identity changed before E3 extraction",
        ));
    }

    let expected_keys = [
        "llm.enabled",
        "llm.gateway.enabled",
        "llm.gateway.mode",
        "llm.routing.default_backend",
        "agents.enabled",
        "agents.defaults.execution.scope",
        "agents.defaults.cli.mode",
        "world.enabled",
    ];
    let global_display = PathBuf::from(&accepted_home.physical_path).join(global_relative_path);
    let workspace_display =
        PathBuf::from(&workspace_root.physical_path).join(workspace_relative_path);
    let mut ordered_explain_origins = Vec::with_capacity(expected_keys.len());
    for key in expected_keys {
        let explained = snapshot
            .explain
            .keys
            .0
            .get(key)
            .ok_or_else(|| user_error(format!("missing E3 config explanation for {key}")))?;
        if explained.merge_strategy != "replace" || explained.sources.len() != 1 {
            return Err(user_error(format!(
                "malformed E3 config explanation for {key}"
            )));
        }
        let explained_source = &explained.sources[0];
        let (source_kind, source_location) = match explained_source.layer.as_str() {
            "default" if explained_source.path.is_none() => {
                (E3ConfigExplainOriginKindV1::Default, None)
            }
            "override_env" if explained_source.path.is_none() => {
                (E3ConfigExplainOriginKindV1::OverrideEnv, None)
            }
            "global_patch"
                if explained_source.path.as_deref() == global_display.to_str()
                    && global_bytes_sha256.is_some() =>
            {
                (
                    E3ConfigExplainOriginKindV1::GlobalPatch,
                    Some(E3ConfigExplainFileV1 {
                        source_root: accepted_home.clone(),
                        source_relative_path: global_relative_path.clone(),
                        source_bytes_sha256: global_bytes_sha256.clone().expect("checked above"),
                    }),
                )
            }
            "workspace_patch" if explained_source.path.as_deref() == workspace_display.to_str() => {
                (
                    E3ConfigExplainOriginKindV1::WorkspacePatch,
                    Some(E3ConfigExplainFileV1 {
                        source_root: workspace_root.clone(),
                        source_relative_path: workspace_relative_path.clone(),
                        source_bytes_sha256: workspace_bytes_sha256.clone(),
                    }),
                )
            }
            _ => {
                return Err(user_error(format!(
                    "E3 config explanation does not match its retained source for {key}"
                )))
            }
        };
        ordered_explain_origins.push(E3ConfigExplainOriginV1 {
            key: key.to_string(),
            source_kind,
            source_location,
        });
    }

    let values = E3EffectiveConfigInputV1 {
        llm_enabled: snapshot.effective.llm.enabled,
        agents_enabled: snapshot.effective.agents.enabled,
        world_enabled: snapshot.effective.world.enabled,
        default_execution_scope: match snapshot.effective.agents.defaults.execution.scope {
            AgentExecutionScope::Host => "host",
            AgentExecutionScope::World => "world",
        }
        .to_string(),
        default_cli_mode: match snapshot.effective.agents.defaults.cli.mode {
            AgentCliMode::Persistent => "persistent",
            AgentCliMode::PerRequest => "per_request",
        }
        .to_string(),
        managed_gateway_enabled: snapshot.effective.llm.gateway.enabled,
        managed_gateway_mode: match snapshot.effective.llm.gateway.mode {
            LlmGatewayMode::InWorld => "in_world",
            LlmGatewayMode::HostOnly => "host_only",
        }
        .to_string(),
        default_backend_id: snapshot.effective.llm.routing.default_backend.clone(),
    };
    if !values.llm_enabled
        || !values.managed_gateway_enabled
        || !values.agents_enabled
        || !values.world_enabled
        || values.managed_gateway_mode != "in_world"
        || values.default_execution_scope != "world"
        || values.default_cli_mode != "persistent"
        || values.default_backend_id != "cli:codex-world"
    {
        return Err(user_error(
            "effective configuration is not eligible for E3 projection",
        ));
    }
    let mut source = EffectiveSubstrateConfigSourceV1 {
        schema_version: 1,
        authority_store_id: authority_store_id.to_string(),
        accepted_home: accepted_home.clone(),
        workspace_root: workspace_root.clone(),
        values,
        ordered_explain_origins,
        source_revision: String::new(),
        source_hash: String::new(),
    };
    let mut revision_value = serde_json::to_value(&source)?;
    let revision_object = revision_value
        .as_object_mut()
        .ok_or_else(|| user_error("E3 effective config source is not an object"))?;
    revision_object.remove("source_revision");
    revision_object.remove("source_hash");
    let revision_bytes = ConfigProjectionCodecV1::encode_canonical_json(&revision_value)
        .map_err(|_| user_error("failed to canonicalize E3 effective config revision"))?;
    source.source_revision = format!("ecsr1_{:x}", Sha256::digest(revision_bytes));
    let mut source_value = serde_json::to_value(&source)?;
    source_value
        .as_object_mut()
        .ok_or_else(|| user_error("E3 effective config source is not an object"))?
        .remove("source_hash");
    let mut payload = BTreeMap::new();
    payload.insert("source".to_string(), source_value);
    source.source_hash = ConfigProjectionCodecV1::domain_sha256(
        "substrate.e3.effective-substrate-config-source.v1",
        &payload,
    )
    .map_err(|_| user_error("failed to hash E3 effective config source"))?;
    opened
        .revalidate()
        .map_err(|error| user_error(error.to_string()))?;
    Ok(source)
}

#[allow(
    dead_code,
    reason = "used by the A1.1e explicit-home entry point before A1.3 adoption"
)]
fn load_workspace_config_patch(cwd: &Path) -> Result<Option<ConfigPatchLayer>> {
    let Some(root) = workspace::find_workspace_root(cwd) else {
        return Ok(None);
    };
    let legacy = workspace::workspace_legacy_settings_path(&root);
    let path = workspace::workspace_marker_path(&root);
    if legacy.exists() {
        return Err(user_error(format!(
            "substrate: unsupported legacy workspace config detected:\n  - {}\nConfig is now read from:\n  - {}\nNext steps:\n  - Delete the legacy file and use `substrate config workspace set ...`\n",
            legacy.display(),
            path.display()
        )));
    }
    let raw =
        fs::read_to_string(&path).with_context(|| format!("failed to read {}", path.display()))?;
    Ok(Some((
        parse_captured_config_patch(&path, &raw, AgentSelectionSourceScopeV1::Workspace)?,
        path,
    )))
}

fn load_config_patch_layers_cached(cwd: &Path) -> Result<LoadedConfigPatchLayers> {
    let global_path = global_config_path()?;
    let workspace_root = workspace::find_workspace_root(cwd);
    let workspace_path = workspace_root
        .as_ref()
        .map(|root| workspace::workspace_marker_path(root));

    if let Some(root) = &workspace_root {
        let legacy = workspace::workspace_legacy_settings_path(root);
        if legacy.exists() {
            return Err(user_error(format!(
                "substrate: unsupported legacy workspace config detected:\n  - {}\nConfig is now read from:\n  - {}\nNext steps:\n  - Delete the legacy file and use `substrate config workspace set ...`\n",
                legacy.display(),
                workspace::workspace_marker_path(root).display()
            )));
        }
    }

    let global_stat = FileStatKey::for_path(&global_path)?;
    let workspace_stat = workspace_path
        .as_ref()
        .map(|path| FileStatKey::for_path(path))
        .transpose()?;

    let cache = CONFIG_PATCH_CACHE.get_or_init(|| Mutex::new(None));
    if let Ok(guard) = cache.lock() {
        if let Some(entry) = guard.as_ref() {
            if entry.workspace_root == workspace_root
                && entry.global_path == global_path
                && entry.workspace_path == workspace_path
                && entry.global_stat == global_stat
                && entry.workspace_stat == workspace_stat
            {
                // Metadata alone can alias a different revision or physical root.
                // Reuse parsed values only while their captured sources still match.
                let global_current = captured_config_patch_matches_path(
                    &entry.global_patch,
                    &global_path,
                    AgentSelectionSourceScopeV1::Global,
                )?;
                let workspace_current = match (&entry.workspace_patch, &entry.workspace_path) {
                    (Some(patch), Some(path)) => captured_config_patch_matches_path(
                        patch,
                        path,
                        AgentSelectionSourceScopeV1::Workspace,
                    )?,
                    (None, None) => true,
                    _ => false,
                };
                if global_current && workspace_current {
                    let workspace_layer = entry
                        .workspace_patch
                        .clone()
                        .zip(entry.workspace_path.clone());
                    return Ok((entry.global_patch.clone(), global_path, workspace_layer));
                }
            }
        }
    }

    let global_patch = match fs::read_to_string(&global_path) {
        Ok(raw) => {
            parse_captured_config_patch(&global_path, &raw, AgentSelectionSourceScopeV1::Global)?
        }
        Err(err) if err.kind() == io::ErrorKind::NotFound => captured_absent_config_patch(),
        Err(err) => return Err(anyhow!("failed to read {}: {err}", global_path.display())),
    };

    let workspace_layer = if let Some(workspace_path) = &workspace_path {
        let raw = fs::read_to_string(workspace_path)
            .with_context(|| format!("failed to read {}", workspace_path.display()))?;
        let patch = parse_captured_config_patch(
            workspace_path,
            &raw,
            AgentSelectionSourceScopeV1::Workspace,
        )?;
        Some((patch, workspace_path.clone()))
    } else {
        None
    };

    if let Ok(mut guard) = cache.lock() {
        *guard = Some(ConfigPatchCacheEntry {
            workspace_root,
            global_path: global_path.clone(),
            workspace_path,
            global_stat,
            workspace_stat,
            global_patch: global_patch.clone(),
            workspace_patch: workspace_layer.as_ref().map(|(p, _)| p.clone()),
        });
    }

    Ok((global_patch, global_path, workspace_layer))
}

type ConfigPatchLayer = (SubstrateConfigPatch, PathBuf);
type LoadedConfigPatchLayers = (SubstrateConfigPatch, PathBuf, Option<ConfigPatchLayer>);

fn resolve_effective_from_layers(
    global_patch: &SubstrateConfigPatch,
    global_path: &Path,
    workspace_patch: Option<(&SubstrateConfigPatch, &Path)>,
    env_overrides: &EnvOverrides,
    cli_overrides: &CliConfigOverrides,
    explain: bool,
    inject_protected_excludes: bool,
) -> Result<(SubstrateConfig, Option<ConfigExplainV1>)> {
    let mut effective = SubstrateConfig::default();
    let mut explain_keys = if explain {
        Some(OrderedExplainKeys::default())
    } else {
        None
    };

    let workspace_enabled = workspace_patch.is_some();
    let workspace_path = workspace_patch.map(|(_, p)| p);

    // Helper for replace keys.
    #[derive(Clone, Copy)]
    enum ReplaceSource {
        CliFlag,
        OverrideEnv,
        WorkspacePatch,
        GlobalPatch,
        Default,
    }

    fn explain_source(
        source: ReplaceSource,
        global_path: &Path,
        workspace_path: Option<&Path>,
    ) -> ConfigExplainSource {
        match source {
            ReplaceSource::CliFlag => ConfigExplainSource {
                layer: "cli_flag".to_string(),
                path: None,
            },
            ReplaceSource::OverrideEnv => ConfigExplainSource {
                layer: "override_env".to_string(),
                path: None,
            },
            ReplaceSource::WorkspacePatch => ConfigExplainSource {
                layer: "workspace_patch".to_string(),
                path: workspace_path.map(|p| p.display().to_string()),
            },
            ReplaceSource::GlobalPatch => ConfigExplainSource {
                layer: "global_patch".to_string(),
                path: Some(global_path.display().to_string()),
            },
            ReplaceSource::Default => ConfigExplainSource {
                layer: "default".to_string(),
                path: None,
            },
        }
    }

    fn resolve_replace<T: Clone>(
        default: T,
        global: Option<T>,
        workspace: Option<T>,
        env: Option<T>,
        cli: Option<T>,
        workspace_enabled: bool,
    ) -> (T, ReplaceSource) {
        if let Some(v) = cli {
            return (v, ReplaceSource::CliFlag);
        }
        if workspace_enabled {
            if let Some(v) = workspace {
                return (v, ReplaceSource::WorkspacePatch);
            }
        } else if let Some(v) = env {
            // Preserve historical behavior: env overrides apply only when no workspace exists.
            return (v, ReplaceSource::OverrideEnv);
        }
        if let Some(v) = global {
            return (v, ReplaceSource::GlobalPatch);
        }
        (default, ReplaceSource::Default)
    }

    // world.enabled
    let (world_enabled, world_enabled_src) = resolve_replace(
        effective.world.enabled,
        global_patch.world.enabled,
        workspace_patch
            .map(|(p, _)| p.world.enabled)
            .unwrap_or(None),
        env_overrides.world_enabled,
        cli_overrides.world_enabled,
        workspace_enabled,
    );
    effective.world.enabled = world_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    world_enabled_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // world.anchor_mode
    let (anchor_mode, anchor_mode_src) = resolve_replace(
        effective.world.anchor_mode,
        global_patch.world.anchor_mode,
        workspace_patch
            .map(|(p, _)| p.world.anchor_mode)
            .unwrap_or(None),
        env_overrides.anchor_mode,
        cli_overrides.anchor_mode,
        workspace_enabled,
    );
    effective.world.anchor_mode = anchor_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.anchor_mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(anchor_mode_src, global_path, workspace_path)],
            },
        );
    }

    // world.anchor_path
    let (anchor_path, anchor_path_src) = resolve_replace(
        effective.world.anchor_path.clone(),
        global_patch.world.anchor_path.clone(),
        workspace_patch
            .map(|(p, _)| p.world.anchor_path.clone())
            .unwrap_or(None),
        env_overrides.anchor_path.clone(),
        cli_overrides.anchor_path.clone(),
        workspace_enabled,
    );
    effective.world.anchor_path = anchor_path;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.anchor_path".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(anchor_path_src, global_path, workspace_path)],
            },
        );
    }

    // world.caged
    let (caged, caged_src) = resolve_replace(
        effective.world.caged,
        global_patch.world.caged,
        workspace_patch.map(|(p, _)| p.world.caged).unwrap_or(None),
        env_overrides.caged,
        cli_overrides.caged,
        workspace_enabled,
    );
    effective.world.caged = caged;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.caged".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(caged_src, global_path, workspace_path)],
            },
        );
    }

    // world.net.filter
    let (net_filter, net_filter_src) = resolve_replace(
        effective.world.net.filter,
        global_patch.world.net.filter,
        workspace_patch
            .map(|(p, _)| p.world.net.filter)
            .unwrap_or(None),
        env_overrides.world_net_filter,
        None,
        workspace_enabled,
    );
    effective.world.net.filter = net_filter;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.net.filter".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(net_filter_src, global_path, workspace_path)],
            },
        );
    }

    // world.env.inherit_from_host
    let (inherit_from_host, inherit_from_host_src) = resolve_replace(
        effective.world.env.inherit_from_host,
        global_patch.world.env.inherit_from_host,
        workspace_patch
            .map(|(p, _)| p.world.env.inherit_from_host)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.env.inherit_from_host = inherit_from_host;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.env.inherit_from_host".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    inherit_from_host_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // repl.exit_cwd
    let (exit_cwd, exit_cwd_src) = resolve_replace(
        effective.repl.exit_cwd,
        global_patch.repl.exit_cwd,
        workspace_patch
            .map(|(p, _)| p.repl.exit_cwd)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.repl.exit_cwd = exit_cwd;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "repl.exit_cwd".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(exit_cwd_src, global_path, workspace_path)],
            },
        );
    }

    // repl.max_pty_buffered_lines
    let (raw_max_pty_buffered_lines, max_pty_buffered_lines_src) = resolve_replace(
        effective.repl.max_pty_buffered_lines,
        global_patch.repl.max_pty_buffered_lines,
        workspace_patch
            .map(|(p, _)| p.repl.max_pty_buffered_lines)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    let max_pty_buffered_lines = raw_max_pty_buffered_lines.clamp(
        REPL_MAX_PTY_BUFFERED_LINES_MIN,
        REPL_MAX_PTY_BUFFERED_LINES_MAX,
    );
    effective.repl.max_pty_buffered_lines = max_pty_buffered_lines;
    if raw_max_pty_buffered_lines != max_pty_buffered_lines {
        effective.repl.max_pty_buffered_lines_clamp = Some(I64ClampInfo {
            provided: raw_max_pty_buffered_lines,
            effective: max_pty_buffered_lines,
            min: REPL_MAX_PTY_BUFFERED_LINES_MIN,
            max: REPL_MAX_PTY_BUFFERED_LINES_MAX,
        });
    } else {
        effective.repl.max_pty_buffered_lines_clamp = None;
    }
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "repl.max_pty_buffered_lines".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    max_pty_buffered_lines_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // llm.enabled
    let (llm_enabled, llm_enabled_src) = resolve_replace(
        effective.llm.enabled,
        global_patch.llm.enabled,
        workspace_patch.map(|(p, _)| p.llm.enabled).unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.llm.enabled = llm_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "llm.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(llm_enabled_src, global_path, workspace_path)],
            },
        );
    }

    // llm.gateway.enabled
    let (llm_gateway_enabled, llm_gateway_enabled_src) = resolve_replace(
        effective.llm.gateway.enabled,
        global_patch.llm.gateway.enabled,
        workspace_patch
            .map(|(p, _)| p.llm.gateway.enabled)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.llm.gateway.enabled = llm_gateway_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "llm.gateway.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    llm_gateway_enabled_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // llm.gateway.mode
    let (llm_gateway_mode, llm_gateway_mode_src) = resolve_replace(
        effective.llm.gateway.mode,
        global_patch.llm.gateway.mode,
        workspace_patch
            .map(|(p, _)| p.llm.gateway.mode)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.llm.gateway.mode = llm_gateway_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "llm.gateway.mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    llm_gateway_mode_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // llm.routing.default_backend
    let (llm_default_backend, llm_default_backend_src) = resolve_replace(
        effective.llm.routing.default_backend.clone(),
        global_patch.llm.routing.default_backend.clone(),
        workspace_patch
            .map(|(p, _)| p.llm.routing.default_backend.clone())
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.llm.routing.default_backend = llm_default_backend;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "llm.routing.default_backend".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    llm_default_backend_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.enabled
    let (agents_enabled, agents_enabled_src) = resolve_replace(
        effective.agents.enabled,
        global_patch.agents.enabled,
        workspace_patch
            .map(|(p, _)| p.agents.enabled)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.enabled = agents_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    agents_enabled_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.defaults.execution.scope
    let (agent_execution_scope, agent_execution_scope_src) = resolve_replace(
        effective.agents.defaults.execution.scope,
        global_patch.agents.defaults.execution.scope,
        workspace_patch
            .map(|(p, _)| p.agents.defaults.execution.scope)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.defaults.execution.scope = agent_execution_scope;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.defaults.execution.scope".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    agent_execution_scope_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.defaults.cli.mode
    let (agent_cli_mode, agent_cli_mode_src) = resolve_replace(
        effective.agents.defaults.cli.mode,
        global_patch.agents.defaults.cli.mode,
        workspace_patch
            .map(|(p, _)| p.agents.defaults.cli.mode)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.defaults.cli.mode = agent_cli_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.defaults.cli.mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    agent_cli_mode_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.hub.orchestrator_agent_id
    let (orchestrator_agent_id, orchestrator_agent_id_src) = resolve_replace(
        effective.agents.hub.orchestrator_agent_id.clone(),
        global_patch.agents.hub.orchestrator_agent_id.clone(),
        workspace_patch
            .map(|(p, _)| p.agents.hub.orchestrator_agent_id.clone())
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.hub.orchestrator_agent_id = orchestrator_agent_id;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.hub.orchestrator_agent_id".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    orchestrator_agent_id_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.hub.world_restart.on_drift
    let (world_restart_on_drift, world_restart_on_drift_src) = resolve_replace(
        effective.agents.hub.world_restart.on_drift,
        global_patch.agents.hub.world_restart.on_drift,
        workspace_patch
            .map(|(p, _)| p.agents.hub.world_restart.on_drift)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.hub.world_restart.on_drift = world_restart_on_drift;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.hub.world_restart.on_drift".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    world_restart_on_drift_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.toolbox.enabled
    let (toolbox_enabled, toolbox_enabled_src) = resolve_replace(
        effective.agents.toolbox.enabled,
        global_patch.agents.toolbox.enabled,
        workspace_patch
            .map(|(p, _)| p.agents.toolbox.enabled)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.toolbox.enabled = toolbox_enabled;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.toolbox.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    toolbox_enabled_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // agents.toolbox.bind.transport
    let (toolbox_transport, toolbox_transport_src) = resolve_replace(
        effective.agents.toolbox.bind.transport,
        global_patch.agents.toolbox.bind.transport,
        workspace_patch
            .map(|(p, _)| p.agents.toolbox.bind.transport)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.agents.toolbox.bind.transport = toolbox_transport;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "agents.toolbox.bind.transport".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    toolbox_transport_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // policy.mode (from config.yaml/workspace.yaml, not policy.yaml)
    let (policy_mode, policy_mode_src) = resolve_replace(
        effective.policy.mode,
        global_patch.policy.mode,
        workspace_patch.map(|(p, _)| p.policy.mode).unwrap_or(None),
        env_overrides.policy_mode,
        None,
        workspace_enabled,
    );
    effective.policy.mode = policy_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "policy.mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(policy_mode_src, global_path, workspace_path)],
            },
        );
    }

    // sync.auto_sync
    let (auto_sync, auto_sync_src) = resolve_replace(
        effective.sync.auto_sync,
        global_patch.sync.auto_sync,
        workspace_patch
            .map(|(p, _)| p.sync.auto_sync)
            .unwrap_or(None),
        env_overrides.sync_auto_sync,
        None,
        workspace_enabled,
    );
    effective.sync.auto_sync = auto_sync;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.auto_sync".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(auto_sync_src, global_path, workspace_path)],
            },
        );
    }

    // sync.direction
    let (sync_direction, sync_direction_src) = resolve_replace(
        effective.sync.direction,
        global_patch.sync.direction,
        workspace_patch
            .map(|(p, _)| p.sync.direction)
            .unwrap_or(None),
        env_overrides.sync_direction,
        None,
        workspace_enabled,
    );
    effective.sync.direction = sync_direction;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.direction".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    sync_direction_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // sync.conflict_policy
    let (conflict_policy, conflict_policy_src) = resolve_replace(
        effective.sync.conflict_policy,
        global_patch.sync.conflict_policy,
        workspace_patch
            .map(|(p, _)| p.sync.conflict_policy)
            .unwrap_or(None),
        env_overrides.sync_conflict_policy,
        None,
        workspace_enabled,
    );
    effective.sync.conflict_policy = conflict_policy;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.conflict_policy".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(
                    conflict_policy_src,
                    global_path,
                    workspace_path,
                )],
            },
        );
    }

    // sync.exclude (replace semantics; protected excludes are injected later)
    let (exclude, exclude_src) = resolve_replace(
        effective.sync.exclude.clone(),
        global_patch.sync.exclude.clone(),
        workspace_patch
            .map(|(p, _)| p.sync.exclude.clone())
            .unwrap_or(None),
        env_overrides.sync_exclude.clone(),
        None,
        workspace_enabled,
    );
    effective.sync.exclude = exclude;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "sync.exclude".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(exclude_src, global_path, workspace_path)],
            },
        );
    }

    // world.deps.enabled (concat_dedupe_ordered_set)
    let default_list = effective.world.deps.enabled.clone();
    let mut enabled_sources = Vec::new();
    let mut layers = Vec::new();
    let has_any_patch = global_patch.world.deps.enabled.is_some()
        || (workspace_enabled
            && workspace_patch
                .and_then(|(p, _)| p.world.deps.enabled.as_ref())
                .is_some());
    if !default_list.is_empty() || !has_any_patch {
        enabled_sources.push(ConfigExplainSource {
            layer: "default".to_string(),
            path: None,
        });
        layers.push(default_list);
    }
    if let Some(list) = &global_patch.world.deps.enabled {
        enabled_sources.push(ConfigExplainSource {
            layer: "global_patch".to_string(),
            path: Some(global_path.display().to_string()),
        });
        layers.push(list.clone());
    }
    if workspace_enabled {
        if let Some(list) = workspace_patch.and_then(|(p, _)| p.world.deps.enabled.as_ref()) {
            enabled_sources.push(ConfigExplainSource {
                layer: "workspace_patch".to_string(),
                path: workspace_path.map(|p| p.display().to_string()),
            });
            layers.push(list.clone());
        }
    }
    effective.world.deps.enabled = concat_dedupe_ordered_set(&layers);
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.enabled".to_string(),
            ConfigExplainKey {
                merge_strategy: "concat_dedupe_ordered_set".to_string(),
                sources: enabled_sources,
            },
        );
    }

    // world.deps.inventory_mode (replace)
    let (inv_mode, inv_mode_src) = resolve_replace(
        effective.world.deps.inventory_mode,
        global_patch.world.deps.inventory_mode,
        workspace_patch
            .map(|(p, _)| p.world.deps.inventory_mode)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.deps.inventory_mode = inv_mode;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.inventory_mode".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(inv_mode_src, global_path, workspace_path)],
            },
        );
    }

    // world.deps.builtins (replace)
    let (builtins, builtins_src) = resolve_replace(
        effective.world.deps.builtins,
        global_patch.world.deps.builtins,
        workspace_patch
            .map(|(p, _)| p.world.deps.builtins)
            .unwrap_or(None),
        None,
        None,
        workspace_enabled,
    );
    effective.world.deps.builtins = builtins;
    if let Some(keys) = &mut explain_keys {
        keys.insert(
            "world.deps.builtins".to_string(),
            ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![explain_source(builtins_src, global_path, workspace_path)],
            },
        );
    }

    if inject_protected_excludes {
        apply_protected_excludes(&mut effective.sync.exclude);
    }

    bind_agent_selection_config(&mut effective, global_patch, global_path, workspace_patch)?;
    if let Some(keys) = &mut explain_keys {
        // Explanation describes semantic precedence even for caller-held bytes,
        // which deliberately carry no physical runtime selection provenance.
        let preference_scope = if workspace_patch
            .is_some_and(|(patch, _)| patch.agents.defaults.selection.is_some())
        {
            AgentSelectionSourceScopeV1::Workspace
        } else if global_patch.agents.defaults.selection.is_some() {
            AgentSelectionSourceScopeV1::Global
        } else {
            AgentSelectionSourceScopeV1::BuiltIn
        };
        keys.insert(
            "agents.defaults.selection".into(),
            ConfigExplainKey {
                merge_strategy: "replace".into(),
                sources: vec![ConfigExplainSource {
                    layer: match preference_scope {
                        AgentSelectionSourceScopeV1::BuiltIn => "default",
                        AgentSelectionSourceScopeV1::Global => "global_patch",
                        AgentSelectionSourceScopeV1::Workspace => "workspace_patch",
                    }
                    .into(),
                    path: match preference_scope {
                        AgentSelectionSourceScopeV1::BuiltIn => None,
                        AgentSelectionSourceScopeV1::Global => {
                            Some(global_path.display().to_string())
                        }
                        AgentSelectionSourceScopeV1::Workspace => {
                            workspace_path.map(|path| path.display().to_string())
                        }
                    },
                }],
            },
        );
    }
    validate_local_config(&effective)?;

    let explain = explain_keys.map(|keys| ConfigExplainV1 {
        kind: "substrate.config.explain.v1".to_string(),
        keys,
    });
    Ok((effective, explain))
}

/// Capture a source revision from bytes already read by the source owner.
pub(crate) fn selection_source_from_bytes(
    root: &Path,
    relative_path: &str,
    scope: AgentSelectionSourceScopeV1,
    bytes: &[u8],
) -> Result<AgentSelectionSourceV1> {
    let canonical = fs::canonicalize(root).with_context(|| {
        format!(
            "failed to identify selection source root {}",
            root.display()
        )
    })?;
    let metadata = fs::metadata(&canonical)?;
    #[cfg(unix)]
    let identity = {
        use std::os::unix::fs::MetadataExt;
        format!(
            "{}:{}:{}",
            metadata.dev(),
            metadata.ino(),
            canonical.display()
        )
    };
    #[cfg(not(unix))]
    let identity = format!("{}:{}", canonical.display(), metadata.len());
    let hash = format!("{:x}", Sha256::digest(bytes));
    let source = AgentSelectionSourceV1 {
        scope,
        root_identity: identity,
        relative_path: relative_path.into(),
        revision: hash.clone(),
        sha256: hash,
    };
    source.validate().map_err(user_error)?;
    Ok(source)
}

/// Parse bytes at an ordinary source-owner boundary and capture their exact revision.
/// The pure parser/composer never calls this filesystem-reading function.
fn parse_captured_config_patch(
    path: &Path,
    raw: &str,
    scope: AgentSelectionSourceScopeV1,
) -> Result<SubstrateConfigPatch> {
    let mut patch = parse_config_patch_yaml(path, raw)?;
    let root = path
        .parent()
        .ok_or_else(|| user_error("selection config path has no root"))?;
    let relative = path
        .file_name()
        .and_then(|name| name.to_str())
        .ok_or_else(|| user_error("selection config source requires UTF-8"))?;
    patch.captured_selection_source = Some(CapturedSelectionConfigSourceV1::Present(
        selection_source_from_bytes(root, relative, scope, raw.as_bytes())?,
    ));
    Ok(patch)
}

// Cache validation stays with the ordinary source owner, outside composition.
fn captured_config_patch_matches_path(
    patch: &SubstrateConfigPatch,
    path: &Path,
    scope: AgentSelectionSourceScopeV1,
) -> Result<bool> {
    match fs::read(path) {
        Ok(bytes) => {
            let root = path
                .parent()
                .ok_or_else(|| user_error("selection config path has no root"))?;
            let relative = path
                .file_name()
                .and_then(|name| name.to_str())
                .ok_or_else(|| user_error("selection config source requires UTF-8"))?;
            let source = selection_source_from_bytes(root, relative, scope, &bytes)?;
            Ok(patch.captured_selection_source.as_ref()
                == Some(&CapturedSelectionConfigSourceV1::Present(source)))
        }
        Err(error) if error.kind() == io::ErrorKind::NotFound => {
            Ok(patch.captured_selection_source.as_ref()
                == Some(&CapturedSelectionConfigSourceV1::Absent))
        }
        Err(error) => Err(error).with_context(|| format!("failed to read {}", path.display())),
    }
}

fn captured_absent_config_patch() -> SubstrateConfigPatch {
    SubstrateConfigPatch {
        captured_selection_source: Some(CapturedSelectionConfigSourceV1::Absent),
        ..Default::default()
    }
}

fn bind_agent_selection_config(
    effective: &mut SubstrateConfig,
    global: &SubstrateConfigPatch,
    global_path: &Path,
    workspace: Option<(&SubstrateConfigPatch, &Path)>,
) -> Result<()> {
    let built_in = AgentSelectionSourceV1 {
        scope: AgentSelectionSourceScopeV1::BuiltIn,
        root_identity: "substrate-agent-routes.v1".into(),
        relative_path: "config/agent-routes".into(),
        revision: "1".into(),
        sha256: format!(
            "{:x}",
            Sha256::digest(b"substrate.codex-subscription.claude-existing.routes.v1")
        ),
    };
    let all_sources_captured = global.captured_selection_source.is_some()
        && workspace.is_none_or(|(patch, _)| patch.captured_selection_source.is_some());
    // Source-owner metadata enables runtime selection. Caller-held bytes may
    // compose values, but cannot publish a physically captured selection.
    effective.selection_sources.clear();
    effective.selection_preference_source = None;
    effective.selection_profiles.clear();
    if all_sources_captured {
        effective.selection_sources = vec![built_in.clone()];
        effective.selection_preference_source = Some(built_in.clone());
        effective.selection_profiles = vec![
            AgentConnectionProfileV1 {
                id: "built_in:codex-subscription".into(),
                revision: "1".into(),
                provider: "openai".into(),
                billing: AgentConnectionBillingV1::Subscription,
                endpoint_identity: "openai-codex-subscription".into(),
                upstream_base_url: "https://chatgpt.com/backend-api".into(),
                auth: AgentConnectionAuthV1::CodexAccount {
                    reference: "configured-account".into(),
                },
                source: built_in.clone(),
            },
            AgentConnectionProfileV1 {
                id: "built_in:claude-existing".into(),
                revision: "1".into(),
                provider: "anthropic".into(),
                billing: AgentConnectionBillingV1::Api,
                endpoint_identity: "anthropic-api".into(),
                upstream_base_url: "https://api.anthropic.com".into(),
                auth: AgentConnectionAuthV1::ApiKeyEnvironment {
                    name: "ANTHROPIC_API_KEY".into(),
                },
                source: built_in,
            },
        ];
    }
    effective.agents.connection_profiles.clear();
    for (patch, path, scope) in
        std::iter::once((global, global_path, AgentSelectionSourceScopeV1::Global)).chain(
            workspace.map(|(patch, path)| (patch, path, AgentSelectionSourceScopeV1::Workspace)),
        )
    {
        if let Some(selection) = &patch.agents.defaults.selection {
            effective.agents.defaults.selection = selection.clone();
        }
        for (id, input) in &patch.agents.connection_profiles {
            if !id.starts_with(&format!("{}:", scope.as_str())) {
                return Err(user_error(format!(
                    "connection profile '{}' in {} cannot redefine another source namespace",
                    id,
                    path.display()
                )));
            }
            // Value validity does not depend on physical source capture.
            AgentConnectionProfileV1::validate_fields(
                id,
                &input.revision,
                &input.provider,
                input.billing,
                &input.endpoint_identity,
                &input.upstream_base_url,
                &input.auth,
            )
            .map_err(|error| {
                user_error(format!(
                    "invalid connection profile '{}' in {}: {}",
                    id,
                    path.display(),
                    error,
                ))
            })?;
            effective
                .agents
                .connection_profiles
                .insert(id.clone(), input.clone());
        }
        if !all_sources_captured {
            continue;
        }
        match patch.captured_selection_source.as_ref() {
            Some(CapturedSelectionConfigSourceV1::Absent) => {
                if patch.source_sha256.is_some() || !patch.is_empty() {
                    return Err(user_error(
                        "absent selection config source has parsed values",
                    ));
                }
            }
            Some(CapturedSelectionConfigSourceV1::Present(source)) => {
                source.validate().map_err(user_error)?;
                if source.scope != scope || patch.source_sha256.as_ref() != Some(&source.sha256) {
                    return Err(user_error(format!(
                        "selection config source mismatch at {}",
                        path.display()
                    )));
                }
                effective.selection_sources.push(source.clone());
                if patch.agents.defaults.selection.is_some() {
                    effective.selection_preference_source = Some(source.clone());
                }
                for (id, input) in &patch.agents.connection_profiles {
                    let profile = input.bind(id, source);
                    profile.validate().map_err(user_error)?;
                    effective.selection_profiles.push(profile);
                }
            }
            None => return Err(user_error("selection source capture is incomplete")),
        }
    }
    Ok(())
}

fn concat_dedupe_ordered_set(layers: &[Vec<String>]) -> Vec<String> {
    let mut out = Vec::new();
    for layer in layers {
        for item in layer {
            if !out.iter().any(|existing| existing == item) {
                out.push(item.clone());
            }
        }
    }
    out
}

#[derive(Debug, Default)]
struct EnvOverrides {
    world_enabled: Option<bool>,
    anchor_mode: Option<WorldRootMode>,
    anchor_path: Option<String>,
    caged: Option<bool>,
    world_net_filter: Option<bool>,
    policy_mode: Option<PolicyMode>,
    sync_auto_sync: Option<bool>,
    sync_direction: Option<SyncDirection>,
    sync_conflict_policy: Option<SyncConflictPolicy>,
    sync_exclude: Option<Vec<String>>,
}

fn parse_env_overrides() -> Result<EnvOverrides> {
    let mut overrides = EnvOverrides::default();

    if let Ok(world) = env::var("SUBSTRATE_OVERRIDE_WORLD") {
        let normalized = world.trim().to_ascii_lowercase();
        if !normalized.is_empty() {
            overrides.world_enabled = Some(match normalized.as_str() {
                "enabled" => true,
                "disabled" => false,
                _ => {
                    return Err(user_error(format!(
                        "SUBSTRATE_OVERRIDE_WORLD must be 'enabled' or 'disabled' (found '{}')",
                        world
                    )));
                }
            });
        }
    }

    if let Ok(mode) = env::var("SUBSTRATE_OVERRIDE_ANCHOR_MODE") {
        let trimmed = mode.trim();
        if !trimmed.is_empty() {
            overrides.anchor_mode = Some(WorldRootMode::parse(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_ANCHOR_MODE must be one of workspace, follow-cwd, or custom (found '{}')",
                    mode
                ))
            })?);
        }
    }

    if let Ok(path) = env::var("SUBSTRATE_OVERRIDE_ANCHOR_PATH") {
        overrides.anchor_path = Some(path);
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_CAGED") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.caged = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_CAGED must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_WORLD_NET_FILTER") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.world_net_filter = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_WORLD_NET_FILTER must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_POLICY_MODE") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.policy_mode = Some(PolicyMode::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_POLICY_MODE must be one of disabled, observe, or enforce (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_auto_sync = Some(parse_bool_flag(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_SYNC_AUTO_SYNC must be a boolean (true|false|1|0|yes|no|on|off) (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_DIRECTION") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_direction = Some(SyncDirection::parse_insensitive(trimmed).ok_or_else(|| {
                user_error(format!(
                    "SUBSTRATE_OVERRIDE_SYNC_DIRECTION must be one of from_world, from_host, or both (found '{}')",
                    raw
                ))
            })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY") {
        let trimmed = raw.trim();
        if !trimmed.is_empty() {
            overrides.sync_conflict_policy =
                Some(SyncConflictPolicy::parse_insensitive(trimmed).ok_or_else(|| {
                    user_error(format!(
                        "SUBSTRATE_OVERRIDE_SYNC_CONFLICT_POLICY must be one of prefer_host, prefer_world, or abort (found '{}')",
                        raw
                    ))
                })?);
        }
    }

    if let Ok(raw) = env::var("SUBSTRATE_OVERRIDE_SYNC_EXCLUDE") {
        let items = raw
            .split(',')
            .map(|item| item.trim())
            .filter(|item| !item.is_empty())
            .map(|item| item.to_string())
            .collect::<Vec<_>>();
        overrides.sync_exclude = Some(items);
    }

    Ok(overrides)
}

pub(crate) fn apply_protected_excludes(excludes: &mut Vec<String>) {
    excludes.retain(|item| !PROTECTED_EXCLUDES.contains(&item.as_str()));
    for (idx, item) in PROTECTED_EXCLUDES.iter().enumerate() {
        excludes.insert(idx, item.to_string());
    }
}

pub(crate) fn parse_updates(raw_updates: &[String]) -> Result<Vec<ConfigUpdate>> {
    raw_updates.iter().map(|raw| parse_update(raw)).collect()
}

fn parse_update(raw: &str) -> Result<ConfigUpdate> {
    let (key, op, value) = if let Some((k, v)) = raw.split_once("+=") {
        (k, UpdateOp::Append, v)
    } else if let Some((k, v)) = raw.split_once("-=") {
        (k, UpdateOp::Remove, v)
    } else if let Some((k, v)) = raw.split_once('=') {
        (k, UpdateOp::Set, v)
    } else {
        return Err(user_error(format!(
            "invalid update '{}'; expected key=value, key+=value, or key-=value",
            raw
        )));
    };

    let key = key.trim();
    if key.is_empty() {
        return Err(user_error(format!("invalid update '{}'; missing key", raw)));
    }

    Ok(ConfigUpdate {
        key: key.to_string(),
        op,
        value: value.to_string(),
    })
}

pub(crate) fn apply_updates_to_patch(
    patch: &mut SubstrateConfigPatch,
    updates: &[ConfigUpdate],
) -> Result<bool> {
    let mut changed = false;
    for update in updates {
        let updated = apply_update_to_patch(patch, update)?;
        if updated {
            patch.captured_selection_source = None;
            patch.source_sha256 = None;
        }
        changed |= updated;
    }
    Ok(changed)
}

pub(crate) fn reset_patch_keys(patch: &mut SubstrateConfigPatch, keys: &[String]) -> Result<bool> {
    let mut changed = false;
    for key in keys {
        let updated = reset_patch_key(patch, key)?;
        if updated {
            patch.captured_selection_source = None;
            patch.source_sha256 = None;
        }
        changed |= updated;
    }
    Ok(changed)
}

fn validate_local_config(cfg: &SubstrateConfig) -> Result<()> {
    cfg.agents
        .defaults
        .selection
        .validate()
        .map_err(user_error)?;
    if cfg.world.anchor_mode == WorldRootMode::Custom && cfg.world.anchor_path.trim().is_empty() {
        return Err(user_error(
            "anchor_mode=custom requires world.anchor_path to be non-empty",
        ));
    }
    validate_backend_id_or_empty(
        &cfg.llm.routing.default_backend,
        "llm.routing.default_backend",
    )?;
    Ok(())
}

fn validate_config_patch(patch: &SubstrateConfigPatch) -> Result<()> {
    if let Some(selection) = &patch.agents.defaults.selection {
        selection.validate().map_err(user_error)?;
    }
    if let Some(default_backend) = patch.llm.routing.default_backend.as_deref() {
        validate_backend_id_or_empty(default_backend, "llm.routing.default_backend")?;
    }
    Ok(())
}

fn validate_backend_id_or_empty(value: &str, key: &str) -> Result<()> {
    if value.trim().is_empty() {
        return Ok(());
    }
    validate_backend_id(value).map_err(|_| {
        user_error(format!(
            "invalid {} '{}'; expected <kind>:<name> with kind [a-z0-9_]+ and name [a-z0-9_-]+",
            key,
            value.trim()
        ))
    })
}

fn validate_config_against_effective_policy(cwd: &Path, cfg: &SubstrateConfig) -> Result<()> {
    if cfg.llm.gateway.mode != LlmGatewayMode::HostOnly {
        return Ok(());
    }
    let (policy, _) = substrate_broker::resolve_effective_policy_with_explain(cwd, false)
        .map_err(|err| user_error(err.to_string()))?;
    validate_config_against_policy(cfg, &policy)
}

fn validate_config_against_policy(
    cfg: &SubstrateConfig,
    policy: &substrate_broker::Policy,
) -> Result<()> {
    if cfg.llm.gateway.mode != LlmGatewayMode::HostOnly {
        return Ok(());
    }
    if policy.llm_fail_closed_routing {
        return Err(user_error(
            "llm.gateway.mode=host_only requires effective policy llm.fail_closed.routing=false",
        ));
    }
    Ok(())
}

fn apply_update_to_patch(patch: &mut SubstrateConfigPatch, update: &ConfigUpdate) -> Result<bool> {
    match update.key.as_str() {
        "world.enabled" => apply_bool_opt(&mut patch.world.enabled, &update.op, &update.value),
        "world.anchor_mode" => {
            apply_enum_anchor_mode_opt(&mut patch.world.anchor_mode, &update.op, &update.value)
        }
        "world.anchor_path" => {
            apply_string_opt(&mut patch.world.anchor_path, &update.op, &update.value)
        }
        "world.caged" => apply_bool_opt(&mut patch.world.caged, &update.op, &update.value),
        "world.net.filter" => {
            apply_bool_opt(&mut patch.world.net.filter, &update.op, &update.value)
        }
        "world.env.inherit_from_host" => apply_bool_opt(
            &mut patch.world.env.inherit_from_host,
            &update.op,
            &update.value,
        ),

        "world.deps.enabled" => apply_string_list_opt(&mut patch.world.deps.enabled, update),
        "world.deps.inventory_mode" => apply_enum_inventory_mode_opt(
            &mut patch.world.deps.inventory_mode,
            &update.op,
            &update.value,
        ),
        "world.deps.builtins" => {
            apply_enum_builtins_opt(&mut patch.world.deps.builtins, &update.op, &update.value)
        }

        "policy.mode" => {
            apply_enum_policy_mode_opt(&mut patch.policy.mode, &update.op, &update.value)
        }

        "sync.auto_sync" => apply_bool_opt(&mut patch.sync.auto_sync, &update.op, &update.value),
        "sync.direction" => {
            apply_enum_sync_direction_opt(&mut patch.sync.direction, &update.op, &update.value)
        }
        "sync.conflict_policy" => apply_enum_sync_conflict_policy_opt(
            &mut patch.sync.conflict_policy,
            &update.op,
            &update.value,
        ),
        "sync.exclude" => apply_string_list_opt(&mut patch.sync.exclude, update),

        "repl.exit_cwd" => {
            apply_enum_repl_exit_cwd_opt(&mut patch.repl.exit_cwd, &update.op, &update.value)
        }
        "repl.max_pty_buffered_lines" => apply_i64_opt(
            &mut patch.repl.max_pty_buffered_lines,
            &update.op,
            &update.value,
        ),
        "llm.enabled" => apply_bool_opt(&mut patch.llm.enabled, &update.op, &update.value),
        "llm.gateway.enabled" => {
            apply_bool_opt(&mut patch.llm.gateway.enabled, &update.op, &update.value)
        }
        "llm.gateway.mode" => {
            apply_enum_llm_gateway_mode_opt(&mut patch.llm.gateway.mode, &update.op, &update.value)
        }
        "llm.routing.default_backend" => apply_backend_id_or_empty_opt(
            &mut patch.llm.routing.default_backend,
            &update.op,
            &update.value,
            "llm.routing.default_backend",
        ),
        "agents.enabled" => apply_bool_opt(&mut patch.agents.enabled, &update.op, &update.value),
        "agents.defaults.selection" => {
            if update.op != UpdateOp::Set {
                return Err(user_error(
                    "agents.defaults.selection requires a complete replacement",
                ));
            }
            let selection: AgentSelectionTupleV1 =
                serde_json::from_str(&update.value).map_err(|error| {
                    user_error(format!(
                        "invalid complete agents.defaults.selection JSON: {error}"
                    ))
                })?;
            selection.validate().map_err(user_error)?;
            let changed = patch.agents.defaults.selection.as_ref() != Some(&selection);
            patch.agents.defaults.selection = Some(selection);
            Ok(changed)
        }
        "agents.defaults.execution.scope" => apply_enum_agent_execution_scope_opt(
            &mut patch.agents.defaults.execution.scope,
            &update.op,
            &update.value,
        ),
        "agents.defaults.cli.mode" => apply_enum_agent_cli_mode_opt(
            &mut patch.agents.defaults.cli.mode,
            &update.op,
            &update.value,
        ),
        "agents.hub.orchestrator_agent_id" => apply_string_opt(
            &mut patch.agents.hub.orchestrator_agent_id,
            &update.op,
            &update.value,
        ),
        "agents.hub.world_restart.on_drift" => apply_enum_world_restart_on_drift_opt(
            &mut patch.agents.hub.world_restart.on_drift,
            &update.op,
            &update.value,
        ),
        "agents.toolbox.enabled" => {
            apply_bool_opt(&mut patch.agents.toolbox.enabled, &update.op, &update.value)
        }
        "agents.toolbox.bind.transport" => apply_enum_agent_toolbox_bind_transport_opt(
            &mut patch.agents.toolbox.bind.transport,
            &update.op,
            &update.value,
        ),

        _ => Err(user_error(format!(
            "unsupported config key '{}'",
            update.key
        ))),
    }
}

fn reset_patch_key(patch: &mut SubstrateConfigPatch, key: &str) -> Result<bool> {
    match key {
        "world.enabled" => reset_opt(&mut patch.world.enabled),
        "world.anchor_mode" => reset_opt(&mut patch.world.anchor_mode),
        "world.anchor_path" => reset_opt(&mut patch.world.anchor_path),
        "world.caged" => reset_opt(&mut patch.world.caged),
        "world.net.filter" => reset_opt(&mut patch.world.net.filter),
        "world.env.inherit_from_host" => reset_opt(&mut patch.world.env.inherit_from_host),

        "world.deps.enabled" => reset_opt(&mut patch.world.deps.enabled),
        "world.deps.inventory_mode" => reset_opt(&mut patch.world.deps.inventory_mode),
        "world.deps.builtins" => reset_opt(&mut patch.world.deps.builtins),

        "policy.mode" => reset_opt(&mut patch.policy.mode),

        "sync.auto_sync" => reset_opt(&mut patch.sync.auto_sync),
        "sync.direction" => reset_opt(&mut patch.sync.direction),
        "sync.conflict_policy" => reset_opt(&mut patch.sync.conflict_policy),
        "sync.exclude" => reset_opt(&mut patch.sync.exclude),

        "repl.exit_cwd" => reset_opt(&mut patch.repl.exit_cwd),
        "repl.max_pty_buffered_lines" => reset_opt(&mut patch.repl.max_pty_buffered_lines),
        "llm.enabled" => reset_opt(&mut patch.llm.enabled),
        "llm.gateway.enabled" => reset_opt(&mut patch.llm.gateway.enabled),
        "llm.gateway.mode" => reset_opt(&mut patch.llm.gateway.mode),
        "llm.routing.default_backend" => reset_opt(&mut patch.llm.routing.default_backend),
        "agents.enabled" => reset_opt(&mut patch.agents.enabled),
        "agents.defaults.selection" => reset_opt(&mut patch.agents.defaults.selection),
        "agents.defaults.execution.scope" => reset_opt(&mut patch.agents.defaults.execution.scope),
        "agents.defaults.cli.mode" => reset_opt(&mut patch.agents.defaults.cli.mode),
        "agents.hub.orchestrator_agent_id" => {
            reset_opt(&mut patch.agents.hub.orchestrator_agent_id)
        }
        "agents.hub.world_restart.on_drift" => {
            reset_opt(&mut patch.agents.hub.world_restart.on_drift)
        }
        "agents.toolbox.enabled" => reset_opt(&mut patch.agents.toolbox.enabled),
        "agents.toolbox.bind.transport" => reset_opt(&mut patch.agents.toolbox.bind.transport),

        _ => Err(user_error(format!("unsupported config key '{}'", key))),
    }
}

fn reset_opt<T>(target: &mut Option<T>) -> Result<bool> {
    let changed = target.is_some();
    *target = None;
    Ok(changed)
}

fn apply_bool_opt(target: &mut Option<bool>, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for boolean key"));
    }
    let next = parse_bool_flag(raw).ok_or_else(|| {
        user_error(format!(
            "invalid boolean '{}'; expected true|false|1|0|yes|no|on|off",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_i64_opt(target: &mut Option<i64>, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for integer key"));
    }
    let next = raw.trim().parse::<i64>().map_err(|_| {
        user_error(format!(
            "invalid integer '{}'; expected a whole number",
            raw.trim()
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_anchor_mode_opt(
    target: &mut Option<WorldRootMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldRootMode::parse(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.anchor_mode '{}'; expected workspace, follow-cwd, or custom",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_policy_mode_opt(
    target: &mut Option<PolicyMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = PolicyMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid policy.mode '{}'; expected disabled, observe, or enforce",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_repl_exit_cwd_opt(
    target: &mut Option<ReplExitCwdMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = ReplExitCwdMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid repl.exit_cwd '{}'; expected entered or last_world",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_world_restart_on_drift_opt(
    target: &mut Option<WorldRestartOnDriftMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldRestartOnDriftMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid agents.hub.world_restart.on_drift '{}'; expected auto_restart or fail_closed",
            raw
        ))
    })?;
    let changed = target.as_ref() != Some(&next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_llm_gateway_mode_opt(
    target: &mut Option<LlmGatewayMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = LlmGatewayMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid llm.gateway.mode '{}'; expected in_world or host_only",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_agent_execution_scope_opt(
    target: &mut Option<AgentExecutionScope>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = AgentExecutionScope::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid agents.defaults.execution.scope '{}'; expected host or world",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_agent_cli_mode_opt(
    target: &mut Option<AgentCliMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = AgentCliMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid agents.defaults.cli.mode '{}'; expected persistent or per_request",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_agent_toolbox_bind_transport_opt(
    target: &mut Option<AgentToolboxBindTransport>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = AgentToolboxBindTransport::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid agents.toolbox.bind.transport '{}'; expected uds or tcp",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_sync_direction_opt(
    target: &mut Option<SyncDirection>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = SyncDirection::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid sync.direction '{}'; expected from_world, from_host, or both",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_sync_conflict_policy_opt(
    target: &mut Option<SyncConflictPolicy>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = SyncConflictPolicy::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid sync.conflict_policy '{}'; expected prefer_host, prefer_world, or abort",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_inventory_mode_opt(
    target: &mut Option<WorldDepsInventoryMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldDepsInventoryMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.deps.inventory_mode '{}'; expected merged or workspace_only",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_enum_builtins_opt(
    target: &mut Option<WorldDepsBuiltinsMode>,
    op: &UpdateOp,
    raw: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for enum key"));
    }
    let next = WorldDepsBuiltinsMode::parse_insensitive(raw).ok_or_else(|| {
        user_error(format!(
            "invalid world.deps.builtins '{}'; expected enabled or disabled",
            raw
        ))
    })?;
    let changed = *target != Some(next);
    *target = Some(next);
    Ok(changed)
}

fn apply_string_opt(target: &mut Option<String>, op: &UpdateOp, raw: &str) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for string key"));
    }
    let changed = target.as_deref() != Some(raw);
    *target = Some(raw.to_string());
    Ok(changed)
}

fn apply_backend_id_or_empty_opt(
    target: &mut Option<String>,
    op: &UpdateOp,
    raw: &str,
    key: &str,
) -> Result<bool> {
    if *op != UpdateOp::Set {
        return Err(user_error("unsupported operator for string key"));
    }
    validate_backend_id_or_empty(raw, key)?;
    let changed = target.as_deref() != Some(raw);
    *target = Some(raw.to_string());
    Ok(changed)
}

fn apply_string_list_opt(target: &mut Option<Vec<String>>, update: &ConfigUpdate) -> Result<bool> {
    match update.op {
        UpdateOp::Set => {
            let mut parsed: Vec<String> = serde_yaml::from_str(&update.value).map_err(|_| {
                user_error(format!(
                    "{} with '=' must be a YAML list literal (e.g., [] or [\"a\",\"b\"]); got '{}'",
                    update.key, update.value
                ))
            })?;
            if update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(&mut parsed);
            }
            let changed = target.as_ref() != Some(&parsed);
            *target = Some(parsed);
            Ok(changed)
        }
        UpdateOp::Append => {
            let list = target.get_or_insert_with(Vec::new);
            if list.iter().any(|item| item == &update.value) {
                return Ok(false);
            }
            list.push(update.value.clone());
            if update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(list);
            }
            Ok(true)
        }
        UpdateOp::Remove => {
            let Some(list) = target.as_mut() else {
                return Ok(false);
            };
            let before = list.len();
            list.retain(|item| item != &update.value);
            let changed = before != list.len();
            if changed && update.key == "world.deps.enabled" {
                dedupe_ordered_set_in_place(list);
            }
            Ok(changed)
        }
    }
}

fn dedupe_ordered_set_in_place(items: &mut Vec<String>) {
    let mut out: Vec<String> = Vec::with_capacity(items.len());
    for item in items.drain(..) {
        if !out.iter().any(|existing| existing == &item) {
            out.push(item);
        }
    }
    *items = out;
}

#[cfg(test)]
pub(crate) const PROTECTED_SELECTION_CONFIG_FIXTURES_V1: &[&str] = &[
    "world: {anchor_path: &model exact-model}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: *model}}}\n",
    "agents: {defaults: {selection: &tuple {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model}}}\n",
    "agents: &agents {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model}}}\n",
    "agents: {defaults: {selection: {harness: codex, provider: !!str openai, connection_profile: 'built_in:codex-subscription', model: exact-model}}}\n",
    "agents: {defaults: {<<: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model}}}}\n",
    "agents: {defaults: {selection: {harness: codex}}}\n",
    "agents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model, unknown: true}}}\n",
    "agents: {connection_profiles: {'global:broken': {}}}\n",
    "agents: {connection_profiles: {'global:broken': {revision: '', provider: openai, billing: subscription, endpoint_identity: openai-subscription, upstream_base_url: 'https://chatgpt.com/backend-api', auth: {auth_kind: codex_account, reference: configured-account}}}}\n",
    "agents: {connection_profiles: {'workspace:team': {revision: '1', provider: openai, billing: subscription, endpoint_identity: openai-subscription, upstream_base_url: 'https://chatgpt.com/backend-api', auth: {auth_kind: codex_account, reference: configured-account}}}}\n",
    "[{agents: {defaults: {selection: {harness: codex}}}}]\n",
    "agents: [{defaults: {selection: {harness: codex}}}]\n",
    "agents: {defaults: [{selection: {harness: codex}}]}\n",
    "world: [\n",
    "world: {enabled: invalid}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: pinned-model}}}\n",
];

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(unix)]
    use crate::execution::agent_runtime::HostSessionAuthority;
    use serial_test::serial;
    use std::ffi::OsString;
    use std::fs;
    use tempfile::TempDir;

    fn compose_selection_test_layers(
        global: &SubstrateConfigPatch,
        global_path: &Path,
        workspace: Option<(&SubstrateConfigPatch, &Path)>,
    ) -> (SubstrateConfig, ConfigExplainV1) {
        let (config, explain) = resolve_effective_from_layers(
            global,
            global_path,
            workspace,
            &EnvOverrides::default(),
            &CliConfigOverrides::default(),
            true,
            true,
        )
        .unwrap();
        (config, explain.unwrap())
    }

    #[test]
    fn test_config_yaml_preserves_unrelated_and_mixed_aliases() {
        let path = Path::new("/caller-held/config.yaml");
        let selection = "agents:\n  defaults:\n    selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model}\n";
        let world = "world:\n  enabled: &flag false\n  caged: *flag\n";
        for raw in [
            world.to_string(),
            format!("{world}{selection}"),
            format!("{world}{SELECTION_GLOBAL_PROFILE}").replace("world: {caged: false}\n", ""),
        ] {
            let patch = parse_config_patch_yaml(path, &raw).unwrap();
            assert_eq!(patch.world.enabled, Some(false));
            assert_eq!(patch.world.caged, Some(false));
            assert_eq!(
                patch.source_sha256,
                Some(format!("{:x}", Sha256::digest(raw.as_bytes())))
            );
        }
        for raw in [
            "agents: &old {enabled: true, defaults: {execution: {scope: world}}}\n",
            "world: {anchor_path: &word world}\nagents: {defaults: {execution: {scope: *word}}}\n",
            "world: {enabled: !!bool false}\n",
            "world:\n  anchor_path: |-\n    *text &text !text\n    ---\n",
        ] {
            parse_config_patch_yaml(path, raw).unwrap();
        }
        let raw = format!("{world}{selection}");
        let overrides = CapturedConfigOverridesV1 {
            environment: EnvOverrides::default(),
            cli: CliConfigOverrides::default(),
        };
        let (config, _) = resolve_effective_config_from_sources_v1(
            path,
            Some(raw.as_bytes()),
            Path::new("/caller-held/workspace.yaml"),
            b"{}\n",
            &overrides,
            &substrate_broker::Policy::default(),
        )
        .unwrap();
        assert!(!config.world.enabled && !config.world.caged);
        assert_eq!(config.agents.defaults.selection.model, "exact-model");
        assert!(config.selection_sources.is_empty());
        assert!(config.selection_profiles.is_empty());
        assert!(config.selection_preference_source.is_none());
    }

    #[test]
    fn test_config_yaml_refuses_indirect_selection_profile_provenance() {
        let path = Path::new("/caller-held/provenance.yaml");
        let tuple = "{harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: exact-model}";
        let attacks = [
            format!("agents: {{defaults: {{selection: &tuple {tuple}}}}}\n"),
            format!("agents: &agents {{defaults: {{selection: {tuple}}}}}\n"),
            format!("agents: {{defaults: &defaults {{selection: {tuple}}}}}\n"),
            format!("&root {{agents: {{defaults: {{selection: {tuple}}}}}}}\n"),
            format!("!!map {{agents: {{defaults: {{selection: {tuple}}}}}}}\n"),
            format!("agents: !!map {{defaults: {{selection: {tuple}}}}}\n"),
            format!("agents: {{defaults: !!map {{selection: {tuple}}}}}\n"),
            format!("agents: {{defaults: {{selection: !!map {tuple}}}}}\n"),
            "world: {anchor_path: &model exact-model}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: *model}}}\n".into(),
            "agents: {defaults: {selection: {harness: codex, provider: !!str openai, connection_profile: 'built_in:codex-subscription', model: exact-model}}}\n".into(),
            format!("world: {{anchor_path: &key selection}}\nagents: {{defaults: {{*key : {tuple}}}}}\n"),
            format!("agents: {{defaults: {{&key selection: {tuple}}}}}\n"),
            format!("template: &template {{defaults: {{selection: {tuple}}}}}\nagents: *template\n"),
            format!("template: &template {{selection: {tuple}}}\nagents: {{defaults: *template}}\n"),
            format!("template: &template {tuple}\nagents: {{defaults: {{selection: *template}}}}\n"),
            format!("<<: {{agents: {{defaults: {{selection: {tuple}}}}}}}\n"),
            format!("template: &template {{agents: {{defaults: {{selection: {tuple}}}}}}}\n<<: [*template]\n"),
            format!("agents: {{<<: {{defaults: {{selection: {tuple}}}}}}}\n"),
            format!("agents: {{defaults: {{<<: {{selection: {tuple}}}}}}}\n"),
            format!("agents: {{defaults: {{selection: {{<<: {tuple}, model: overridden}}}}}}\n"),
            "agents: {connection_profiles: &profiles {}}\n".into(),
            "agents: {connection_profiles: !!map {}}\n".into(),
            "template: &profiles {}\nagents: {connection_profiles: *profiles}\n".into(),
            "template: &agents {connection_profiles: {}}\nagents: *agents\n".into(),
            "agents: {<<: {connection_profiles: {}}}\n".into(),
            "world: {anchor_path: &key connection_profiles}\nagents: {*key : {}}\n".into(),
            "agents: {connection_profiles: {<<: {}}}\n".into(),
            SELECTION_GLOBAL_PROFILE.replace("revision: global-r1", "revision: &revision global-r1"),
            SELECTION_GLOBAL_PROFILE.replace("reference: selected-account", "reference: !!str selected-account"),
        ];
        for raw in attacks {
            let direct = validate_config_yaml_selection_paths_v1(path, &raw)
                .unwrap_err()
                .to_string();
            assert!(
                direct.contains("selection/profile YAML provenance"),
                "{direct}; {raw}"
            );
            let integrated = parse_config_patch_yaml(path, &raw).unwrap_err().to_string();
            assert!(
                integrated.contains("selection/profile YAML provenance"),
                "{integrated}"
            );
            assert!(integrated.contains(path.to_str().unwrap()));
        }
    }

    #[test]
    fn test_config_yaml_retains_structural_refusals_and_exact_source_hash() {
        let path = Path::new("/caller-held/structure.yaml");
        for raw in [
            "world: {enabled: true, enabled: false}\n",
            "agents: {connection_profiles: {'global:duplicate': {}, 'global:duplicate': {}}}\n",
            "agents: {defaults: {selection: {harness: codex, provider: openai, provider: other, connection_profile: 'built_in:codex-subscription', model: m}}}\n",
        ] {
            assert!(parse_config_patch_yaml(path, raw).unwrap_err().to_string().contains("duplicate YAML mapping key"));
        }
        for raw in [
            "world: {enabled: true, enabled: false}\n",
            "agents: {defaults: {selection: {harness: codex, provider: openai, provider: other, connection_profile: 'built_in:codex-subscription', model: m}}}\n",
            "agents: {connection_profiles: {'global:duplicate': {}, 'global:duplicate': {}}}\n",
            "world: {}\n---\nworld: {}\n",
            "---\nworld: {}\n---\nagents: {}\n",
            "world: {unknown: true}\n",
            "agents: {defaults: {selection: {harness: codex}}}\n",
            "agents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: m, unknown: true}}}\n",
            "agents: {connection_profiles: {'global:empty': {}}}\n",
        ] {
            assert!(parse_config_patch_yaml(path, raw).is_err(), "{raw}");
        }
        let raw = "world: {enabled: &flag false, caged: *flag}\n";
        let first = parse_config_patch_yaml(path, raw).unwrap();
        let second =
            parse_config_patch_yaml(path, &format!("{raw}# same values, new raw revision\n"))
                .unwrap();
        assert_eq!(
            serde_json::to_value(&first).unwrap(),
            serde_json::to_value(&second).unwrap()
        );
        assert_ne!(first.source_sha256, second.source_sha256);
        assert!(
            first.captured_selection_source.is_none() && second.captured_selection_source.is_none()
        );
    }

    #[test]
    fn test_selection_defaults_replace_whole_block_and_reject_partial_unknown() {
        let global_path = Path::new("/caller-held/global/config.yaml");
        let workspace_path = Path::new("/caller-held/workspace/workspace.yaml");
        let global = parse_config_patch_yaml(global_path,
            "agents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'global:subscription', model: global-model}}}\n").unwrap();
        let workspace = parse_config_patch_yaml(workspace_path,
            "agents: {defaults: {selection: {harness: claude_code, provider: anthropic, connection_profile: 'workspace:team', model: workspace-model}}}\n").unwrap();
        let (config, explain) =
            compose_selection_test_layers(&global, global_path, Some((&workspace, workspace_path)));
        assert_eq!(
            config.agents.defaults.selection,
            workspace.agents.defaults.selection.clone().unwrap()
        );
        assert_eq!(
            config.agents.defaults.selection.harness,
            AgentHarnessFamilyV1::ClaudeCode
        );
        assert_eq!(config.agents.defaults.selection.provider, "anthropic");
        assert_eq!(
            config.agents.defaults.selection.connection_profile,
            "workspace:team"
        );
        assert_eq!(config.agents.defaults.selection.model, "workspace-model");
        assert_eq!(
            explain.keys.0["agents.defaults.selection"].sources[0].layer,
            "workspace_patch"
        );
        assert_eq!(
            explain.keys.0["agents.defaults.selection"].sources[0]
                .path
                .as_deref(),
            workspace_path.to_str()
        );
        let (without_workspace, _) = compose_selection_test_layers(&global, global_path, None);
        assert_eq!(
            without_workspace.agents.defaults.selection,
            global.agents.defaults.selection.clone().unwrap()
        );

        let complete = serde_json::json!({"harness":"codex", "provider":"openai", "connection_profile":"global:subscription", "model":"exact-model"});
        for path in [global_path, workspace_path] {
            for missing in ["harness", "provider", "connection_profile", "model"] {
                let mut partial = complete.clone();
                partial.as_object_mut().unwrap().remove(missing);
                let raw = format!("agents:\n  defaults:\n    selection: {}\n", partial);
                assert!(parse_config_patch_yaml(path, &raw)
                    .unwrap_err()
                    .to_string()
                    .contains(path.to_str().unwrap()));
            }
            let mut unknown = complete.clone();
            unknown["agent_id"] = serde_json::json!("unapproved");
            let raw = format!("agents:\n  defaults:\n    selection: {}\n", unknown);
            assert!(parse_config_patch_yaml(path, &raw).is_err());
        }
    }

    #[test]
    #[serial]
    fn test_selection_source_cache_rechecks_exact_bytes_when_size_and_mtime_match() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let temp = TempDir::new().unwrap();
        let home = temp.path().join("home");
        fs::create_dir(&home).unwrap();
        let _home = EnvGuard::set("SUBSTRATE_HOME", &home);
        let path = home.join("config.yaml");
        fs::write(&path, SELECTION_GLOBAL_PROFILE).unwrap();
        invalidate_config_cache();
        let (first, _, _) = load_config_patch_layers_cached(temp.path()).unwrap();
        let before = fs::metadata(&path).unwrap();
        let changed = SELECTION_GLOBAL_PROFILE.replace("exact-model", "other-model");
        assert_eq!(changed.len(), SELECTION_GLOBAL_PROFILE.len());
        fs::write(&path, &changed).unwrap();
        fs::File::open(&path)
            .unwrap()
            .set_modified(before.modified().unwrap())
            .unwrap();
        assert_eq!(
            FileStatKey::for_path(&path).unwrap(),
            FileStatKey {
                exists: true,
                mtime: before.modified().ok(),
                size: Some(before.len())
            }
        );
        let (second, _, _) = load_config_patch_layers_cached(temp.path()).unwrap();
        assert_eq!(
            first.agents.defaults.selection.as_ref().unwrap().model,
            "exact-model"
        );
        assert_eq!(
            second.agents.defaults.selection.as_ref().unwrap().model,
            "other-model"
        );
        assert_ne!(first.source_sha256, second.source_sha256);
        let (resolved, _) = compose_selection_test_layers(&second, &path, None);
        let source = resolved.selection_preference_source.unwrap();
        assert_eq!(
            source.sha256,
            format!("{:x}", Sha256::digest(changed.as_bytes()))
        );
        invalidate_config_cache();
    }

    const SELECTION_GLOBAL_PROFILE: &str = "agents:\n  defaults:\n    selection: {harness: codex, provider: openai, connection_profile: 'global:subscription', model: exact-model}\n  connection_profiles:\n    'global:subscription':\n      revision: global-r1\n      provider: openai\n      billing: subscription\n      endpoint_identity: openai-codex-subscription\n      upstream_base_url: https://chatgpt.com/backend-api\n      auth: {auth_kind: codex_account, reference: selected-account}\nworld: {caged: false}\n";

    #[test]
    fn test_selection_profile_semantics_refuse_invalid_pure_and_captured_inputs() {
        let temp = TempDir::new().unwrap();
        let global_path = temp.path().join("config.yaml");
        let workspace_path = temp.path().join("workspace.yaml");
        let overrides = CapturedConfigOverridesV1 {
            environment: EnvOverrides::default(),
            cli: CliConfigOverrides::default(),
        };
        let policy = substrate_broker::Policy::default();
        let original: serde_json::Value = serde_yaml::from_str(SELECTION_GLOBAL_PROFILE).unwrap();
        for scope in [
            AgentSelectionSourceScopeV1::Global,
            AgentSelectionSourceScopeV1::Workspace,
        ] {
            let id = format!("{}:subscription", scope.as_str());
            let mut valid = original.clone();
            let profile = valid["agents"]["connection_profiles"]
                .as_object_mut()
                .unwrap()
                .remove("global:subscription")
                .unwrap();
            valid["agents"]["connection_profiles"][&id] = profile;
            valid["agents"]["defaults"]["selection"]["connection_profile"] = serde_json::json!(id);
            let path = if scope == AgentSelectionSourceScopeV1::Global {
                &global_path
            } else {
                &workspace_path
            };
            let compose_pure = |raw: &str| {
                let (global_bytes, workspace_bytes) =
                    if scope == AgentSelectionSourceScopeV1::Global {
                        (Some(raw.as_bytes()), b"{}\n".as_slice())
                    } else {
                        (None, raw.as_bytes())
                    };
                resolve_effective_config_from_sources_v1(
                    &global_path,
                    global_bytes,
                    &workspace_path,
                    workspace_bytes,
                    &overrides,
                    &policy,
                )
            };
            let valid_raw = serde_yaml::to_string(&valid).unwrap();
            let (pure, _) = compose_pure(&valid_raw).unwrap();
            assert!(pure.selection_sources.is_empty());
            assert!(pure.selection_profiles.is_empty());
            assert!(pure.selection_preference_source.is_none());
            let captured = parse_captured_config_patch(path, &valid_raw, scope).unwrap();
            let empty = captured_absent_config_patch();
            let (physical, _) = if scope == AgentSelectionSourceScopeV1::Global {
                compose_selection_test_layers(&captured, &global_path, None)
            } else {
                compose_selection_test_layers(
                    &empty,
                    &global_path,
                    Some((&captured, &workspace_path)),
                )
            };
            assert_eq!(
                serde_json::to_value(&pure).unwrap(),
                serde_json::to_value(&physical).unwrap()
            );
            assert!(physical
                .selection_profiles
                .iter()
                .any(|profile| profile.id == id));

            let mut empty_profile = valid.clone();
            empty_profile["agents"]["connection_profiles"][&id] = serde_json::json!({});
            let raw = serde_yaml::to_string(&empty_profile).unwrap();
            assert!(compose_pure(&raw).is_err());
            assert!(parse_captured_config_patch(path, &raw, scope).is_err());

            let cases = [
                ("revision", serde_json::json!("")),
                ("provider", serde_json::json!("")),
                ("endpoint_identity", serde_json::json!("")),
                ("upstream_base_url", serde_json::json!("")),
                (
                    "upstream_base_url",
                    serde_json::json!("http://example.test/v1"),
                ),
                (
                    "upstream_base_url",
                    serde_json::json!("https://secret@example.test/v1"),
                ),
                (
                    "auth",
                    serde_json::json!({"auth_kind":"codex_account", "reference":""}),
                ),
                (
                    "auth",
                    serde_json::json!({"auth_kind":"api_key_environment", "name":""}),
                ),
                (
                    "auth",
                    serde_json::json!({"auth_kind":"api_key_environment", "name":"9BAD"}),
                ),
            ];
            for (field, value) in cases {
                let mut invalid = valid.clone();
                invalid["agents"]["connection_profiles"][&id][field] = value;
                let raw = serde_yaml::to_string(&invalid).unwrap();
                let pure_error = compose_pure(&raw).unwrap_err().to_string();
                let captured = parse_captured_config_patch(path, &raw, scope).unwrap();
                let (global, workspace) = if scope == AgentSelectionSourceScopeV1::Global {
                    (&captured, None)
                } else {
                    (&empty, Some((&captured, workspace_path.as_path())))
                };
                let physical_error = resolve_effective_from_layers(
                    global,
                    &global_path,
                    workspace,
                    &EnvOverrides::default(),
                    &CliConfigOverrides::default(),
                    true,
                    true,
                )
                .unwrap_err()
                .to_string();
                assert_eq!(
                    pure_error, physical_error,
                    "same profile value checks for {scope:?}/{field}"
                );
                assert!(pure_error.contains(path.to_str().unwrap()), "{pure_error}");
                assert!(
                    pure_error.contains("invalid connection profile"),
                    "{pure_error}"
                );
            }
        }
    }

    #[test]
    fn test_selection_profiles_preserve_source_namespaces_endpoint_and_auth() {
        let temp = TempDir::new().unwrap();
        let global_path = temp.path().join("config.yaml");
        let workspace_path = temp.path().join("workspace.yaml");
        let global = parse_captured_config_patch(
            &global_path,
            SELECTION_GLOBAL_PROFILE,
            AgentSelectionSourceScopeV1::Global,
        )
        .unwrap();
        let workspace_raw = "agents:\n  defaults:\n    selection: {harness: codex, provider: openai, connection_profile: 'workspace:team', model: workspace-model}\n  connection_profiles:\n    'workspace:team':\n      revision: local-r2\n      provider: openai\n      billing: subscription\n      endpoint_identity: openai-codex-subscription\n      upstream_base_url: https://chatgpt.com/backend-api\n      auth: {auth_kind: codex_account, reference: local-account}\n";
        let workspace = parse_captured_config_patch(
            &workspace_path,
            workspace_raw,
            AgentSelectionSourceScopeV1::Workspace,
        )
        .unwrap();
        let (config, _) = compose_selection_test_layers(
            &global,
            &global_path,
            Some((&workspace, &workspace_path)),
        );
        let selected = config
            .selection_profiles
            .iter()
            .find(|profile| profile.id == "global:subscription")
            .unwrap();
        assert_eq!(selected.revision, "global-r1");
        assert_eq!(selected.source.scope, AgentSelectionSourceScopeV1::Global);
        assert_eq!(selected.endpoint_identity, "openai-codex-subscription");
        assert_eq!(
            selected.upstream_base_url,
            "https://chatgpt.com/backend-api"
        );
        assert_eq!(
            selected.auth,
            AgentConnectionAuthV1::CodexAccount {
                reference: "selected-account".into()
            }
        );
        let local = config
            .selection_profiles
            .iter()
            .find(|profile| profile.id == "workspace:team")
            .unwrap();
        assert_eq!(local.source.scope, AgentSelectionSourceScopeV1::Workspace);
        assert_eq!(
            local.auth,
            AgentConnectionAuthV1::CodexAccount {
                reference: "local-account".into()
            }
        );
        assert_eq!(config.agents.defaults.selection.model, "workspace-model");
        assert_eq!(
            config.agents.defaults.selection.connection_profile,
            "workspace:team"
        );
        assert_eq!(
            config.selection_preference_source.as_ref(),
            Some(&local.source)
        );
        assert_eq!(
            local.source.sha256,
            format!("{:x}", Sha256::digest(workspace_raw.as_bytes()))
        );
        for malicious in [
            workspace_raw.replace("workspace:team", "global:subscription"),
            workspace_raw
                .replace("workspace:team", "global:subscription")
                .replace(
                    "https://chatgpt.com/backend-api",
                    "https://other.example/api",
                ),
            workspace_raw
                .replace("workspace:team", "global:subscription")
                .replace("local-account", "substituted-account"),
            workspace_raw.replace("workspace:team", "built_in:codex-subscription"),
        ] {
            let patch = parse_config_patch_yaml(&workspace_path, &malicious).unwrap();
            let error = resolve_effective_from_layers(
                &global,
                &global_path,
                Some((&patch, &workspace_path)),
                &EnvOverrides::default(),
                &CliConfigOverrides::default(),
                true,
                true,
            )
            .unwrap_err();
            assert!(error
                .to_string()
                .contains("cannot redefine another source namespace"));
            assert!(error.to_string().contains(workspace_path.to_str().unwrap()));
        }
    }

    #[test]
    fn test_selection_complete_set_reset_preserves_unrelated_config_and_invalidates_capture() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.yaml");
        let original = parse_captured_config_patch(
            &path,
            SELECTION_GLOBAL_PROFILE,
            AgentSelectionSourceScopeV1::Global,
        )
        .unwrap();
        let before = serde_json::to_value(&original).unwrap();
        let replacement = serde_json::json!({"harness":"claude_code", "provider":"anthropic", "connection_profile":"workspace:team", "model":"changed-exact-model"});
        for value in [
            r#"{"harness":"codex"}"#,
            r#"{"harness":"codex","provider":"openai","connection_profile":"global:subscription","model":"m","extra":true}"#,
        ] {
            let mut patch = original.clone();
            let update = ConfigUpdate {
                key: "agents.defaults.selection".into(),
                op: UpdateOp::Set,
                value: value.into(),
            };
            assert!(apply_updates_to_patch(&mut patch, &[update]).is_err());
            assert_eq!(patch, original);
        }
        for op in [UpdateOp::Append, UpdateOp::Remove] {
            let mut patch = original.clone();
            assert!(apply_updates_to_patch(
                &mut patch,
                &[ConfigUpdate {
                    key: "agents.defaults.selection".into(),
                    op,
                    value: replacement.to_string()
                }]
            )
            .is_err());
            assert_eq!(patch, original);
        }
        let mut patch = original.clone();
        let update = ConfigUpdate {
            key: "agents.defaults.selection".into(),
            op: UpdateOp::Set,
            value: replacement.to_string(),
        };
        assert!(apply_updates_to_patch(&mut patch, std::slice::from_ref(&update)).unwrap());
        let mut expected = before.clone();
        expected["agents"]["defaults"]["selection"] = replacement;
        assert_eq!(serde_json::to_value(&patch).unwrap(), expected);
        assert!(patch.captured_selection_source.is_none());
        assert!(patch.source_sha256.is_none());
        assert!(!apply_updates_to_patch(&mut patch, &[update]).unwrap());
        assert!(reset_patch_keys(&mut patch, &["agents.defaults.selection".into()]).unwrap());
        expected["agents"]["defaults"]
            .as_object_mut()
            .unwrap()
            .remove("selection");
        if expected["agents"]["defaults"]
            .as_object()
            .unwrap()
            .is_empty()
        {
            expected["agents"]
                .as_object_mut()
                .unwrap()
                .remove("defaults");
        }
        assert_eq!(serde_json::to_value(&patch).unwrap(), expected);
        assert!(!reset_patch_keys(&mut patch, &["agents.defaults.selection".into()]).unwrap());
        let roundtrip =
            parse_config_patch_yaml(&path, &serde_yaml::to_string(&patch).unwrap()).unwrap();
        assert_eq!(serde_json::to_value(&roundtrip).unwrap(), expected);
        assert_eq!(
            roundtrip.agents.connection_profiles,
            original.agents.connection_profiles
        );
    }

    #[test]
    fn test_selection_source_capture_is_separate_from_caller_held_pure_composition() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.yaml");
        let raw = SELECTION_GLOBAL_PROFILE;
        let captured =
            parse_captured_config_patch(&path, raw, AgentSelectionSourceScopeV1::Global).unwrap();
        let captured_source = match captured.captured_selection_source.as_ref().unwrap() {
            CapturedSelectionConfigSourceV1::Present(source) => source.clone(),
            CapturedSelectionConfigSourceV1::Absent => panic!("present source required"),
        };
        assert_eq!(captured_source.scope, AgentSelectionSourceScopeV1::Global);
        assert_eq!(captured_source.relative_path, "config.yaml");
        assert_eq!(
            captured_source.sha256,
            format!("{:x}", Sha256::digest(raw.as_bytes()))
        );
        assert_eq!(captured_source.revision, captured_source.sha256);
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let metadata = fs::metadata(temp.path()).unwrap();
            assert_eq!(
                captured_source.root_identity,
                format!(
                    "{}:{}:{}",
                    metadata.dev(),
                    metadata.ino(),
                    temp.path().canonicalize().unwrap().display()
                )
            );
        }
        // Removing the real source root after capture must not add I/O to composition.
        temp.close().unwrap();
        let (physical, _) = compose_selection_test_layers(&captured, &path, None);
        assert_eq!(
            physical.selection_preference_source.as_ref(),
            Some(&captured_source)
        );
        assert!(physical.selection_sources.contains(&captured_source));
        let selected = physical
            .selection_profiles
            .iter()
            .find(|profile| profile.id == "global:subscription")
            .unwrap();
        assert_eq!(selected.source, captured_source);
        let pure_patch = parse_config_patch_yaml(&path, raw).unwrap();
        let (pure, _) = compose_selection_test_layers(&pure_patch, &path, None);
        assert_eq!(
            serde_json::to_value(&pure).unwrap(),
            serde_json::to_value(&physical).unwrap()
        );
        assert!(pure.selection_sources.is_empty());
        assert!(pure.selection_profiles.is_empty());
        assert!(pure.selection_preference_source.is_none());
        let overrides = CapturedConfigOverridesV1 {
            environment: EnvOverrides::default(),
            cli: CliConfigOverrides::default(),
        };
        let (caller_held, _) = resolve_effective_config_from_sources_v1(
            &path,
            Some(raw.as_bytes()),
            Path::new("/unopened/workspace/workspace.yaml"),
            b"{}\n",
            &overrides,
            &substrate_broker::Policy::default(),
        )
        .unwrap();
        assert_eq!(
            caller_held.agents.defaults.selection,
            physical.agents.defaults.selection
        );
        assert!(caller_held.selection_sources.is_empty());
        assert!(caller_held.selection_profiles.is_empty());
        assert!(caller_held.selection_preference_source.is_none());
        let snapshot = crate::execution::agent_inventory::AgentInventorySnapshotV3 {
            entries: BTreeMap::new(),
            definitions: Vec::new(),
        };
        assert!(snapshot
            .resolve_selection(&caller_held, None, None, AgentExecutionScope::Host)
            .unwrap_err()
            .to_string()
            .contains("no captured source"));
    }

    #[test]
    #[serial]
    fn workspace_settings_config_captures_overrides_once_and_preserves_precedence() {
        let _env = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        let mut cli = CliConfigOverrides {
            world_enabled: Some(true),
            ..Default::default()
        };
        let captured = CapturedConfigOverridesV1::capture(&cli).unwrap();
        cli.world_enabled = Some(false);
        let _changed_env = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "invalid-after-capture");
        let (config, explain) = resolve_effective_config_from_sources_v1(
            Path::new("/explicit/config.yaml"),
            Some(b"world: {enabled: false, caged: false}\n"),
            Path::new("/workspace/.substrate/workspace.yaml"),
            b"world: {enabled: false, caged: true}\n",
            &captured,
            &substrate_broker::Policy::default(),
        )
        .unwrap();
        assert!(config.world.enabled);
        assert!(config.world.caged);
        assert_eq!(explain.keys.0["world.enabled"].sources[0].layer, "cli_flag");
        assert_eq!(
            explain.keys.0["world.caged"].sources[0].layer,
            "workspace_patch"
        );
        assert_eq!(captured.environment.world_enabled, Some(false));
        assert_eq!(captured.cli.world_enabled, Some(true));
    }

    #[test]
    fn workspace_settings_config_explicit_layers_match_existing_composer() {
        let global_path = Path::new("/explicit/config.yaml");
        let workspace_path = Path::new("/workspace/.substrate/workspace.yaml");
        let global = b"sync: {exclude: [one, two]}\nworld: {caged: false}\n";
        let workspace = b"sync: {exclude: [two, three]}\nworld: {caged: true}\n";
        let overrides = CapturedConfigOverridesV1 {
            environment: EnvOverrides::default(),
            cli: CliConfigOverrides::default(),
        };
        let explicit = resolve_effective_config_from_sources_v1(
            global_path,
            Some(global),
            workspace_path,
            workspace,
            &overrides,
            &substrate_broker::Policy::default(),
        )
        .unwrap();
        let expected = resolve_effective_from_layers(
            &parse_config_patch_yaml(global_path, std::str::from_utf8(global).unwrap()).unwrap(),
            global_path,
            Some((
                &parse_config_patch_yaml(workspace_path, std::str::from_utf8(workspace).unwrap())
                    .unwrap(),
                workspace_path,
            )),
            &EnvOverrides::default(),
            &CliConfigOverrides::default(),
            true,
            true,
        )
        .unwrap();
        assert_eq!(explicit.0, expected.0);
        assert_eq!(Some(explicit.1), expected.1);
        assert_eq!(
            explicit.0.sync.exclude,
            vec![".git/**", ".substrate/**", "two", "three"]
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn test_e3f_classification_absence_and_source_errors() {
        use std::os::unix::fs::PermissionsExt;
        let home = tempfile::tempdir_in(std::env::var_os("HOME").unwrap()).unwrap();
        fs::set_permissions(home.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let workspace = tempfile::tempdir_in(home.path()).unwrap();
        let metadata = fs::metadata(workspace.path()).unwrap();
        let root = config_projection::CanonicalDirectoryV1 {
            physical_path: workspace.path().to_str().unwrap().into(),
            physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        };
        let authority = HostSessionAuthority::open(home.path()).unwrap();
        let bootstrap = authority.bootstrap_home();
        let snapshot = resolve_e3_effective_config_source_v1(workspace.path(), &bootstrap, &root)
            .expect("guarded absence must classify");
        let compatibility = resolve_effective_config_with_explain_for_bootstrap_home(
            workspace.path(),
            &CliConfigOverrides::default(),
            &bootstrap,
            true,
        )
        .unwrap();
        // Preserve equality of every semantic configuration field; physical
        // capture metadata now belongs only to the ordinary source owner.
        assert_eq!(
            serde_json::to_value(&snapshot.effective).unwrap(),
            serde_json::to_value(&compatibility.0).unwrap()
        );
        assert!(snapshot.effective.selection_sources.is_empty());
        assert!(snapshot.effective.selection_profiles.is_empty());
        assert!(snapshot.effective.selection_preference_source.is_none());
        assert_eq!(compatibility.0.selection_sources.len(), 1);
        assert_eq!(
            compatibility.0.selection_sources[0].scope,
            AgentSelectionSourceScopeV1::BuiltIn
        );
        assert_eq!(
            serde_json::to_value(&snapshot.explain).unwrap(),
            serde_json::to_value(compatibility.1.unwrap()).unwrap()
        );
        assert!(extract_e3_effective_config_source_v1(
            "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
            &snapshot
        )
        .is_err());
        assert!(!workspace.path().join(".substrate").exists());
    }

    #[cfg(unix)]
    #[test]
    fn explicit_bootstrap_home_explain_uses_selected_global_config_path() {
        let selected = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(tempfile::tempdir_in)
            .transpose()
            .expect("create selected bootstrap home")
            .unwrap_or_else(|| TempDir::new().expect("selected bootstrap home"));
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700))
                .expect("secure selected bootstrap home");
        }
        fs::write(
            selected.path().join("config.yaml"),
            "sync:\n  auto_sync: true\n",
        )
        .expect("write selected config");
        let workspace = TempDir::new().expect("workspace");
        let authority = HostSessionAuthority::open(selected.path()).expect("open authority");
        // A selected config remains subject to the trusted-reader mode rule.
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(
            selected.path().join("config.yaml"),
            fs::Permissions::from_mode(0o644),
        )
        .unwrap();
        assert!(authority.bootstrap_home().read_config_yaml().is_err());
        fs::set_permissions(
            selected.path().join("config.yaml"),
            fs::Permissions::from_mode(0o600),
        )
        .unwrap();
        let (config, explain) = resolve_effective_config_with_explain_for_bootstrap_home(
            workspace.path(),
            &CliConfigOverrides::default(),
            &authority.bootstrap_home(),
            true,
        )
        .expect("resolve selected config");

        assert!(config.sync.auto_sync);
        let global_source = config
            .selection_sources
            .iter()
            .find(|source| source.scope == AgentSelectionSourceScopeV1::Global)
            .unwrap();
        assert_eq!(global_source.relative_path, "config.yaml");
        assert_eq!(
            global_source.sha256,
            format!("{:x}", Sha256::digest(b"sync:\n  auto_sync: true\n"))
        );
        assert_eq!(global_source.revision, global_source.sha256);
        let explain =
            serde_json::to_string(&explain.expect("explain payload")).expect("serialize explain");
        assert!(explain.contains(&selected.path().join("config.yaml").display().to_string()));
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn test_e3f_sealed_snapshot_authoring_lifetime() {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

        let selected = tempfile::tempdir_in(std::env::var_os("HOME").unwrap()).unwrap();
        fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700)).unwrap();
        let config = selected.path().join("config.yaml");
        fs::write(&config, "llm: {enabled: true, gateway: {enabled: true, mode: in_world}, routing: {default_backend: 'cli:codex-world'}}\nagents: {enabled: true, defaults: {execution: {scope: world}, cli: {mode: persistent}}}\nworld: {enabled: true}\n").unwrap();
        fs::set_permissions(&config, fs::Permissions::from_mode(0o600)).unwrap();
        let workspace = TempDir::new().unwrap();
        let control = workspace.path().join(".substrate");
        fs::create_dir(&control).unwrap();
        let workspace_config = control.join("workspace.yaml");
        fs::write(&workspace_config, "{}\n").unwrap();
        let metadata = fs::metadata(workspace.path()).unwrap();
        let root = config_projection::CanonicalDirectoryV1 {
            physical_path: workspace
                .path()
                .canonicalize()
                .unwrap()
                .to_str()
                .unwrap()
                .into(),
            physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        };
        let authority = HostSessionAuthority::open(selected.path()).unwrap();
        let snapshot = resolve_e3_effective_config_source_v1(
            workspace.path(),
            &authority.bootstrap_home(),
            &root,
        )
        .unwrap();
        let effective = snapshot.effective_config();
        assert!(effective.llm.enabled);
        let source = extract_e3_effective_config_source_v1(
            "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
            &snapshot,
        )
        .unwrap();
        assert_eq!(source.values.llm_enabled, effective.llm.enabled);
        let E3PinnedConfigPatchSourceV1::Workspace { held_file, .. } = &snapshot.workspace_patch
        else {
            panic!("missing workspace guard");
        };
        let fd = held_file.as_raw_fd();
        // An extracted owned value must not let publication ignore source replacement.
        let replacement = control.join("replacement.yaml");
        fs::write(&replacement, "{}\n").unwrap();
        fs::rename(&replacement, &workspace_config).unwrap();
        assert!(extract_e3_effective_config_source_v1(
            "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
            &snapshot,
        )
        .is_err());
        // SAFETY: F_GETFD observes the retained descriptor without modifying it.
        assert!(unsafe { libc::fcntl(fd, libc::F_GETFD) } >= 0);
        drop(snapshot);
        // No allocation/open occurs between drop and the descriptor observation.
        assert_eq!(unsafe { libc::fcntl(fd, libc::F_GETFD) }, -1);
        assert_eq!(
            std::io::Error::last_os_error().raw_os_error(),
            Some(libc::EBADF)
        );
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn e3c_effective_config_uses_one_pinned_source_snapshot() {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

        let selected = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(tempfile::tempdir_in)
            .transpose()
            .expect("create selected bootstrap home")
            .unwrap_or_else(|| TempDir::new().expect("selected bootstrap home"));
        fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700))
            .expect("secure selected bootstrap home");
        let global_path = selected.path().join("config.yaml");
        fs::write(
            &global_path,
            "llm:\n  enabled: true\n  gateway:\n    enabled: true\n    mode: in_world\n  routing:\n    default_backend: cli:codex-world\nagents:\n  enabled: true\n  defaults:\n    execution:\n      scope: world\n    cli:\n      mode: persistent\nworld:\n  enabled: true\n",
        )
        .expect("write selected config");
        fs::set_permissions(&global_path, fs::Permissions::from_mode(0o600))
            .expect("secure selected config");

        let workspace = TempDir::new().expect("workspace");
        let substrate = workspace.path().join(".substrate");
        fs::create_dir(&substrate).expect("create workspace control directory");
        fs::write(substrate.join("workspace.yaml"), "{}\n").expect("write workspace config");
        let workspace_path = workspace
            .path()
            .canonicalize()
            .expect("canonical workspace");
        let workspace_metadata = fs::metadata(&workspace_path).expect("workspace metadata");
        let expected_workspace = config_projection::CanonicalDirectoryV1 {
            physical_path: workspace_path.to_str().expect("workspace utf8").to_string(),
            physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
                device_id: workspace_metadata.dev(),
                inode: workspace_metadata.ino(),
            },
        };
        let authority = HostSessionAuthority::open(selected.path()).expect("open authority");
        let snapshot = resolve_e3_effective_config_source_v1(
            workspace.path(),
            &authority.bootstrap_home(),
            &expected_workspace,
        )
        .expect("resolve pinned E3 config");
        let source = extract_e3_effective_config_source_v1(
            "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
            &snapshot,
        )
        .expect("extract E3 source");

        assert_eq!(source.workspace_root, expected_workspace);
        assert_eq!(source.ordered_explain_origins.len(), 8);
        assert!(source.values.llm_enabled);
        assert!(source.values.managed_gateway_enabled);
        assert_eq!(source.values.default_backend_id, "cli:codex-world");
        assert!(source.source_revision.starts_with("ecsr1_"));
        assert_eq!(source.source_hash.len(), 64);
    }

    #[cfg(target_os = "linux")]
    #[test]
    #[serial]
    fn e3c_effective_config_rejects_workspace_inode_substitution() {
        use std::os::unix::fs::{MetadataExt as _, PermissionsExt as _};

        let selected = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(tempfile::tempdir_in)
            .transpose()
            .expect("create selected bootstrap home")
            .unwrap_or_else(|| TempDir::new().expect("selected bootstrap home"));
        fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700))
            .expect("secure selected bootstrap home");
        let global_path = selected.path().join("config.yaml");
        fs::write(
            &global_path,
            "llm: {enabled: true, gateway: {enabled: true, mode: in_world}, routing: {default_backend: cli:codex-world}}\nagents: {enabled: true, defaults: {execution: {scope: world}, cli: {mode: persistent}}}\nworld: {enabled: true}\n",
        )
        .expect("write selected config");
        fs::set_permissions(&global_path, fs::Permissions::from_mode(0o600))
            .expect("secure selected config");
        let workspace = TempDir::new().expect("workspace");
        let substrate = workspace.path().join(".substrate");
        fs::create_dir(&substrate).expect("create workspace control directory");
        let workspace_config = substrate.join("workspace.yaml");
        fs::write(&workspace_config, "{}\n").expect("write workspace config");
        let workspace_path = workspace
            .path()
            .canonicalize()
            .expect("canonical workspace");
        let metadata = fs::metadata(&workspace_path).expect("workspace metadata");
        let expected_workspace = config_projection::CanonicalDirectoryV1 {
            physical_path: workspace_path.to_str().expect("workspace utf8").to_string(),
            physical_identity: config_projection::DirectoryPhysicalIdentityV1::Linux {
                device_id: metadata.dev(),
                inode: metadata.ino(),
            },
        };
        let authority = HostSessionAuthority::open(selected.path()).expect("open authority");
        let snapshot = resolve_e3_effective_config_source_v1(
            workspace.path(),
            &authority.bootstrap_home(),
            &expected_workspace,
        )
        .expect("resolve pinned E3 config");
        let replacement = substrate.join("replacement.yaml");
        fs::write(&replacement, "{}\n").expect("write replacement");
        fs::rename(&replacement, &workspace_config).expect("replace workspace source inode");

        assert!(extract_e3_effective_config_source_v1(
            "cpa_018f0892-cc48-7a56-b711-9f17a2a81586",
            &snapshot,
        )
        .is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn e3c_bootstrap_config_rejects_source_byte_substitution() {
        use std::os::unix::fs::PermissionsExt as _;

        let selected = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(tempfile::tempdir_in)
            .transpose()
            .expect("create selected bootstrap home")
            .unwrap_or_else(|| TempDir::new().expect("selected bootstrap home"));
        fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700))
            .expect("secure selected bootstrap home");
        let config_path = selected.path().join("config.yaml");
        fs::write(&config_path, "world: {enabled: true}\n").expect("write config");
        fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
            .expect("secure selected config");
        let authority = HostSessionAuthority::open(selected.path()).expect("open authority");
        let opened = authority
            .bootstrap_home()
            .open_e3_config_source()
            .expect("open pinned source");
        fs::write(&config_path, "world: {enabled: false}\n").expect("substitute bytes");

        assert!(opened.revalidate().is_err());
    }

    #[cfg(target_os = "linux")]
    #[test]
    fn e3c_bootstrap_config_rejects_equal_byte_inode_substitution() {
        use std::os::unix::fs::PermissionsExt as _;

        let selected = std::env::var_os("XDG_RUNTIME_DIR")
            .or_else(|| std::env::var_os("HOME"))
            .map(tempfile::tempdir_in)
            .transpose()
            .expect("create selected bootstrap home")
            .unwrap_or_else(|| TempDir::new().expect("selected bootstrap home"));
        fs::set_permissions(selected.path(), fs::Permissions::from_mode(0o700))
            .expect("secure selected bootstrap home");
        let config_path = selected.path().join("config.yaml");
        let source = b"world: {enabled: true}\n";
        fs::write(&config_path, source).expect("write config");
        fs::set_permissions(&config_path, fs::Permissions::from_mode(0o600))
            .expect("secure selected config");
        let authority = HostSessionAuthority::open(selected.path()).expect("open authority");
        let opened = authority
            .bootstrap_home()
            .open_e3_config_source()
            .expect("open pinned source");
        let replacement = selected.path().join("replacement.yaml");
        fs::write(&replacement, source).expect("write byte-equal replacement");
        fs::set_permissions(&replacement, fs::Permissions::from_mode(0o600))
            .expect("secure replacement");
        fs::rename(&replacement, &config_path).expect("replace config inode");

        assert!(opened.revalidate().is_err());
    }

    struct EnvGuard {
        key: &'static str,
        prev: Option<OsString>,
        _authority_env: Option<crate::execution::AuthorityEnvTestGuard>,
    }

    impl EnvGuard {
        fn set(key: &'static str, value: &std::path::Path) -> Self {
            let authority_env = matches!(key, "SUBSTRATE_HOME" | "SUBSTRATE_WORLD_SOCKET")
                .then(crate::execution::AuthorityEnvTestGuard::preserve);
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            Self {
                key,
                prev,
                _authority_env: authority_env,
            }
        }

        fn set_str(key: &'static str, value: &str) -> Self {
            let authority_env = matches!(key, "SUBSTRATE_HOME" | "SUBSTRATE_WORLD_SOCKET")
                .then(crate::execution::AuthorityEnvTestGuard::preserve);
            let prev = std::env::var_os(key);
            std::env::set_var(key, value);
            Self {
                key,
                prev,
                _authority_env: authority_env,
            }
        }
    }

    impl Drop for EnvGuard {
        fn drop(&mut self) {
            match self.prev.take() {
                Some(v) => std::env::set_var(self.key, v),
                None => std::env::remove_var(self.key),
            }
        }
    }

    fn write_file(path: &Path, body: &str) {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(path, body).unwrap();
    }

    fn canonicalize_for_compare(path: &Path) -> PathBuf {
        path.canonicalize().unwrap_or_else(|_| path.to_path_buf())
    }

    fn resolve_world_disable_message(
        cwd: &Path,
        overrides: CliConfigOverrides,
    ) -> Option<&'static str> {
        let (effective, explain) =
            resolve_effective_config_with_explain(cwd, &overrides, true).unwrap();
        world_disable_attribution_message(
            effective.world.enabled,
            explain
                .as_ref()
                .and_then(|explain| explain.keys.get("world.enabled")),
        )
    }

    fn explain_key(layer: &str, path: Option<&str>) -> ConfigExplainKey {
        ConfigExplainKey {
            merge_strategy: "replace".to_string(),
            sources: vec![ConfigExplainSource {
                layer: layer.to_string(),
                path: path.map(|value| value.to_string()),
            }],
        }
    }

    fn assert_same_path(actual: Option<&String>, expected: &Path) {
        let actual = actual
            .map(PathBuf::from)
            .unwrap_or_else(|| panic!("expected path, got None"));
        let actual = canonicalize_for_compare(&actual);
        let expected = canonicalize_for_compare(expected);
        assert_eq!(actual, expected);
    }

    fn assert_disable_attribution(
        attribution: &DoctorDisableAttribution,
        expected_reason: &'static str,
        expected_layer: &'static str,
        expected_flag: Option<&'static str>,
        expected_env: Option<&'static str>,
        expected_path_display: Option<&'static str>,
    ) {
        assert_eq!(attribution.reason, expected_reason);
        assert_eq!(attribution.source.key, "world.enabled");
        assert_eq!(attribution.source.layer, expected_layer);
        assert!(!attribution.source.value_display);
        assert_eq!(attribution.source.flag, expected_flag);
        assert_eq!(attribution.source.env, expected_env);
        assert_eq!(attribution.source.path_display, expected_path_display);
    }

    #[test]
    #[serial]
    fn test_phase_a_concat_dedupe_and_replace_provenance() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a", "b"]
    inventory_mode: merged
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b", "c"]
    inventory_mode: workspace_only
"#,
        );

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a", "b", "c"]);
        assert_eq!(
            effective.world.deps.inventory_mode,
            WorldDepsInventoryMode::WorkspaceOnly
        );

        let explain = explain.unwrap();
        assert_eq!(explain.kind, "substrate.config.explain.v1");

        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.merge_strategy, "concat_dedupe_ordered_set");
        assert_eq!(enabled.sources.len(), 2);
        let expected_global_path = global_path.display().to_string();
        assert_eq!(enabled.sources[0].layer, "global_patch");
        assert_eq!(
            enabled.sources[0].path.as_deref(),
            Some(expected_global_path.as_str())
        );
        assert_eq!(enabled.sources[1].layer, "workspace_patch");
        assert_same_path(enabled.sources[1].path.as_ref(), &workspace_yaml);

        let inv = explain.keys.get("world.deps.inventory_mode").unwrap();
        assert_eq!(inv.merge_strategy, "replace");
        assert_eq!(inv.sources.len(), 1);
        assert_eq!(inv.sources[0].layer, "workspace_patch");
        assert_same_path(inv.sources[0].path.as_ref(), &workspace_yaml);
    }

    #[test]
    #[serial]
    fn test_phase_a_explicit_empty_list_counts_as_source() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a"]
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: []
"#,
        );

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a"]);

        let explain = explain.unwrap();
        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.sources.len(), 2);
        assert_eq!(enabled.sources[0].layer, "global_patch");
        assert_eq!(enabled.sources[1].layer, "workspace_patch");
    }

    #[test]
    #[serial]
    fn test_phase_a_workspace_disabled_ignores_workspace_patch() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a"]
    inventory_mode: merged
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b"]
    inventory_mode: workspace_only
"#,
        );
        let disabled_marker =
            crate::execution::workspace::workspace_disabled_marker_path(&workspace_root);
        write_file(&disabled_marker, "disabled\n");

        let (effective, explain) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        assert_eq!(effective.world.deps.enabled, vec!["a"]);
        assert_eq!(
            effective.world.deps.inventory_mode,
            WorldDepsInventoryMode::Merged
        );

        let explain = explain.unwrap();
        let enabled = explain.keys.get("world.deps.enabled").unwrap();
        assert_eq!(enabled.sources.len(), 1);
        assert_eq!(enabled.sources[0].layer, "global_patch");

        let inv = explain.keys.get("world.deps.inventory_mode").unwrap();
        assert_eq!(inv.sources.len(), 1);
        assert_eq!(inv.sources[0].layer, "global_patch");
        let expected_global_path = global_path.display().to_string();
        assert_eq!(
            inv.sources[0].path.as_deref(),
            Some(expected_global_path.as_str())
        );
    }

    #[test]
    #[serial]
    fn test_diagnostics_world_enabled_prefers_cli_over_workspace_and_env() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: false
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let enabled = resolve_effective_config(
            &workspace_root,
            &CliConfigOverrides {
                world_enabled: Some(true),
                ..Default::default()
            },
        )
        .unwrap()
        .world
        .enabled;
        assert!(enabled, "CLI override should win over workspace and env");

        let disabled = resolve_effective_config(
            &workspace_root,
            &CliConfigOverrides {
                world_enabled: Some(false),
                ..Default::default()
            },
        )
        .unwrap()
        .world
        .enabled;
        assert!(
            !disabled,
            "CLI disable override should win over workspace and env"
        );

        let resolved = resolve_effective_config(&workspace_root, &CliConfigOverrides::default())
            .unwrap()
            .world
            .enabled;
        assert!(
            resolved,
            "workspace config should win over SUBSTRATE_OVERRIDE_WORLD when a workspace is enabled"
        );
    }

    #[test]
    #[serial]
    fn test_diagnostics_world_enabled_uses_env_override_without_workspace() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let resolved = resolve_effective_config(&workspace_root, &CliConfigOverrides::default())
            .unwrap()
            .world
            .enabled;
        assert!(
            !resolved,
            "SUBSTRATE_OVERRIDE_WORLD should apply when no enabled workspace exists"
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_cli_flag() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");
        let message = resolve_world_disable_message(
            &workspace_root,
            CliConfigOverrides {
                world_enabled: Some(false),
                ..Default::default()
            },
        );
        assert_eq!(
            message,
            Some("world isolation disabled by CLI flag --no-world")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_override_env() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();
        let _env_guard = EnvGuard::set_str("SUBSTRATE_OVERRIDE_WORLD", "disabled");

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_workspace_patch() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: true
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: false
"#,
        );

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_maps_global_patch() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  enabled: false
"#,
        );

        let workspace_root = tmp.path().join("ws");
        fs::create_dir_all(&workspace_root).unwrap();

        let message = resolve_world_disable_message(&workspace_root, CliConfigOverrides::default());
        assert_eq!(
            message,
            Some("world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)")
        );
    }

    #[test]
    fn test_world_disable_attribution_maps_default_layer() {
        let message = world_disable_attribution_message(false, Some(&explain_key("default", None)));
        assert_eq!(
            message,
            Some("world isolation disabled by default config (world.enabled: false)")
        );
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_omits_enabled_cases() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  enabled: true
"#,
        );

        let message = resolve_world_disable_message(
            &workspace_root,
            CliConfigOverrides {
                world_enabled: Some(true),
                ..Default::default()
            },
        );
        assert_eq!(message, None);
    }

    #[test]
    fn test_world_disable_attribution_falls_back_when_provenance_missing() {
        assert_eq!(
            world_disable_attribution_message(false, None),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(
            world_disable_attribution_message(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![],
                }),
            ),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(
            world_disable_attribution_message(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: "not-a-real-layer".to_string(),
                        path: Some("/tmp/secret/path".to_string()),
                    }],
                })
            ),
            Some("world isolation disabled by effective config (source unknown)")
        );
        assert_eq!(
            world_disable_attribution_message(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![
                        ConfigExplainSource {
                            layer: "global_patch".to_string(),
                            path: Some("/tmp/secret/config.yaml".to_string()),
                        },
                        ConfigExplainSource {
                            layer: "workspace_patch".to_string(),
                            path: Some("/tmp/secret/workspace.yaml".to_string()),
                        },
                    ],
                })
            ),
            Some("world isolation disabled by effective config (source unknown)")
        );
    }

    #[test]
    fn test_world_disable_attribution_ignores_source_path() {
        let message = world_disable_attribution_message(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "global_patch".to_string(),
                    path: Some("/tmp/secret/substrate/config.yaml".to_string()),
                }],
            }),
        );
        let message = message.expect("expected attribution message");
        assert_eq!(
            message,
            "world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)"
        );
        assert!(!message.contains("/tmp/secret/substrate/config.yaml"));
    }

    #[test]
    #[serial]
    fn test_world_disable_attribution_builder_maps_sources() {
        let cases = [
            (
                "cli_flag",
                Some("world isolation disabled by CLI flag --no-world"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "cli_flag",
                    value_display: false,
                    flag: Some("--no-world"),
                    env: None,
                    path_display: None,
                },
            ),
            (
                "source_unknown",
                Some("world isolation disabled by effective config (source unknown)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "source_unknown",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: None,
                },
            ),
            (
                "override_env",
                Some("world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "override_env",
                    value_display: false,
                    flag: None,
                    env: Some("SUBSTRATE_OVERRIDE_WORLD"),
                    path_display: None,
                },
            ),
            (
                "workspace_patch",
                Some("world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "workspace_patch",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: Some("<workspace>/.substrate/workspace.yaml"),
                },
            ),
            (
                "global_patch",
                Some("world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "global_patch",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: Some("$SUBSTRATE_HOME/config.yaml"),
                },
            ),
            (
                "default",
                Some("world isolation disabled by default config (world.enabled: false)"),
                WorldDisableSource {
                    key: "world.enabled",
                    layer: "default",
                    value_display: false,
                    flag: None,
                    env: None,
                    path_display: None,
                },
            ),
        ];

        for (layer, expected_reason, expected_source) in cases {
            let attribution = world_disable_attribution(
                false,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: layer.to_string(),
                        path: None,
                    }],
                }),
            )
            .expect("expected attribution");
            assert_disable_attribution(
                &attribution,
                expected_reason.expect("expected reason"),
                expected_source.layer,
                expected_source.flag,
                expected_source.env,
                expected_source.path_display,
            );
        }
    }

    #[test]
    fn test_world_disable_attribution_builder_falls_back_to_source_unknown() {
        let attribution = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![],
            }),
        )
        .expect("expected attribution");
        assert_disable_attribution(
            &attribution,
            "world isolation disabled by effective config (source unknown)",
            "source_unknown",
            None,
            None,
            None,
        );
    }

    #[test]
    fn test_world_disable_attribution_builder_ignores_raw_source_path() {
        let workspace = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "workspace_patch".to_string(),
                    path: Some("/tmp/secret/workspace.yaml".to_string()),
                }],
            }),
        )
        .expect("expected attribution");
        assert_disable_attribution(
            &workspace,
            "world isolation disabled by workspace config <workspace>/.substrate/workspace.yaml (world.enabled: false)",
            "workspace_patch",
            None,
            None,
            Some("<workspace>/.substrate/workspace.yaml"),
        );
        let workspace_json = serde_json::to_string(&workspace.source).unwrap();
        assert!(workspace_json.contains("<workspace>/.substrate/workspace.yaml"));
        assert!(!workspace_json.contains("/tmp/secret/workspace.yaml"));

        let global = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "global_patch".to_string(),
                    path: Some("/tmp/secret/config.yaml".to_string()),
                }],
            }),
        )
        .expect("expected attribution");
        assert_disable_attribution(
            &global,
            "world isolation disabled by global config $SUBSTRATE_HOME/config.yaml (world.enabled: false)",
            "global_patch",
            None,
            None,
            Some("$SUBSTRATE_HOME/config.yaml"),
        );
        let global_json = serde_json::to_string(&global.source).unwrap();
        assert!(global_json.contains("$SUBSTRATE_HOME/config.yaml"));
        assert!(!global_json.contains("/tmp/secret/config.yaml"));
    }

    #[test]
    fn test_world_disable_attribution_builder_redacts_env_token_in_source() {
        let attribution = world_disable_attribution(
            false,
            Some(&ConfigExplainKey {
                merge_strategy: "replace".to_string(),
                sources: vec![ConfigExplainSource {
                    layer: "override_env".to_string(),
                    path: None,
                }],
            }),
        )
        .expect("expected attribution");

        assert_disable_attribution(
            &attribution,
            "world isolation disabled by env override SUBSTRATE_OVERRIDE_WORLD=disabled",
            "override_env",
            None,
            Some("SUBSTRATE_OVERRIDE_WORLD"),
            None,
        );

        let source_json = serde_json::to_string(&attribution.source).unwrap();
        assert!(source_json.contains("SUBSTRATE_OVERRIDE_WORLD"));
        assert!(!source_json.contains("disabled"));
    }

    #[test]
    fn test_world_disable_attribution_builder_omits_enabled_cases() {
        assert_eq!(world_disable_attribution(true, None), None);
        assert_eq!(
            world_disable_attribution(
                true,
                Some(&ConfigExplainKey {
                    merge_strategy: "replace".to_string(),
                    sources: vec![ConfigExplainSource {
                        layer: "cli_flag".to_string(),
                        path: None,
                    }],
                }),
            ),
            None
        );
    }

    #[test]
    fn test_enum_validation_rejects_invalid_values_without_mutation() {
        let before = SubstrateConfigPatch::default();

        let mut patch = before.clone();
        let updates = parse_updates(&["world.deps.inventory_mode=bogus".to_string()]).unwrap();
        let err = apply_updates_to_patch(&mut patch, &updates).unwrap_err();
        assert!(is_user_error(&err));
        assert_eq!(patch, before);

        let mut patch = before.clone();
        let updates = parse_updates(&["world.deps.builtins=bogus".to_string()]).unwrap();
        let err = apply_updates_to_patch(&mut patch, &updates).unwrap_err();
        assert!(is_user_error(&err));
        assert_eq!(patch, before);
    }

    #[test]
    #[serial]
    fn test_explain_json_bytes_are_deterministic_for_identical_inputs() {
        let _authority_env = crate::execution::AuthorityEnvTestGuard::preserve();
        let tmp = TempDir::new().unwrap();
        let substrate_home = tmp.path().join(".substrate");
        fs::create_dir_all(&substrate_home).unwrap();
        let _guard = EnvGuard::set("SUBSTRATE_HOME", &substrate_home);

        let global_path = global_config_path().unwrap();
        write_file(
            &global_path,
            r#"
world:
  deps:
    enabled: ["a", "b"]
"#,
        );

        let workspace_root = tmp.path().join("ws");
        let workspace_yaml = crate::execution::workspace::workspace_marker_path(&workspace_root);
        write_file(
            &workspace_yaml,
            r#"
world:
  deps:
    enabled: ["b", "c"]
"#,
        );

        let (_, explain_1) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();
        let (_, explain_2) = resolve_effective_config_with_explain(
            &workspace_root,
            &CliConfigOverrides::default(),
            true,
        )
        .unwrap();

        let bytes_1 = serde_json::to_vec_pretty(&explain_1.unwrap()).unwrap();
        let bytes_2 = serde_json::to_vec_pretty(&explain_2.unwrap()).unwrap();
        assert_eq!(bytes_1, bytes_2);
    }

    #[test]
    fn test_full_settings_rejects_protected_provenance() {
        let raw = "world: {anchor_path: &model exact-model}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: *model}}}\n";
        assert!(parse_config_yaml(Path::new("/caller-held/config.yaml"), raw).is_err());
    }

    #[test]
    fn test_full_settings_selection_errors_are_typed() {
        let path = Path::new("/caller-held/config.yaml");
        for raw in PROTECTED_SELECTION_CONFIG_FIXTURES_V1 {
            let error = parse_config_yaml(path, raw).unwrap_err();
            assert!(
                is_protected_selection_config_error_v1(&error),
                "{error:#}; {raw}"
            );
            assert!(is_protected_selection_config_error_v1(
                &error.context("outer writer")
            ));
        }
        for raw in [
            "world: {enabled: invalid}\n",
            "[{world: {enabled: invalid}}]\n",
            "agents: [{enabled: invalid}]\n",
            "agents: {defaults: [{execution: invalid}]}\n",
        ] {
            let error = parse_config_yaml(path, raw).unwrap_err();
            assert!(
                !is_protected_selection_config_error_v1(&error),
                "{error:#}; {raw}"
            );
        }
    }

    #[test]
    fn test_full_settings_selection_guard_preserves_generic_anchors() {
        let path = Path::new("/caller-held/config.yaml");
        for raw in [
            "world: {enabled: &flag false, caged: *flag}\n".to_string(),
            "world: {enabled: &flag false, caged: *flag}\nagents: {defaults: {selection: {harness: codex, provider: openai, connection_profile: 'built_in:codex-subscription', model: pinned-model}}}\n".to_string(),
            SELECTION_GLOBAL_PROFILE.replace("world: {caged: false}", "world: {enabled: &flag false, caged: *flag}"),
        ] {
            let cfg = parse_config_yaml(path, &raw).unwrap();
            assert!(!cfg.world.enabled && !cfg.world.caged);
            assert!(cfg.selection_sources.is_empty());
            assert!(cfg.selection_profiles.is_empty());
            assert!(cfg.selection_preference_source.is_none());
            if raw.contains("pinned-model") {
                assert_eq!(cfg.agents.defaults.selection.model, "pinned-model");
            }
        }
    }
}
