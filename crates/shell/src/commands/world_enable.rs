use crate::WorldEnableArgs;
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use serde_json::{Map, Value};
use std::env;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::{Duration, Instant};
use substrate_common::paths as substrate_paths;
use tempfile::NamedTempFile;

#[derive(Debug, Clone)]
pub struct InstallConfig {
    pub world_enabled: bool,
    existed: bool,
    extras: Map<String, Value>,
}

impl InstallConfig {
    pub fn exists(&self) -> bool {
        self.existed
    }

    pub fn set_world_enabled(&mut self, enabled: bool) {
        self.world_enabled = enabled;
    }
}

impl Default for InstallConfig {
    fn default() -> Self {
        Self {
            world_enabled: true,
            existed: false,
            extras: Map::new(),
        }
    }
}

pub fn load_install_config(path: &Path) -> Result<InstallConfig> {
    match fs::read_to_string(path) {
        Ok(contents) => parse_config(path, &contents),
        Err(err) if err.kind() == io::ErrorKind::NotFound => Ok(InstallConfig::default()),
        Err(err) => Err(anyhow!("failed to read {}: {err}", path.display())),
    }
}

fn parse_config(path: &Path, contents: &str) -> Result<InstallConfig> {
    let mut raw: Map<String, Value> = serde_json::from_str(contents)
        .with_context(|| format!("invalid JSON in {}", path.display()))?;
    let world_enabled = match raw.remove("world_enabled") {
        Some(Value::Bool(value)) => value,
        Some(other) => {
            bail!(
                "world_enabled in {} must be a boolean (found {other})",
                path.display()
            );
        }
        None => true,
    };
    Ok(InstallConfig {
        world_enabled,
        existed: true,
        extras: raw,
    })
}

pub fn save_install_config(path: &Path, cfg: &InstallConfig) -> Result<()> {
    let parent = path
        .parent()
        .ok_or_else(|| anyhow!("config path {} has no parent", path.display()))?;
    fs::create_dir_all(parent)
        .with_context(|| format!("failed to create directory for {}", path.display()))?;

    let mut data = cfg.extras.clone();
    data.insert("world_enabled".to_string(), Value::Bool(cfg.world_enabled));

    let mut tmp = NamedTempFile::new_in(parent)
        .with_context(|| format!("failed to create temp file near {}", path.display()))?;
    serde_json::to_writer_pretty(&mut tmp, &Value::Object(data))
        .with_context(|| format!("failed to serialize install config at {}", path.display()))?;
    tmp.flush()?;
    tmp.persist(path)
        .map_err(|e| anyhow!("failed to persist {}: {}", path.display(), e.error))?;
    Ok(())
}

pub fn run_enable(args: &WorldEnableArgs) -> Result<()> {
    if cfg!(target_os = "windows") {
        bail!("substrate world enable is not yet supported on Windows");
    }

    let prefix = resolve_prefix(args.prefix.as_deref())?;
    let config_path = prefix.join("config.json");
    let mut config = load_install_config(&config_path)?;

    if config.world_enabled && config.exists() && !args.force && !args.dry_run {
        println!(
            "World backend already enabled (metadata at {}). Use --force to rerun provisioning.",
            config_path.display()
        );
        return Ok(());
    }

    if !config.exists() {
        println!(
            "substrate: info: no install metadata at {}; continuing and creating it after provisioning",
            config_path.display()
        );
    }

    let version_dir = resolve_version_dir(&prefix)?;
    let script_path = locate_helper_script(&prefix, &version_dir)?;
    let log_path = prepare_log_file(&prefix)?;
    append_log_line(&log_path, &format!("helper: {}", script_path.display()))?;

    run_helper_script(&script_path, args, &prefix, &log_path)?;

    if args.dry_run {
        println!(
            "Dry run complete. Review provisioning log at {}",
            log_path.display()
        );
        return Ok(());
    }

    verify_world_health(&log_path, Duration::from_secs(args.timeout), args.verbose)?;

    config.set_world_enabled(true);
    save_install_config(&config_path, &config)?;

    println!(
        "World provisioning complete. Metadata updated at {}.",
        config_path.display()
    );
    println!(
        "Provisioning log: {}\nNext: run 'substrate world doctor --json' or start a new shell to use the world backend.",
        log_path.display()
    );

    Ok(())
}

fn resolve_prefix(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(prefix) = explicit {
        return Ok(prefix.to_path_buf());
    }
    substrate_paths::substrate_home()
        .context("failed to locate Substrate home (override via --prefix or SUBSTRATE_HOME)")
}

fn resolve_version_dir(prefix: &Path) -> Result<PathBuf> {
    let bin_name = if cfg!(target_os = "windows") {
        "substrate.exe"
    } else {
        "substrate"
    };
    let bin_path = prefix.join("bin").join(bin_name);
    if !bin_path.exists() {
        bail!(
            "Substrate binary not found at {}. Reinstall or pass --prefix to an existing install.",
            bin_path.display()
        );
    }
    let canonical = bin_path
        .canonicalize()
        .with_context(|| format!("failed to resolve {}", bin_path.display()))?;
    let bin_dir = canonical
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", canonical.display()))?;
    let version_dir = bin_dir
        .parent()
        .ok_or_else(|| anyhow!("{} has no parent directory", bin_dir.display()))?;
    Ok(version_dir.to_path_buf())
}

fn locate_helper_script(prefix: &Path, version_dir: &Path) -> Result<PathBuf> {
    if let Ok(override_path) = env::var("SUBSTRATE_WORLD_ENABLE_SCRIPT") {
        let path = PathBuf::from(&override_path);
        if path.exists() {
            return Ok(path);
        }
        bail!(
            "SUBSTRATE_WORLD_ENABLE_SCRIPT={} does not exist",
            override_path
        );
    }

    let candidates = [
        version_dir.join("scripts/substrate/world-enable.sh"),
        prefix.join("scripts/substrate/world-enable.sh"),
    ];

    for candidate in candidates {
        if candidate.exists() {
            return Ok(candidate);
        }
    }

    bail!(
        "world-enable helper script not found under {}. Reinstall Substrate to refresh scripts.",
        version_dir.display()
    )
}

fn prepare_log_file(prefix: &Path) -> Result<PathBuf> {
    let log_dir = prefix.join("logs");
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("failed to create {}", log_dir.display()))?;
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    let path = log_dir.join(format!("world-enable-{}.log", stamp));
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;
    writeln!(
        file,
        "# Substrate world enable log\ntimestamp: {}",
        Utc::now().to_rfc3339()
    )?;
    Ok(path)
}

fn run_helper_script(
    script: &Path,
    args: &WorldEnableArgs,
    prefix: &Path,
    log_path: &Path,
) -> Result<()> {
    append_log_line(
        log_path,
        &format!(
            "running helper {} (dry_run={}, verbose={}, force={})",
            script.display(),
            args.dry_run,
            args.verbose,
            args.force
        ),
    )?;

    let mut cmd = Command::new(script);
    cmd.arg("--prefix").arg(prefix);
    cmd.arg("--profile").arg(&args.profile);
    if args.dry_run {
        cmd.arg("--dry-run");
    }
    if args.verbose {
        cmd.arg("--verbose");
    }
    if args.force {
        cmd.arg("--force");
    }

    let mut child = cmd
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("failed to launch {}", script.display()))?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow!("helper stdout missing"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow!("helper stderr missing"))?;

    let log_file = OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    let log = Arc::new(Mutex::new(log_file));

    let threads = vec![
        stream_reader(stdout, "stdout", Arc::clone(&log), args.verbose),
        stream_reader(stderr, "stderr", Arc::clone(&log), args.verbose),
    ];

    let status = child
        .wait()
        .with_context(|| format!("failed to wait on {}", script.display()))?;
    for handle in threads {
        handle.join().unwrap()?;
    }

    if !status.success() {
        bail!(
            "world enable helper exited with status {}. See {} for details.",
            status,
            log_path.display()
        );
    }

    Ok(())
}

fn stream_reader<R>(
    reader: R,
    label: &'static str,
    log: Arc<Mutex<File>>,
    echo: bool,
) -> thread::JoinHandle<Result<()>>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut buf_reader = BufReader::new(reader);
        let mut line = String::new();
        loop {
            line.clear();
            let bytes = buf_reader.read_line(&mut line)?;
            if bytes == 0 {
                break;
            }
            {
                let mut file = log.lock().unwrap();
                write!(file, "[{}] {}", label, line)?;
                file.flush().ok();
            }
            if echo {
                print!("[{}] {}", label, line);
            }
        }
        Ok(())
    })
}

fn verify_world_health(log_path: &Path, timeout: Duration, verbose: bool) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        wait_for_socket(timeout, log_path)?;
    }
    run_world_doctor(log_path, verbose)
}

#[cfg(target_os = "linux")]
fn wait_for_socket(timeout: Duration, log_path: &Path) -> Result<()> {
    let socket_path = Path::new("/run/substrate.sock");
    let deadline = Instant::now() + timeout;
    while Instant::now() <= deadline {
        if socket_path.exists() {
            append_log_line(log_path, "socket: /run/substrate.sock detected")?;
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    append_log_line(log_path, "socket: timeout waiting for /run/substrate.sock")?;
    bail!(
        "Timed out waiting for /run/substrate.sock after {} seconds. See {} for logs.",
        timeout.as_secs(),
        log_path.display()
    )
}

#[cfg(not(target_os = "linux"))]
fn wait_for_socket(_timeout: Duration, _log_path: &Path) -> Result<()> {
    Ok(())
}

fn run_world_doctor(log_path: &Path, verbose: bool) -> Result<()> {
    append_log_line(log_path, "running: substrate world doctor --json")?;
    let exe = env::current_exe().context("failed to locate current executable")?;
    let output = Command::new(exe)
        .arg("world")
        .arg("doctor")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .context("failed to run 'substrate world doctor --json'")?;

    write_world_doctor_output(log_path, "stdout", &output.stdout, verbose)?;
    write_world_doctor_output(log_path, "stderr", &output.stderr, verbose)?;

    if !output.status.success() {
        bail!(
            "'substrate world doctor --json' failed (status {}). Review {} for details.",
            output.status,
            log_path.display()
        );
    }
    Ok(())
}

fn write_world_doctor_output(
    log_path: &Path,
    label: &str,
    data: &[u8],
    verbose: bool,
) -> Result<()> {
    if data.is_empty() {
        return Ok(());
    }
    let mut file = OpenOptions::new()
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    writeln!(file, "--- world doctor {} ---", label)?;
    file.write_all(data)?;
    if !data.ends_with(b"\n") {
        writeln!(file)?;
    }
    if verbose {
        io::stdout().write_all(data)?;
        if !data.ends_with(b"\n") {
            println!();
        }
    }
    Ok(())
}

fn append_log_line(log_path: &Path, message: &str) -> Result<()> {
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_path)
        .with_context(|| format!("failed to open {}", log_path.display()))?;
    writeln!(file, "{}", message)?;
    Ok(())
}
