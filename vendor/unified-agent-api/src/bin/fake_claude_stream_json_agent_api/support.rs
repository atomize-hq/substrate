use std::{
    env,
    fs::{self, OpenOptions},
    io::{self, Write},
};

use serde_json::json;

pub(crate) const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
pub(crate) const ADD_DIR_RAW_PATH_SECRET: &str = "ADD_DIR_RAW_PATH_SECRET";
pub(crate) const ADD_DIR_STDOUT_SECRET: &str = "ADD_DIR_STDOUT_SECRET";
pub(crate) const ADD_DIR_STDERR_SECRET: &str = "ADD_DIR_STDERR_SECRET";

pub(crate) fn write_line(out: &mut (impl Write + ?Sized), line: &str) -> io::Result<()> {
    out.write_all(line.as_bytes())?;
    out.flush()?;
    Ok(())
}

pub(crate) fn write_assistant_text(out: &mut (impl Write + ?Sized), text: &str) -> io::Result<()> {
    let payload = json!({
        "type": "assistant",
        "session_id": "sess-1",
        "message": {
            "content": [{
                "type": "text",
                "text": text,
            }],
        },
    });
    write_line(out, &payload.to_string())?;
    write_line(out, "\n")?;
    Ok(())
}

pub(crate) fn write_result_error_with_message(
    out: &mut (impl Write + ?Sized),
    message: &str,
) -> io::Result<()> {
    let payload = json!({
        "type": "result",
        "subtype": "error",
        "session_id": "sess-1",
        "is_error": true,
        "message": message,
    });
    write_line(out, &payload.to_string())?;
    write_line(out, "\n")?;
    Ok(())
}

pub(crate) fn write_result_error_with_stderr_detail(
    out: &mut (impl Write + ?Sized),
    message: &str,
    stderr: &str,
) -> io::Result<()> {
    let payload = json!({
        "type": "result",
        "subtype": "error",
        "session_id": "sess-1",
        "is_error": true,
        "message": message,
        "details": {
            "stderr": stderr,
        },
    });
    write_line(out, &payload.to_string())?;
    write_line(out, "\n")?;
    Ok(())
}

pub(crate) fn write_add_dirs_runtime_rejection(out: &mut (impl Write + ?Sized)) -> io::Result<()> {
    let payload = json!({
        "type": "result",
        "subtype": "error",
        "session_id": "sess-1",
        "is_error": true,
        "message": ADD_DIRS_RUNTIME_REJECTION_MESSAGE,
        "details": {
            "raw_path": ADD_DIR_RAW_PATH_SECRET,
            "stdout": ADD_DIR_STDOUT_SECRET,
        },
    });
    write_line(out, &payload.to_string())?;
    write_line(out, "\n")?;
    Ok(())
}

pub(crate) fn exit_add_dirs_runtime_rejection(out: &mut (impl Write + ?Sized)) -> ! {
    write_add_dirs_runtime_rejection(out).expect("write add_dirs runtime rejection");
    eprintln!("{ADD_DIR_STDERR_SECRET}");
    std::process::exit(add_dirs_runtime_rejection_exit_code());
}

fn add_dirs_runtime_rejection_exit_code() -> i32 {
    match env::var("FAKE_CLAUDE_RUNTIME_REJECTION_EXIT_CODE") {
        Ok(raw) => raw
            .parse::<i32>()
            .unwrap_or_else(|err| panic!("invalid FAKE_CLAUDE_RUNTIME_REJECTION_EXIT_CODE: {err}")),
        Err(env::VarError::NotPresent) => 1,
        Err(err) => panic!("failed to read FAKE_CLAUDE_RUNTIME_REJECTION_EXIT_CODE: {err}"),
    }
}

pub(crate) fn has_flag_value(args: &[String], flag: &str, expected: &str) -> bool {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .is_some_and(|value| value == expected)
}

pub(crate) fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

pub(crate) fn contains_ordered_subsequence(args: &[String], subseq: &[&str]) -> bool {
    if subseq.is_empty() {
        return true;
    }

    let mut idx = 0usize;
    for arg in args {
        if arg == subseq[idx] {
            idx += 1;
            if idx == subseq.len() {
                return true;
            }
        }
    }
    false
}

pub(crate) fn require_env_var(key: &str) -> String {
    env::var(key).unwrap_or_else(|_| panic!("missing required env var {key}"))
}

pub(crate) fn expected_add_dirs() -> Option<Vec<String>> {
    let expected_count_raw = env::var("FAKE_CLAUDE_EXPECT_ADD_DIR_COUNT").ok()?;
    let expected_count = expected_count_raw
        .parse::<usize>()
        .unwrap_or_else(|err| panic!("invalid FAKE_CLAUDE_EXPECT_ADD_DIR_COUNT: {err}"));
    Some(
        (0..expected_count)
            .map(|index| require_env_var(&format!("FAKE_CLAUDE_EXPECT_ADD_DIR_{index}")))
            .collect(),
    )
}

pub(crate) fn assert_add_dirs(args: &[String], out: &mut dyn Write) {
    if env_is_true("FAKE_CLAUDE_EXPECT_NO_ADD_DIR") {
        if has_flag(args, "--add-dir") {
            fail(out, "assertion failed: expected --add-dir to be absent");
        }
        return;
    }

    let Some(expected) = expected_add_dirs() else {
        return;
    };

    let add_dir_indices: Vec<usize> = args
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| (arg == "--add-dir").then_some(idx))
        .collect();
    if add_dir_indices.len() != 1 {
        fail(
            out,
            &format!(
                "assertion failed: expected exactly one --add-dir flag, got {}",
                add_dir_indices.len()
            ),
        );
    }

    let Some(add_dir_idx) = add_dir_indices.into_iter().next() else {
        fail(out, "assertion failed: missing --add-dir");
    };

    let actual: Vec<String> = args
        .iter()
        .skip(add_dir_idx + 1)
        .take_while(|arg| !arg.starts_with("--"))
        .cloned()
        .collect();

    if actual.len() != expected.len() {
        fail(
            out,
            &format!(
                "assertion failed: expected {} add-dir values, got {}",
                expected.len(),
                actual.len()
            ),
        );
    }

    if actual != expected {
        fail(
            out,
            &format!(
                "assertion failed: expected add-dir values {:?}, got {:?}",
                expected, actual
            ),
        );
    }
}

pub(crate) fn maybe_assert_cwd(out: &mut dyn Write) {
    let Ok(expected_cwd) = env::var("FAKE_CLAUDE_EXPECT_CWD") else {
        return;
    };
    let got = env::current_dir().expect("read current dir");
    let expected = std::fs::canonicalize(&expected_cwd)
        .unwrap_or_else(|_| std::path::PathBuf::from(&expected_cwd));
    let got_canonical = std::fs::canonicalize(&got).unwrap_or(got);
    if got_canonical != expected {
        fail(
            out,
            &format!(
                "assertion failed: expected cwd {:?}, got {:?}",
                expected, got_canonical
            ),
        );
    }
}

pub(crate) fn maybe_assert_model_mapping(args: &[String], out: &mut dyn Write) {
    let expect_model = env::var("FAKE_CLAUDE_EXPECT_MODEL").ok();
    let expect_no_model = env_is_true("FAKE_CLAUDE_EXPECT_NO_MODEL");
    let expect_no_fallback_model = env_is_true("FAKE_CLAUDE_EXPECT_NO_FALLBACK_MODEL");

    if expect_model.is_some() && expect_no_model {
        panic!("FAKE_CLAUDE_EXPECT_MODEL and FAKE_CLAUDE_EXPECT_NO_MODEL are mutually exclusive");
    }

    if expect_no_fallback_model && has_flag(args, "--fallback-model") {
        fail(
            out,
            "assertion failed: expected --fallback-model to be absent",
        );
    }

    if expect_no_model {
        if has_flag(args, "--model") {
            fail(out, "assertion failed: expected --model to be absent");
        }
        return;
    }

    let Some(expected_model) = expect_model else {
        return;
    };

    let model_indices: Vec<usize> = args
        .iter()
        .enumerate()
        .filter_map(|(idx, arg)| (arg == "--model").then_some(idx))
        .collect();
    if model_indices.len() != 1 {
        fail(
            out,
            &format!(
                "assertion failed: expected exactly one --model flag, got {}",
                model_indices.len()
            ),
        );
    }
    let model_idx = model_indices[0];
    let actual = args.get(model_idx + 1).cloned().unwrap_or_default();
    if actual != expected_model {
        fail(
            out,
            &format!(
                "assertion failed: expected --model value {:?}, got {:?}",
                expected_model, actual
            ),
        );
    }

    if let Some(permission_idx) = args.iter().position(|arg| arg == "--permission-mode") {
        if model_idx > permission_idx {
            fail(
                out,
                "assertion failed: expected --model to precede --permission-mode",
            );
        }
    }

    for flag in [
        "--add-dir",
        "--continue",
        "--fork-session",
        "--resume",
        "--fallback-model",
        "--verbose",
    ] {
        if let Some(idx) = args.iter().position(|arg| arg == flag) {
            if model_idx > idx {
                fail(
                    out,
                    &format!("assertion failed: expected --model to precede {flag}"),
                );
            }
        }
    }
}

pub(crate) fn selector_assertion_subsequence(tail: &[&str]) -> Vec<String> {
    let mut subseq = vec![
        "--print".to_string(),
        "--output-format".to_string(),
        "stream-json".to_string(),
        "--permission-mode".to_string(),
        "bypassPermissions".to_string(),
    ];
    if let Some(add_dirs) = expected_add_dirs() {
        subseq.push("--add-dir".to_string());
        subseq.extend(add_dirs);
    }
    subseq.extend(tail.iter().map(|item| (*item).to_string()));
    subseq
}

pub(crate) fn require(env_key: &str) -> String {
    env::var(env_key).unwrap_or_else(|_| panic!("missing required env var {env_key}"))
}

pub(crate) fn fail(mut out: impl Write, message: &str) -> ! {
    let _ = write_assistant_text(&mut out, message);
    std::process::exit(2);
}

pub(crate) fn env_is_true(key: &str) -> bool {
    matches!(env::var(key).as_deref(), Ok("1") | Ok("true") | Ok("TRUE"))
}

pub(crate) fn maybe_log_invocation(kind: &str) {
    let Ok(path) = env::var("FAKE_CLAUDE_INVOCATION_LOG") else {
        return;
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open FAKE_CLAUDE_INVOCATION_LOG");
    writeln!(file, "{kind}").expect("append invocation log");
}

pub(crate) fn maybe_assert_flag_presence(
    args: &[String],
    env_key: &str,
    flag: &str,
    out: &mut dyn Write,
) {
    let Ok(raw) = env::var(env_key) else {
        return;
    };
    let expect_present = match raw.as_str() {
        "1" => true,
        "0" => false,
        other => panic!("{env_key} must be \"1\" or \"0\" (got {other:?})"),
    };

    let present = has_flag(args, flag);
    if expect_present != present {
        let expectation = if expect_present { "present" } else { "absent" };
        fail(
            out,
            &format!("assertion failed: expected {flag} to be {expectation}"),
        );
    }
}

pub(crate) fn maybe_write_env_snapshot() {
    let Ok(path) = env::var("FAKE_CLAUDE_ENV_SNAPSHOT_PATH") else {
        return;
    };

    let mut snapshot = String::new();
    for key in [
        "CLAUDE_HOME",
        "HOME",
        "XDG_CONFIG_HOME",
        "XDG_DATA_HOME",
        "XDG_CACHE_HOME",
    ] {
        let value = env::var(key).unwrap_or_default();
        snapshot.push_str(key);
        snapshot.push('=');
        snapshot.push_str(&value);
        snapshot.push('\n');
    }

    fs::write(path, snapshot).expect("write FAKE_CLAUDE_ENV_SNAPSHOT_PATH");
}
