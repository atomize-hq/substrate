use super::config::{load_install_config, save_install_config, InstallConfig};
use crate::WorldEnableArgs;
use anyhow::{anyhow, bail, Context, Result};
use chrono::Utc;
use std::env;
use std::ffi::OsStr;
use std::fs::{self, File, OpenOptions};
use std::io::{self, BufRead, BufReader, Read, Write};
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
#[cfg(target_os = "linux")]
use std::time::Instant;
use substrate_common::paths as substrate_paths;

pub fn run_enable(args: &WorldEnableArgs) -> Result<()> {
    if cfg!(target_os = "windows") {
        bail!("substrate world enable is not yet supported on Windows");
    }

    let prefix = resolve_prefix(args.prefix.as_deref())?;
    let config_path = substrate_paths::config_file()?;
    let manager_env_path = resolve_manager_env_path()?;
    let mut corrupt_config = false;
    let mut config = match load_install_config(&config_path) {
        Ok(cfg) => cfg,
        Err(err) => {
            corrupt_config = true;
            println!(
                "substrate: warn: install metadata at {} is invalid ({err}); it will be replaced after provisioning.",
                config_path.display()
            );
            let mut cfg = InstallConfig::default();
            cfg.set_existed(false);
            cfg.set_world_enabled(false);
            cfg
        }
    };

    if config.world_enabled && config.exists() && !args.force && !args.dry_run {
        println!(
            "World backend already enabled (metadata at {}). Use --force to rerun provisioning.",
            config_path.display()
        );
        return Ok(());
    }

    if !config.exists() && !corrupt_config {
        println!(
            "substrate: info: no install metadata at {}; continuing and creating it after provisioning",
            config_path.display()
        );
    }

    let helper_override = env::var("SUBSTRATE_WORLD_ENABLE_SCRIPT")
        .ok()
        .map(PathBuf::from);
    let version_dir = if helper_override.is_some() {
        None
    } else {
        Some(resolve_version_dir(&prefix)?)
    };
    let script_path = locate_helper_script(&prefix, version_dir.as_deref(), helper_override)?;
    let log_path = next_log_path(&prefix)?;

    if args.dry_run {
        print_dry_run_plan(&script_path, args, &prefix, &log_path)?;
        println!(
            "Dry run only – no changes were made. Run 'substrate world doctor --json' after provisioning to verify connectivity."
        );
        return Ok(());
    }

    initialize_log_file(&log_path)?;
    append_log_line(&log_path, &format!("helper: {}", script_path.display()))?;
    let socket_override = resolve_world_socket_path();
    let wait_seconds = if socket_override.is_some() {
        args.timeout.min(5)
    } else {
        args.timeout
    };
    run_helper_script(
        &script_path,
        args,
        &prefix,
        &log_path,
        socket_override.as_deref(),
    )?;

    verify_world_health(
        &log_path,
        Duration::from_secs(wait_seconds),
        args.verbose,
        socket_override.as_deref(),
    )?;

    config.set_world_enabled(true);
    save_install_config(&config_path, &config)?;
    update_manager_env_exports(&manager_env_path, true)?;

    println!(
        "World provisioning complete. Metadata updated at {}.",
        config_path.display()
    );
    println!(
        "Provisioning log: {}\nManager env updated at {} with SUBSTRATE_WORLD exports.\nNext: run 'substrate world doctor --json' or start a new shell to use the world backend.",
        log_path.display(),
        manager_env_path.display()
    );

    Ok(())
}

fn resolve_prefix(explicit: Option<&Path>) -> Result<PathBuf> {
    if let Some(prefix) = explicit {
        return Ok(prefix.to_path_buf());
    }
    if let Ok(prefix) = env::var("SUBSTRATE_PREFIX") {
        return Ok(PathBuf::from(prefix));
    }
    substrate_paths::substrate_home()
        .context("failed to locate Substrate home (override via --prefix or SUBSTRATE_HOME)")
}

fn resolve_manager_env_path() -> Result<PathBuf> {
    if let Ok(path) = env::var("SUBSTRATE_MANAGER_ENV") {
        return Ok(PathBuf::from(path));
    }
    Ok(substrate_paths::substrate_home()?.join("manager_env.sh"))
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

fn locate_helper_script(
    prefix: &Path,
    version_dir: Option<&Path>,
    override_path: Option<PathBuf>,
) -> Result<PathBuf> {
    if let Some(path) = override_path {
        if path.exists() {
            return Ok(path);
        }
        bail!(
            "SUBSTRATE_WORLD_ENABLE_SCRIPT={} does not exist",
            path.display()
        );
    }

    let version_dir =
        version_dir.ok_or_else(|| anyhow!("missing version directory for helper discovery"))?;
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

fn next_log_path(prefix: &Path) -> Result<PathBuf> {
    let log_dir = prefix.join("logs");
    let stamp = Utc::now().format("%Y%m%d-%H%M%S");
    Ok(log_dir.join(format!("world-enable-{}.log", stamp)))
}

fn initialize_log_file(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    let mut file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(path)?;
    writeln!(
        file,
        "# Substrate world enable log\ntimestamp: {}",
        Utc::now().to_rfc3339()
    )?;
    Ok(())
}

fn print_dry_run_plan(
    script: &Path,
    args: &WorldEnableArgs,
    prefix: &Path,
    log_path: &Path,
) -> Result<()> {
    let mut command_line = vec![
        script.display().to_string(),
        "--prefix".to_string(),
        prefix.display().to_string(),
        "--profile".to_string(),
        args.profile.clone(),
    ];
    if args.verbose {
        command_line.push("--verbose".into());
    }
    if args.force {
        command_line.push("--force".into());
    }
    command_line.push("--dry-run".into());
    println!("Dry run: {}", command_line.join(" "));
    if let Some(parent) = log_path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("failed to create {}", parent.display()))?;
    }
    println!("Helper log would be written to {}", log_path.display());
    Ok(())
}

fn run_helper_script(
    script: &Path,
    args: &WorldEnableArgs,
    prefix: &Path,
    log_path: &Path,
    socket_override: Option<&Path>,
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
    if let Some(socket_path) = socket_override {
        cmd.env("SUBSTRATE_WORLD_SOCKET", socket_path);
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
            "world enable helper exited with status {}. See {} for details, then run 'substrate world doctor --json' for diagnostics.",
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
                if label == "stderr" {
                    eprint!("[{}] {}", label, line);
                } else {
                    print!("[{}] {}", label, line);
                }
            }
        }
        Ok(())
    })
}

fn verify_world_health(
    log_path: &Path,
    timeout: Duration,
    verbose: bool,
    socket_path: Option<&Path>,
) -> Result<()> {
    #[cfg(target_os = "linux")]
    {
        wait_for_socket(socket_path, timeout, log_path)?;
    }
    #[cfg(not(target_os = "linux"))]
    {
        let _ = (timeout, socket_path);
    }
    run_world_doctor(log_path, verbose)
}

#[cfg(target_os = "linux")]
fn wait_for_socket(
    socket_override: Option<&Path>,
    timeout: Duration,
    log_path: &Path,
) -> Result<()> {
    let socket_path = socket_override.unwrap_or_else(|| Path::new("/run/substrate.sock"));
    let deadline = Instant::now() + timeout;
    while Instant::now() <= deadline {
        if socket_path.exists() {
            append_log_line(
                log_path,
                &format!("socket: {} detected", socket_path.display()),
            )?;
            return Ok(());
        }
        thread::sleep(Duration::from_millis(500));
    }
    append_log_line(
        log_path,
        &format!("socket: timeout waiting for {}", socket_path.display()),
    )?;
    bail!(
        "Timed out waiting for {} after {} seconds. See {} for logs, then run 'substrate world doctor --json' to inspect the backend.",
        socket_path.display(),
        timeout.as_secs(),
        log_path.display()
    )
}

fn run_world_doctor(log_path: &Path, verbose: bool) -> Result<()> {
    if env::var("SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR")
        .map(|value| matches!(value.trim(), "1" | "true" | "TRUE"))
        .unwrap_or(false)
    {
        append_log_line(
            log_path,
            "skipping: substrate world doctor --json (SUBSTRATE_WORLD_ENABLE_SKIP_DOCTOR=1)",
        )?;
        return Ok(());
    }
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

fn resolve_world_socket_path() -> Option<PathBuf> {
    env::var_os("SUBSTRATE_WORLD_SOCKET")
        .map(PathBuf::from)
        .map(|path| normalize_path(&path))
}

fn update_manager_env_exports(path: &Path, enabled: bool) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "failed to create directory for manager env at {}",
                parent.display()
            )
        })?;
    }

    let existing = fs::read_to_string(path).unwrap_or_else(|_| String::new());
    let mut lines: Vec<String> = existing.lines().map(|line| line.to_string()).collect();
    let mut shebang = None;
    if let Some(first) = lines.first() {
        if first.starts_with("#!") {
            shebang = Some(lines.remove(0));
        }
    }
    lines.retain(|line| {
        let trimmed = line.trim_start();
        !trimmed.starts_with("export SUBSTRATE_WORLD=")
            && !trimmed.starts_with("export SUBSTRATE_WORLD_ENABLED=")
    });

    let mut output = Vec::new();
    if let Some(sb) = shebang {
        output.push(sb);
    }
    output.push("# Managed by `substrate world enable`".to_string());
    output.push(format!(
        "export SUBSTRATE_WORLD={}",
        if enabled { "enabled" } else { "disabled" }
    ));
    output.push(format!(
        "export SUBSTRATE_WORLD_ENABLED={}",
        if enabled { "1" } else { "0" }
    ));
    if !lines.is_empty() {
        output.push(String::new());
        output.extend(lines);
    }

    let mut body = output.join("\n");
    body.push('\n');
    fs::write(path, body)
        .with_context(|| format!("failed to update manager env at {}", path.display()))?;
    Ok(())
}

fn normalize_path(path: &Path) -> PathBuf {
    let mut prefix_component: Option<std::ffi::OsString> = None;
    let mut has_root = false;
    let mut parts: Vec<std::ffi::OsString> = Vec::new();

    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                if let Some(last) = parts.last() {
                    if last != OsStr::new("..") {
                        parts.pop();
                        continue;
                    }
                }
                if has_root || prefix_component.is_some() {
                    continue;
                }
                parts.push(std::ffi::OsString::from(".."));
            }
            Component::RootDir => {
                has_root = true;
                parts.clear();
            }
            Component::Prefix(prefix) => {
                prefix_component = Some(prefix.as_os_str().to_os_string());
                parts.clear();
            }
            Component::Normal(part) => parts.push(part.to_os_string()),
        }
    }

    let mut normalized = PathBuf::new();
    if let Some(prefix) = prefix_component {
        normalized.push(prefix);
    }
    if has_root {
        normalized.push(Path::new("/"));
    }
    for part in parts {
        normalized.push(part);
    }
    if normalized.as_os_str().is_empty() {
        normalized.push(".");
    }
    normalized
}
