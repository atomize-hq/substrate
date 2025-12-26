use crate::WorldVerifyArgs;
use anyhow::{Context, Result};
use chrono::Utc;
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const REPORT_SCHEMA_VERSION: u32 = 1;

const CHECK_ID_WORLD_BACKEND: &str = "world_backend";
const CHECK_ID_READONLY_REL: &str = "read_only_relative_write";
const CHECK_ID_READONLY_ABS: &str = "read_only_absolute_write";
const CHECK_ID_FULL_CAGE: &str = "full_cage_host_isolation";

const CHECK_DESC_WORLD_BACKEND: &str = "World backend is available (doctor ok=true)";
const CHECK_DESC_READONLY_REL: &str = "world_fs.mode=read_only blocks relative project writes";
const CHECK_DESC_READONLY_ABS: &str = "world_fs.mode=read_only blocks absolute-path project writes";
const CHECK_DESC_FULL_CAGE: &str =
    "world_fs.cage=full enforces allowlists and blocks host paths outside the project";

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum CheckStatus {
    Pass,
    Fail,
    Skip,
}

#[derive(Debug, Serialize)]
struct CheckReport {
    id: &'static str,
    description: &'static str,
    status: CheckStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    hint: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    artifacts: BTreeMap<String, String>,
}

#[derive(Debug, Serialize)]
struct VerifySummary {
    total: usize,
    passed: usize,
    failed: usize,
    skipped: usize,
    enforcement_checks_ran: usize,
    exit_code: i32,
}

#[derive(Debug, Serialize)]
struct VerifyReport {
    schema_version: u32,
    ok: bool,
    platform: String,
    started_at_utc: String,
    strict: bool,
    root: String,
    checks: Vec<CheckReport>,
    summary: VerifySummary,
}

#[derive(Debug)]
struct CommandRun {
    exit_code: i32,
    stdout_path: PathBuf,
    stderr_path: PathBuf,
}

pub fn run(args: &WorldVerifyArgs) -> Result<i32> {
    let started_at = Utc::now();
    let started_at_utc = started_at.to_rfc3339_opts(chrono::SecondsFormat::Secs, true);

    let root = prepare_root(args.root.as_ref(), started_at)?;
    let log_dir = root.join("logs");
    fs::create_dir_all(&log_dir).context("create verify log dir")?;

    let mut checks = Vec::new();

    if cfg!(windows) {
        checks.push(CheckReport {
            id: "platform_supported",
            description: "Platform supports world verify",
            status: CheckStatus::Skip,
            error: Some("world verify is not yet supported on Windows".to_string()),
            hint: Some(
                "Run `substrate world doctor --json` on Linux/macOS to verify isolation enforcement."
                    .to_string(),
            ),
            artifacts: BTreeMap::new(),
        });
        checks.extend(skipped_enforcement_checks(
            "world verify is not supported on Windows",
            Some("Run `substrate world doctor --json` on Linux/macOS to verify isolation enforcement.".to_string()),
        ));

        let report = finalize_report(
            args,
            &root,
            started_at_utc,
            checks,
            0,
            Some(if args.strict { 4 } else { 0 }),
        );
        emit_report(args, &report)?;
        cleanup_temp_projects(args, &root);
        return Ok(report.summary.exit_code);
    }

    let doctor = run_doctor(&root, &log_dir)?;
    checks.push(doctor.check);
    if !doctor.ok {
        checks.extend(skipped_enforcement_checks(
            "world backend unavailable; enforcement checks not run",
            Some("Run `substrate world doctor --json` and provision the backend (`substrate world enable`, or platform provisioning scripts).".to_string()),
        ));
        let report = finalize_report(
            args,
            &root,
            started_at_utc,
            checks,
            0,
            Some(doctor.exit_code),
        );
        emit_report(args, &report)?;
        cleanup_temp_projects(args, &root);
        return Ok(report.summary.exit_code);
    }

    let readonly_result = check_read_only(&root, &log_dir).context("read_only verification")?;
    checks.extend(readonly_result.checks);

    let fullcage_check = check_full_cage(&root, &log_dir).context("full cage verification")?;
    checks.push(fullcage_check);

    let enforcement_checks_ran = checks
        .iter()
        .filter(|check| is_enforcement_check(check.id) && check.status != CheckStatus::Skip)
        .count();
    let report = finalize_report(
        args,
        &root,
        started_at_utc,
        checks,
        enforcement_checks_ran,
        None,
    );

    emit_report(args, &report)?;
    cleanup_temp_projects(args, &root);
    Ok(report.summary.exit_code)
}

fn is_enforcement_check(id: &str) -> bool {
    matches!(
        id,
        CHECK_ID_READONLY_REL | CHECK_ID_READONLY_ABS | CHECK_ID_FULL_CAGE
    )
}

fn skipped_enforcement_checks(error: &str, hint: Option<String>) -> Vec<CheckReport> {
    let mut checks = Vec::new();
    for (id, description) in [
        (CHECK_ID_READONLY_REL, CHECK_DESC_READONLY_REL),
        (CHECK_ID_READONLY_ABS, CHECK_DESC_READONLY_ABS),
        (CHECK_ID_FULL_CAGE, CHECK_DESC_FULL_CAGE),
    ] {
        checks.push(CheckReport {
            id,
            description,
            status: CheckStatus::Skip,
            error: Some(error.to_string()),
            hint: hint.clone(),
            artifacts: BTreeMap::new(),
        });
    }
    checks
}

fn prepare_root(root_arg: Option<&PathBuf>, started_at: chrono::DateTime<Utc>) -> Result<PathBuf> {
    let base = root_arg
        .cloned()
        .unwrap_or_else(std::env::temp_dir)
        .join(format!(
            "substrate-world-verify-{}",
            started_at.format("%Y%m%d-%H%M%S")
        ));
    fs::create_dir_all(&base).context("create verify root")?;
    Ok(base)
}

fn cleanup_temp_projects(args: &WorldVerifyArgs, root: &Path) {
    if args.keep_temp {
        return;
    }
    for dir in ["readonly-project", "fullcage-project", "outside-host"] {
        let path = root.join(dir);
        let _ = fs::remove_dir_all(path);
    }
}

fn emit_report(args: &WorldVerifyArgs, report: &VerifyReport) -> Result<()> {
    if args.json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }

    println!("== substrate world verify ==");
    for check in &report.checks {
        let status = match check.status {
            CheckStatus::Pass => "PASS",
            CheckStatus::Fail => "FAIL",
            CheckStatus::Skip => "SKIP",
        };
        println!("{status:4} | {}: {}", check.id, check.description);
        if let Some(err) = &check.error {
            println!("      | error: {err}");
        }
        if let Some(hint) = &check.hint {
            println!("      | hint: {hint}");
        }
    }
    println!("Artifacts: {}", report.root);
    if report.ok {
        println!("PASS");
    } else {
        println!("FAIL");
    }
    Ok(())
}

fn finalize_report(
    args: &WorldVerifyArgs,
    root: &Path,
    started_at_utc: String,
    checks: Vec<CheckReport>,
    enforcement_checks_ran: usize,
    forced_exit_code: Option<i32>,
) -> VerifyReport {
    let passed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Pass)
        .count();
    let failed = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Fail)
        .count();
    let skipped = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Skip)
        .count();

    let computed_exit_code =
        if failed > 0 || (args.strict && skipped > 0) || enforcement_checks_ran == 0 {
            4
        } else {
            0
        };
    let exit_code = forced_exit_code.unwrap_or(computed_exit_code);

    let ok = exit_code == 0;
    VerifyReport {
        schema_version: REPORT_SCHEMA_VERSION,
        ok,
        platform: std::env::consts::OS.to_string(),
        started_at_utc,
        strict: args.strict,
        root: root.display().to_string(),
        checks,
        summary: VerifySummary {
            total: passed + failed + skipped,
            passed,
            failed,
            skipped,
            enforcement_checks_ran,
            exit_code,
        },
    }
}

struct DoctorResult {
    ok: bool,
    exit_code: i32,
    check: CheckReport,
}

fn run_doctor(root: &Path, log_dir: &Path) -> Result<DoctorResult> {
    let exe = std::env::current_exe().context("resolve current executable")?;
    let doctor_json_path = root.join("world-doctor.json");
    let doctor_err_path = log_dir.join("world-doctor.stderr");

    let output = Command::new(&exe)
        .args(["world", "doctor", "--json"])
        .output()
        .with_context(|| format!("run `{}` world doctor --json", exe.display()))?;

    fs::write(&doctor_json_path, &output.stdout).context("write world-doctor.json")?;
    fs::write(&doctor_err_path, &output.stderr).context("write world-doctor.stderr")?;

    let parsed = serde_json::from_slice::<Value>(&output.stdout);
    let (ok, parse_error, doctor_issue) = match parsed {
        Ok(value) => {
            let doctor_ok = value.get("ok").and_then(Value::as_bool).unwrap_or(false);
            let socket_exists = value
                .get("world_socket")
                .and_then(|socket| socket.get("socket_exists"))
                .and_then(Value::as_bool)
                .or_else(|| {
                    value
                        .get("agent_socket")
                        .and_then(|socket| socket.get("socket_exists"))
                        .and_then(Value::as_bool)
                })
                .unwrap_or(false);

            if !doctor_ok {
                (
                    false,
                    None,
                    Some("world doctor reported ok=false".to_string()),
                )
            } else if !socket_exists {
                (
                    false,
                    None,
                    Some("world socket missing or unreachable (socket_exists=false)".to_string()),
                )
            } else {
                (true, None, None)
            }
        }
        Err(err) => (false, Some(err.to_string()), None),
    };

    let mut artifacts = BTreeMap::new();
    artifacts.insert(
        "doctor_json".to_string(),
        doctor_json_path.display().to_string(),
    );
    artifacts.insert(
        "doctor_stderr".to_string(),
        doctor_err_path.display().to_string(),
    );

    let (exit_code, hint) = if parse_error.is_some() {
        (
            1,
            Some("Unexpected doctor output; re-run `substrate world doctor --json` and file a bug with the captured JSON/stderr.".to_string()),
        )
    } else if ok {
        (0, None)
    } else {
        (
            3,
            Some(
                "Run `substrate world doctor --json` and provision the backend (`substrate world enable`, or platform provisioning scripts).".to_string(),
            ),
        )
    };

    Ok(DoctorResult {
        ok,
        exit_code,
        check: CheckReport {
            id: CHECK_ID_WORLD_BACKEND,
            description: CHECK_DESC_WORLD_BACKEND,
            status: if ok {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            error: if ok {
                None
            } else if let Some(err) = parse_error {
                Some(format!(
                    "failed to parse world doctor JSON ({err}); see {}",
                    doctor_json_path.display()
                ))
            } else if let Some(issue) = doctor_issue {
                Some(format!("{issue} (see {})", doctor_json_path.display()))
            } else {
                Some(format!(
                    "world doctor reported ok=false (see {})",
                    doctor_json_path.display()
                ))
            },
            hint,
            artifacts,
        },
    })
}

struct ReadOnlyResult {
    checks: Vec<CheckReport>,
}

fn check_read_only(root: &Path, log_dir: &Path) -> Result<ReadOnlyResult> {
    let project_dir = root.join("readonly-project");
    fs::create_dir_all(&project_dir).context("create readonly project dir")?;
    write_profile(
        &project_dir.join(".substrate-profile"),
        r#"id: i6-verify-readonly
name: I6 verify (read_only)
world_fs:
  mode: read_only
  cage: project
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist: []
"#,
    )?;

    let rel_path = project_dir.join("rel.txt");
    let abs_path = project_dir.join("abs.txt");

    let exe = std::env::current_exe().context("resolve current executable")?;

    let rel_run = run_substrate_ci(
        &exe,
        &project_dir,
        r#"echo deny > rel.txt"#,
        &[],
        log_dir.join("readonly-rel.stdout"),
        log_dir.join("readonly-rel.stderr"),
    )?;

    let mut checks = Vec::new();
    checks.push(expected_failure_no_file(
        CHECK_ID_READONLY_REL,
        CHECK_DESC_READONLY_REL,
        &rel_run,
        &rel_path,
    ));

    let abs_path_env = abs_path.to_string_lossy().to_string();
    let abs_run = run_substrate_ci(
        &exe,
        &project_dir,
        r#"echo deny > "$VERIFY_ABS_WRITE""#,
        &[("VERIFY_ABS_WRITE", abs_path_env.as_str())],
        log_dir.join("readonly-abs.stdout"),
        log_dir.join("readonly-abs.stderr"),
    )?;

    checks.push(expected_failure_no_file(
        CHECK_ID_READONLY_ABS,
        CHECK_DESC_READONLY_ABS,
        &abs_run,
        &abs_path,
    ));

    Ok(ReadOnlyResult { checks })
}

fn check_full_cage(root: &Path, log_dir: &Path) -> Result<CheckReport> {
    let project_dir = root.join("fullcage-project");
    fs::create_dir_all(&project_dir).context("create full cage project dir")?;
    write_profile(
        &project_dir.join(".substrate-profile"),
        r#"id: i6-verify-full-cage
name: I6 verify (full cage)
world_fs:
  mode: writable
  cage: full
  require_world: true
  read_allowlist:
    - "*"
  write_allowlist:
    - "./writable/*"
"#,
    )?;

    let outside_dir = root.join("outside-host");
    fs::create_dir_all(&outside_dir).context("create outside host dir")?;
    let outside_read = outside_dir.join("outside_read.txt");
    let outside_write = outside_dir.join("outside_write.txt");
    fs::write(&outside_read, b"host-visible\n").context("write outside host read marker")?;

    let exe = std::env::current_exe().context("resolve current executable")?;
    let outside_read_env = outside_read.to_string_lossy().to_string();
    let outside_write_env = outside_write.to_string_lossy().to_string();
    let run = run_substrate_ci(
        &exe,
        &project_dir,
        r#"
set -eu

test -r /etc/passwd

mkdir -p writable
echo ok > writable/ok.txt
test -s writable/ok.txt

if echo nope > not-allowlisted.txt; then
  echo "unexpected: wrote to non-allowlisted project path" >&2
  exit 1
fi

if cat "$VERIFY_OUTSIDE_READ" >/dev/null 2>&1; then
  echo "unexpected: could read host path outside project ($VERIFY_OUTSIDE_READ)" >&2
  exit 1
fi

if echo deny > "$VERIFY_OUTSIDE_WRITE" 2>/dev/null; then
  echo "unexpected: could write host path outside project ($VERIFY_OUTSIDE_WRITE)" >&2
  exit 1
fi
"#,
        &[
            ("VERIFY_OUTSIDE_READ", outside_read_env.as_str()),
            ("VERIFY_OUTSIDE_WRITE", outside_write_env.as_str()),
        ],
        log_dir.join("fullcage.stdout"),
        log_dir.join("fullcage.stderr"),
    )?;

    let mut artifacts = BTreeMap::new();
    artifacts.insert("stdout".to_string(), run.stdout_path.display().to_string());
    artifacts.insert("stderr".to_string(), run.stderr_path.display().to_string());
    artifacts.insert("project_dir".to_string(), project_dir.display().to_string());

    let ok_txt = project_dir.join("writable/ok.txt");
    let disallowed_txt = project_dir.join("not-allowlisted.txt");

    let (status, error, hint) = if run.exit_code == 0 {
        if !ok_txt.is_file() {
            (
                CheckStatus::Fail,
                Some(format!(
                    "expected {} to exist on host after allowlisted write",
                    ok_txt.display()
                )),
                None,
            )
        } else if disallowed_txt.exists() {
            (
                CheckStatus::Fail,
                Some(format!(
                    "unexpected host file created at {} (non-allowlisted write should fail)",
                    disallowed_txt.display()
                )),
                None,
            )
        } else if outside_write.exists() {
            (
                CheckStatus::Fail,
                Some(format!(
                    "unexpected host file created at {} (outside host write should fail)",
                    outside_write.display()
                )),
                None,
            )
        } else {
            (CheckStatus::Pass, None, None)
        }
    } else {
        let stderr = fs::read_to_string(&run.stderr_path).unwrap_or_default();
        if stderr.contains("world_fs.cage=full requested but failed to enter a mount namespace")
            || stderr.contains("failed to spawn unshare wrapper")
        {
            (
                CheckStatus::Skip,
                Some("full cage requires mount namespaces; unshare failed".to_string()),
                Some(
                    "Run with CAP_SYS_ADMIN (root) or enable unprivileged user namespaces (kernel.unprivileged_userns_clone=1).".to_string(),
                ),
            )
        } else {
            (
                CheckStatus::Fail,
                Some(format!(
                    "full cage check failed (exit_code={}); see {}",
                    run.exit_code,
                    run.stderr_path.display()
                )),
                Some(
                    "Re-run with `substrate world doctor --json` and inspect the full cage logs."
                        .to_string(),
                ),
            )
        }
    };

    Ok(CheckReport {
        id: CHECK_ID_FULL_CAGE,
        description: CHECK_DESC_FULL_CAGE,
        status,
        error,
        hint,
        artifacts,
    })
}

fn expected_failure_no_file(
    id: &'static str,
    description: &'static str,
    run: &CommandRun,
    forbidden_path: &Path,
) -> CheckReport {
    let mut artifacts = BTreeMap::new();
    artifacts.insert("stdout".to_string(), run.stdout_path.display().to_string());
    artifacts.insert("stderr".to_string(), run.stderr_path.display().to_string());

    if run.exit_code == 0 {
        return CheckReport {
            id,
            description,
            status: CheckStatus::Fail,
            error: Some("expected command to fail but it succeeded".to_string()),
            hint: Some(
                "Run `substrate world doctor --json` and ensure the world backend is enabled."
                    .to_string(),
            ),
            artifacts,
        };
    }

    if forbidden_path.exists() {
        return CheckReport {
            id,
            description,
            status: CheckStatus::Fail,
            error: Some(format!(
                "unexpected host write at {} (file exists)",
                forbidden_path.display()
            )),
            hint: Some("Isolation enforcement did not prevent the write; inspect the logs and policy wiring.".to_string()),
            artifacts,
        };
    }

    CheckReport {
        id,
        description,
        status: CheckStatus::Pass,
        error: None,
        hint: None,
        artifacts,
    }
}

fn write_profile(path: &Path, contents: &str) -> Result<()> {
    fs::write(path, contents).with_context(|| format!("write profile {}", path.display()))
}

fn run_substrate_ci(
    exe: &Path,
    cwd: &Path,
    script: &str,
    extra_env: &[(&str, &str)],
    stdout_path: PathBuf,
    stderr_path: PathBuf,
) -> Result<CommandRun> {
    let mut cmd = Command::new(exe);
    cmd.arg("--world").arg("--ci").arg("-c").arg(script);
    cmd.current_dir(cwd);
    for (k, v) in extra_env {
        cmd.env(k, v);
    }
    let output = cmd.output().with_context(|| {
        format!(
            "run `{}` --ci -c <script> (cwd={})",
            exe.display(),
            cwd.display()
        )
    })?;

    fs::write(&stdout_path, &output.stdout).context("write verify stdout")?;
    fs::write(&stderr_path, &output.stderr).context("write verify stderr")?;

    Ok(CommandRun {
        exit_code: output.status.code().unwrap_or(1),
        stdout_path,
        stderr_path,
    })
}
