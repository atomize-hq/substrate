use std::{
    collections::{BTreeMap, BTreeSet},
    env,
    ffi::OsString,
    fs::{self, OpenOptions},
    io::{self, Write},
    path::{Path, PathBuf},
    thread,
    time::Duration,
};

const RECORD_PATH_ENV: &str = "FAKE_CODEX_MCP_RECORD_PATH";
const RECORD_ENV_KEYS_ENV: &str = "FAKE_CODEX_MCP_RECORD_ENV_KEYS";
const SCENARIO_ENV: &str = "FAKE_CODEX_MCP_SCENARIO";
const CODEX_HOME_ENV: &str = "CODEX_HOME";
const SENTINEL_DIR: &str = ".agent_api_fake_mcp";
const OVERSIZED_OUTPUT_BYTES: usize = 65_536 + 128;
const SLEEP_FOR_TIMEOUT_MS: u64 = 500;
const FAST_EXIT_STDOUT_SENTINEL: &str = "fake_codex_mcp fast-exit stdout sentinel\n";
const FAST_EXIT_STDERR_SENTINEL: &str = "fake_codex_mcp fast-exit stderr sentinel\n";

// Test-only fake Codex MCP binary contract:
// - Required env: FAKE_CODEX_MCP_RECORD_PATH
// - Optional env: FAKE_CODEX_MCP_RECORD_ENV_KEYS (comma-separated), FAKE_CODEX_MCP_SCENARIO
// - Scenarios: ok, oversized_output, nonzero_exit, sleep_for_timeout, fast_exit_with_output,
//   drift, operation_subcommand_drift, legacy_add_drift, url_add_drift

fn main() -> io::Result<()> {
    let record_path = match required_path_env(RECORD_PATH_ENV) {
        Some(path) => path,
        None => std::process::exit(2),
    };

    let args = command_args();
    let record = build_record(&args)?;
    append_record(&record_path, &record)?;

    let scenario = scenario_name();
    maybe_write_sentinel(&args, &scenario)?;

    match scenario.as_str() {
        "ok" => Ok(()),
        "oversized_output" => {
            write_payload(
                &mut io::stdout().lock(),
                payload("codex-mcp-stdout:", OVERSIZED_OUTPUT_BYTES, b's'),
            )?;
            write_payload(
                &mut io::stderr().lock(),
                payload("codex-mcp-stderr:", OVERSIZED_OUTPUT_BYTES, b'e'),
            )?;
            Ok(())
        }
        "nonzero_exit" => {
            write_payload(&mut io::stdout().lock(), b"fake_codex_mcp nonzero stdout\n")?;
            write_payload(&mut io::stderr().lock(), b"fake_codex_mcp nonzero stderr\n")?;
            std::process::exit(7);
        }
        "sleep_for_timeout" => {
            write_payload(
                &mut io::stdout().lock(),
                b"fake_codex_mcp timeout stdout sentinel\n",
            )?;
            write_payload(
                &mut io::stderr().lock(),
                b"fake_codex_mcp timeout stderr sentinel\n",
            )?;
            thread::sleep(Duration::from_millis(SLEEP_FOR_TIMEOUT_MS));
            Ok(())
        }
        "fast_exit_with_output" => {
            write_payload(
                &mut io::stdout().lock(),
                FAST_EXIT_STDOUT_SENTINEL.as_bytes(),
            )?;
            write_payload(
                &mut io::stderr().lock(),
                FAST_EXIT_STDERR_SENTINEL.as_bytes(),
            )?;
            Ok(())
        }
        "drift" => {
            let message = if args.iter().any(|arg| arg == "--json") {
                "error: unexpected argument '--json' found\n"
            } else {
                "error: unknown subcommand 'mcp'\n"
            };
            write_payload(&mut io::stderr().lock(), message.as_bytes())?;
            std::process::exit(2);
        }
        "operation_subcommand_drift" => {
            let subcommand = invocation_subcommand(&args).unwrap_or("mcp");
            let message = format!("error: unknown subcommand '{subcommand}'\n");
            write_payload(&mut io::stderr().lock(), message.as_bytes())?;
            std::process::exit(2);
        }
        "legacy_add_drift" => {
            let message =
                "error: unexpected argument '--env' found\n\nusage: codex mcp add <name> --url <url>\n";
            write_payload(&mut io::stderr().lock(), message.as_bytes())?;
            std::process::exit(2);
        }
        "url_add_drift" => {
            let message = if args.iter().any(|arg| arg == "--bearer-token-env-var") {
                "error: unexpected argument '--bearer-token-env-var' found\n"
            } else {
                "error: unexpected argument '--url' found\n"
            };
            write_payload(&mut io::stderr().lock(), message.as_bytes())?;
            std::process::exit(2);
        }
        _ => Ok(()),
    }
}

fn command_args() -> Vec<String> {
    env::args_os()
        .skip(1)
        .map(lossy_os_string)
        .collect::<Vec<_>>()
}

fn build_record(args: &[String]) -> io::Result<String> {
    let cwd = env::current_dir()?.to_string_lossy().into_owned();
    let env = snapshot_env();
    Ok(format!(
        "{{\"args\":{},\"cwd\":{},\"env\":{}}}",
        serde_json::to_string(args).expect("args serialize"),
        serde_json::to_string(&cwd).expect("cwd serialize"),
        serde_json::to_string(&env).expect("env serialize")
    ))
}

fn snapshot_env() -> BTreeMap<String, String> {
    let mut keys = BTreeSet::new();
    keys.insert(CODEX_HOME_ENV.to_string());
    keys.extend(parse_allowlisted_env_keys());

    let mut snapshot = BTreeMap::new();
    for key in keys {
        if let Some(value) = env::var_os(&key) {
            snapshot.insert(key, lossy_os_string(value));
        }
    }
    snapshot
}

fn parse_allowlisted_env_keys() -> impl Iterator<Item = String> {
    env::var(RECORD_ENV_KEYS_ENV)
        .unwrap_or_default()
        .split(',')
        .map(str::trim)
        .filter(|item| !item.is_empty())
        .map(ToOwned::to_owned)
        .collect::<Vec<_>>()
        .into_iter()
}

fn append_record(path: &Path, line: &str) -> io::Result<()> {
    create_parent_dirs(path)?;
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    file.write_all(line.as_bytes())?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(())
}

fn maybe_write_sentinel(args: &[String], scenario: &str) -> io::Result<()> {
    if !matches!(scenario, "ok" | "oversized_output" | "nonzero_exit") {
        return Ok(());
    }

    let Some(operation) = invocation_subcommand(args) else {
        return Ok(());
    };
    if operation != "add" && operation != "remove" {
        return Ok(());
    }

    let Some(root) = nonempty_env_path(CODEX_HOME_ENV) else {
        return Ok(());
    };

    let sentinel_path = root
        .join(SENTINEL_DIR)
        .join(format!("{operation}.sentinel"));
    create_parent_dirs(&sentinel_path)?;
    fs::write(sentinel_path, b"1")?;
    Ok(())
}

fn invocation_subcommand(args: &[String]) -> Option<&str> {
    match args.first().map(String::as_str) {
        Some("mcp") => args.get(1).map(String::as_str),
        Some(other) => Some(other),
        None => None,
    }
}

fn scenario_name() -> String {
    match env::var(SCENARIO_ENV) {
        Ok(value) => match value.as_str() {
            "ok"
            | "oversized_output"
            | "nonzero_exit"
            | "sleep_for_timeout"
            | "fast_exit_with_output"
            | "drift"
            | "operation_subcommand_drift"
            | "legacy_add_drift"
            | "url_add_drift" => value,
            _ => "ok".to_string(),
        },
        Err(_) => "ok".to_string(),
    }
}

fn payload(prefix: &str, total_len: usize, fill: u8) -> Vec<u8> {
    let mut bytes = prefix.as_bytes().to_vec();
    bytes.resize(total_len, fill);
    bytes
}

fn write_payload(out: &mut impl Write, bytes: impl AsRef<[u8]>) -> io::Result<()> {
    out.write_all(bytes.as_ref())?;
    out.flush()?;
    Ok(())
}

fn required_path_env(key: &str) -> Option<PathBuf> {
    let value = env::var_os(key)?;
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

fn nonempty_env_path(key: &str) -> Option<PathBuf> {
    let value = env::var_os(key)?;
    if value.is_empty() {
        return None;
    }
    Some(PathBuf::from(value))
}

fn create_parent_dirs(path: &Path) -> io::Result<()> {
    let Some(parent) = path.parent() else {
        return Ok(());
    };
    if parent.as_os_str().is_empty() {
        return Ok(());
    }
    fs::create_dir_all(parent)
}

fn lossy_os_string(value: OsString) -> String {
    value.to_string_lossy().into_owned()
}
