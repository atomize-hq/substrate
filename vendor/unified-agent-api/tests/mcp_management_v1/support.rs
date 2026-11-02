use std::{
    collections::BTreeMap,
    ffi::OsString,
    fs, io,
    path::{Path, PathBuf},
    sync::{Mutex, OnceLock},
};

use serde_json::Value;
use tempfile::{Builder, TempDir};

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct FakeInvocationRecord {
    pub(crate) args: Vec<String>,
    pub(crate) cwd: PathBuf,
    pub(crate) env: BTreeMap<String, String>,
}

#[derive(Debug)]
pub(crate) struct McpTestSandbox {
    root: TempDir,
    record_path: PathBuf,
    bin_dir: PathBuf,
    codex_home: PathBuf,
    claude_home: PathBuf,
}

impl McpTestSandbox {
    pub(crate) fn new(test_name: &str) -> io::Result<Self> {
        let prefix = format!("agent-api-mcp-{}-", sanitize_test_name(test_name));
        let root = Builder::new().prefix(&prefix).tempdir()?;

        let record_path = root.path().join("record.jsonl");
        let bin_dir = root.path().join("bin");
        let codex_home = root.path().join("codex-home");
        let claude_home = root.path().join("claude-home");

        fs::create_dir_all(&bin_dir)?;
        fs::create_dir_all(&codex_home)?;
        fs::create_dir_all(&claude_home)?;

        Ok(Self {
            root,
            record_path,
            bin_dir,
            codex_home,
            claude_home,
        })
    }

    pub(crate) fn root(&self) -> &Path {
        self.root.path()
    }

    pub(crate) fn record_path(&self) -> &Path {
        &self.record_path
    }

    pub(crate) fn bin_dir(&self) -> &Path {
        &self.bin_dir
    }

    pub(crate) fn codex_home(&self) -> &Path {
        &self.codex_home
    }

    pub(crate) fn claude_home(&self) -> &Path {
        &self.claude_home
    }

    pub(crate) fn install_fake_codex(&self) -> io::Result<PathBuf> {
        install_fake_binary(
            Path::new(env!("CARGO_BIN_EXE_fake_codex_mcp_agent_api")),
            &self.bin_dir.join(platform_binary_name("codex")),
        )
    }

    pub(crate) fn install_fake_claude(&self) -> io::Result<PathBuf> {
        install_fake_binary(
            Path::new(env!("CARGO_BIN_EXE_fake_claude_mcp_agent_api")),
            &self.bin_dir.join(platform_binary_name("claude")),
        )
    }

    pub(crate) fn read_records(&self) -> io::Result<Vec<FakeInvocationRecord>> {
        let text = fs::read_to_string(&self.record_path)?;
        parse_records(&text)
    }

    pub(crate) fn read_single_record(&self) -> io::Result<FakeInvocationRecord> {
        let mut records = self.read_records()?;
        match records.len() {
            1 => Ok(records.remove(0)),
            0 => Err(invalid_data(
                "expected exactly one invocation record, found none".into(),
            )),
            count => Err(invalid_data(format!(
                "expected exactly one invocation record, found {count}"
            ))),
        }
    }
}

pub(crate) fn process_env_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

pub(crate) struct EnvGuard {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvGuard {
    pub(crate) fn set(key: &'static str, value: impl Into<OsString>) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value.into());
        Self { key, previous }
    }

    pub(crate) fn unset(key: &'static str) -> Self {
        let previous = std::env::var_os(key);
        std::env::remove_var(key);
        Self { key, previous }
    }
}

impl Drop for EnvGuard {
    fn drop(&mut self) {
        if let Some(value) = self.previous.take() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

pub(crate) fn fake_mcp_sentinel(home_root: &Path, operation: &str) -> PathBuf {
    home_root
        .join(".agent_api_fake_mcp")
        .join(format!("{operation}.sentinel"))
}

pub(crate) fn collect_fake_mcp_sentinels(root: &Path) -> io::Result<Vec<PathBuf>> {
    let mut sentinels = Vec::new();
    collect_fake_mcp_sentinels_from(root, &mut sentinels)?;
    sentinels.sort();
    Ok(sentinels)
}

fn install_fake_binary(source: &Path, destination: &Path) -> io::Result<PathBuf> {
    if destination.exists() {
        return Ok(destination.to_path_buf());
    }

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent)?;
    }

    match fs::hard_link(source, destination) {
        Ok(()) => Ok(destination.to_path_buf()),
        Err(_) => {
            fs::copy(source, destination)?;
            Ok(destination.to_path_buf())
        }
    }
}

fn collect_fake_mcp_sentinels_from(root: &Path, sentinels: &mut Vec<PathBuf>) -> io::Result<()> {
    if !root.exists() {
        return Ok(());
    }

    for entry in fs::read_dir(root)? {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_fake_mcp_sentinels_from(&path, sentinels)?;
            continue;
        }

        if path
            .parent()
            .and_then(Path::file_name)
            .is_some_and(|name| name == ".agent_api_fake_mcp")
            && path.extension().is_some_and(|ext| ext == "sentinel")
        {
            sentinels.push(path);
        }
    }

    Ok(())
}

fn parse_records(text: &str) -> io::Result<Vec<FakeInvocationRecord>> {
    let mut records = Vec::new();

    for (line_idx, line) in text.lines().enumerate() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let value: Value = serde_json::from_str(trimmed).map_err(|err| {
            invalid_data(format!(
                "invalid fake MCP invocation record at line {}: {err}",
                line_idx + 1
            ))
        })?;
        records.push(parse_record_value(&value, line_idx + 1)?);
    }

    Ok(records)
}

fn parse_record_value(value: &Value, line_idx: usize) -> io::Result<FakeInvocationRecord> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} must be a JSON object")))?;

    let args = object
        .get("args")
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} is missing args")))?;
    let args = args_array(args, line_idx)?;

    let cwd = object
        .get("cwd")
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} is missing cwd")))?;
    let cwd = cwd
        .as_str()
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} has non-string cwd")))?;

    let env = object
        .get("env")
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} is missing env")))?;
    let env = env_map(env, line_idx)?;

    Ok(FakeInvocationRecord {
        args,
        cwd: PathBuf::from(cwd),
        env,
    })
}

fn args_array(value: &Value, line_idx: usize) -> io::Result<Vec<String>> {
    let items = value
        .as_array()
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} has non-array args")))?;

    items
        .iter()
        .enumerate()
        .map(|(idx, item)| {
            item.as_str().map(ToOwned::to_owned).ok_or_else(|| {
                invalid_data(format!(
                    "record at line {line_idx} has non-string args[{idx}]"
                ))
            })
        })
        .collect()
}

fn env_map(value: &Value, line_idx: usize) -> io::Result<BTreeMap<String, String>> {
    let object = value
        .as_object()
        .ok_or_else(|| invalid_data(format!("record at line {line_idx} has non-object env")))?;

    object
        .iter()
        .map(|(key, value)| {
            value
                .as_str()
                .map(|item| (key.clone(), item.to_owned()))
                .ok_or_else(|| {
                    invalid_data(format!(
                        "record at line {line_idx} has non-string env[{key}]"
                    ))
                })
        })
        .collect()
}

fn platform_binary_name(base: &str) -> String {
    if cfg!(windows) {
        format!("{base}.exe")
    } else {
        base.to_owned()
    }
}

fn sanitize_test_name(test_name: &str) -> String {
    let sanitized: String = test_name
        .chars()
        .map(|ch| {
            if ch.is_ascii_alphanumeric() {
                ch.to_ascii_lowercase()
            } else {
                '_'
            }
        })
        .collect();

    if sanitized.is_empty() {
        "mcp".to_string()
    } else {
        sanitized
    }
}

fn invalid_data(message: String) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
