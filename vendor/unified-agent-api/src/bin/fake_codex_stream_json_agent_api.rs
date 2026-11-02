use std::{
    collections::BTreeMap,
    env, fs,
    io::{self, Write},
    path::{Path, PathBuf},
    time::Duration,
};

const NEWLINE: &[u8] = b"\r\n";

fn write_bytes(out: &mut impl Write, bytes: &[u8]) -> io::Result<()> {
    out.write_all(bytes)?;
    out.flush()?;
    Ok(())
}

fn emit_jsonl(out: &mut impl Write, line: &str) -> io::Result<()> {
    write_bytes(out, line.as_bytes())?;
    write_bytes(out, NEWLINE)?;
    Ok(())
}

fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(String::as_str)
}

fn require_flag_present(out: &mut impl Write, args: &[String], flag: &str) -> io::Result<bool> {
    if has_flag(args, flag) {
        return Ok(true);
    }
    emit_jsonl(
        out,
        &format!(r#"{{"type":"error","message":"missing required flag: {flag}"}}"#),
    )?;
    Ok(false)
}

fn require_eq(
    out: &mut impl Write,
    name: &str,
    got: Option<&str>,
    expected: Option<&str>,
) -> io::Result<bool> {
    if got == expected {
        return Ok(true);
    }
    let msg = format!("expected {name}={expected:?}, got {got:?}");
    emit_jsonl(out, &format!(r#"{{"type":"error","message":"{msg}"}}"#))?;
    Ok(false)
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

fn dump_c2_env_to_path(path: &Path) -> io::Result<()> {
    create_parent_dirs(path)?;

    let mut vars = BTreeMap::<String, String>::new();
    for (key, value) in env::vars() {
        if key.starts_with("C2_") {
            vars.insert(key, value);
        }
    }

    let mut content = String::new();
    for (key, value) in vars {
        content.push_str(&key);
        content.push('=');
        content.push_str(&value);
        content.push('\n');
    }

    fs::write(path, content.as_bytes())
}

fn scenario() -> String {
    env::var("FAKE_CODEX_SCENARIO").unwrap_or_else(|_| "live_two_events_long_delay".to_string())
}

fn dump_env_path() -> io::Result<PathBuf> {
    let path = env::var("CODEX_WRAPPER_TEST_DUMP_ENV").map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "missing CODEX_WRAPPER_TEST_DUMP_ENV",
        )
    })?;
    if path.trim().is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "empty CODEX_WRAPPER_TEST_DUMP_ENV",
        ));
    }
    Ok(PathBuf::from(path))
}

fn main() -> io::Result<()> {
    // Cross-platform test binary used by `agent_api` tests.
    //
    // Emulates: `codex exec --json ...` by printing JSONL event lines.
    //
    // Scenario is selected via `FAKE_CODEX_SCENARIO` (normative C2 values):
    // - live_two_events_long_delay
    // - emit_normalize_error_with_rawline_secret
    // - dump_env_then_exit
    //
    // The wrapper validates argv deterministically via env vars so tests can assert that the
    // universal backend pins non-interactive behavior and sandbox mode.
    let args: Vec<String> = env::args().collect();
    let mut out = io::stdout().lock();

    if !args.iter().any(|arg| arg == "exec") {
        emit_jsonl(
            &mut out,
            r#"{"type":"error","message":"expected argv to include \"exec\""}"#,
        )?;
        std::process::exit(2);
    }

    if !require_flag_present(&mut out, &args, "--json")? {
        std::process::exit(1);
    }
    if !require_flag_present(&mut out, &args, "--skip-git-repo-check")? {
        std::process::exit(1);
    }

    let expected_sandbox =
        env::var("FAKE_CODEX_EXPECT_SANDBOX").unwrap_or_else(|_| "workspace-write".to_string());
    let expected_approval =
        env::var("FAKE_CODEX_EXPECT_APPROVAL").unwrap_or_else(|_| "never".to_string());

    let sandbox = flag_value(&args, "--sandbox");
    if !require_eq(
        &mut out,
        "--sandbox",
        sandbox,
        Some(expected_sandbox.as_str()),
    )? {
        std::process::exit(1);
    }

    if expected_approval == "<absent>" {
        if has_flag(&args, "--ask-for-approval") {
            emit_jsonl(
                &mut out,
                r#"{"type":"error","message":"did not expect --ask-for-approval"}"#,
            )?;
            std::process::exit(1);
        }
    } else {
        let approval = flag_value(&args, "--ask-for-approval");
        if !require_eq(
            &mut out,
            "--ask-for-approval",
            approval,
            Some(expected_approval.as_str()),
        )? {
            std::process::exit(1);
        }
    }

    match scenario().as_str() {
        "live_two_events_long_delay" => {
            emit_jsonl(
                &mut out,
                r#"{"type":"thread.started","thread_id":"thread-1"}"#,
            )?;
            emit_jsonl(
                &mut out,
                r#"{"type":"turn.started","thread_id":"thread-1","turn_id":"turn-1"}"#,
            )?;
            std::thread::sleep(Duration::from_millis(300));
        }
        "emit_normalize_error_with_rawline_secret" => {
            // 1) Parse error (invalid JSON) with secret in the raw line.
            write_bytes(&mut out, b"THIS IS NOT JSON RAWLINE_SECRET_DO_NOT_LEAK\r\n")?;
            // 2) Normalize error (valid JSON missing required context) with the same secret.
            emit_jsonl(
                &mut out,
                r#"{"type":"thread.started","secret":"RAWLINE_SECRET_DO_NOT_LEAK"}"#,
            )?;
            // 3) A valid event so downstream tests can observe continued streaming.
            emit_jsonl(
                &mut out,
                r#"{"type":"thread.started","thread_id":"thread-1"}"#,
            )?;
        }
        "dump_env_then_exit" => {
            let path = dump_env_path()?;
            dump_c2_env_to_path(&path)?;
            emit_jsonl(
                &mut out,
                r#"{"type":"thread.started","thread_id":"thread-1"}"#,
            )?;
        }
        other => {
            emit_jsonl(
                &mut out,
                &format!(r#"{{"type":"error","message":"unknown FAKE_CODEX_SCENARIO: {other}"}}"#),
            )?;
            std::process::exit(2);
        }
    }

    Ok(())
}
