use std::{
    env,
    io::{self, Read, Write},
    sync::mpsc,
    time::Duration,
};

pub(super) const ADD_DIRS_RUNTIME_REJECTION_MESSAGE: &str = "add_dirs rejected by runtime";
pub(super) const ADD_DIR_RAW_PATH_SECRET: &str = "ADD_DIR_RAW_PATH_SECRET";
pub(super) const ADD_DIR_STDOUT_SECRET: &str = "ADD_DIR_STDOUT_SECRET";
pub(super) const ADD_DIR_STDERR_SECRET: &str = "ADD_DIR_STDERR_SECRET";

pub(super) fn write_line(out: &mut impl Write, line: &str) -> io::Result<()> {
    out.write_all(line.as_bytes())?;
    out.flush()?;
    Ok(())
}

pub(super) fn emit_jsonl(out: &mut impl Write, line: &str) -> io::Result<()> {
    write_line(out, line)?;
    write_line(out, "\n")?;
    Ok(())
}

pub(super) fn has_flag(args: &[String], flag: &str) -> bool {
    args.iter().any(|arg| arg == flag)
}

pub(super) fn flag_value<'a>(args: &'a [String], flag: &str) -> Option<&'a str> {
    args.iter()
        .position(|arg| arg == flag)
        .and_then(|idx| args.get(idx + 1))
        .map(String::as_str)
}

pub(super) fn contains_ordered_subsequence(args: &[String], subseq: &[&str]) -> bool {
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

pub(super) fn require_eq(
    out: &mut impl Write,
    name: &str,
    got: Option<&str>,
    expected: Option<&str>,
) -> io::Result<bool> {
    if got == expected {
        return Ok(true);
    }
    let msg = format!("expected {name}={expected:?}, got {got:?}");
    write_line(out, &format!(r#"{{"type":"error","message":"{msg}"}}"#))?;
    write_line(out, "\n")?;
    Ok(false)
}

pub(super) fn require_flag_present(
    out: &mut impl Write,
    args: &[String],
    flag: &str,
) -> io::Result<bool> {
    if has_flag(args, flag) {
        return Ok(true);
    }
    emit_jsonl(
        out,
        &format!(r#"{{"type":"error","message":"missing required flag: {flag}"}}"#),
    )?;
    Ok(false)
}

pub(super) fn assert_env_overrides(out: &mut impl Write) -> io::Result<bool> {
    for (key, expected) in env::vars() {
        let Some(target) = key.strip_prefix("FAKE_CODEX_ASSERT_ENV_") else {
            continue;
        };
        let got = env::var(target).ok();
        if got.as_deref() != Some(expected.as_str()) {
            let msg = format!("expected env {target}={expected:?}, got {got:?}");
            emit_jsonl(out, &format!(r#"{{"type":"error","message":"{msg}"}}"#))?;
            return Ok(false);
        }
    }
    Ok(true)
}

pub(super) fn assert_current_dir(out: &mut impl Write) -> io::Result<bool> {
    let Ok(expected_cwd) = env::var("FAKE_CODEX_EXPECT_CWD") else {
        return Ok(true);
    };
    let got = env::current_dir()?;
    let expected = std::fs::canonicalize(&expected_cwd)
        .unwrap_or_else(|_| std::path::PathBuf::from(&expected_cwd));
    let got_canonical = std::fs::canonicalize(&got).unwrap_or(got.clone());
    if got_canonical == expected {
        return Ok(true);
    }

    let msg = format!(
        "expected cwd={}, got {}",
        expected.display(),
        got_canonical.display()
    );
    emit_jsonl(out, &format!(r#"{{"type":"error","message":"{msg}"}}"#))?;
    Ok(false)
}

pub(super) fn assert_add_dirs(out: &mut impl Write, args: &[String]) -> io::Result<bool> {
    let Ok(expected_count_raw) = env::var("FAKE_CODEX_EXPECT_ADD_DIR_COUNT") else {
        return Ok(true);
    };
    let expected_count = match expected_count_raw.parse::<usize>() {
        Ok(value) => value,
        Err(err) => {
            emit_jsonl(
                out,
                &format!(
                    r#"{{"type":"error","message":"invalid FAKE_CODEX_EXPECT_ADD_DIR_COUNT: {err}"}}"#
                ),
            )?;
            return Ok(false);
        }
    };

    let mut actual = Vec::new();
    let mut idx = 0usize;
    while idx < args.len() {
        if args[idx] == "--add-dir" {
            let Some(value) = args.get(idx + 1) else {
                emit_jsonl(
                    out,
                    r#"{"type":"error","message":"--add-dir missing required value"}"#,
                )?;
                return Ok(false);
            };
            actual.push(value.clone());
            idx += 2;
            continue;
        }
        idx += 1;
    }

    if actual.len() != expected_count {
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"error","message":"expected {expected_count} --add-dir values, got {}"}}"#,
                actual.len()
            ),
        )?;
        return Ok(false);
    }

    for (index, got) in actual.iter().enumerate() {
        let key = format!("FAKE_CODEX_EXPECT_ADD_DIR_{index}");
        let expected = require_env_var(out, &key)?;
        if got != &expected {
            emit_jsonl(
                out,
                &format!(
                    r#"{{"type":"error","message":"expected add-dir[{index}]={expected:?}, got {got:?}"}}"#
                ),
            )?;
            return Ok(false);
        }
    }

    Ok(true)
}

pub(super) fn assert_model(out: &mut impl Write, args: &[String]) -> io::Result<bool> {
    let Ok(expected_model) = env::var("FAKE_CODEX_EXPECT_MODEL") else {
        return Ok(true);
    };

    let positions: Vec<_> = args
        .iter()
        .enumerate()
        .filter_map(|(index, arg)| (arg == "--model").then_some(index))
        .collect();

    if expected_model == "<absent>" {
        if positions.is_empty() {
            return Ok(true);
        }
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"error","message":"did not expect --model flag, got {}"}}"#,
                positions.len()
            ),
        )?;
        return Ok(false);
    }

    if positions.len() != 1 {
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"error","message":"expected exactly one --model flag, got {}"}}"#,
                positions.len()
            ),
        )?;
        return Ok(false);
    }

    let model_index = positions[0];
    let Some(got_model) = args.get(model_index + 1) else {
        emit_jsonl(
            out,
            r#"{"type":"error","message":"--model missing required value"}"#,
        )?;
        return Ok(false);
    };

    if got_model != &expected_model {
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"error","message":"expected --model={expected_model:?}, got {got_model:?}"}}"#
            ),
        )?;
        return Ok(false);
    }

    if let Some(add_dir_index) = args.iter().position(|arg| arg == "--add-dir") {
        if model_index > add_dir_index {
            emit_jsonl(
                out,
                r#"{"type":"error","message":"expected --model before first --add-dir"}"#,
            )?;
            return Ok(false);
        }
    }

    Ok(true)
}

pub(super) fn require_env_var(out: &mut impl Write, key: &str) -> io::Result<String> {
    match env::var(key) {
        Ok(value) if !value.trim().is_empty() => Ok(value),
        _ => {
            emit_jsonl(
                out,
                &format!(r#"{{"type":"error","message":"missing required env var: {key}"}}"#),
            )?;
            std::process::exit(2);
        }
    }
}

fn read_stdin_to_end_with_timeout(timeout: Duration) -> io::Result<Vec<u8>> {
    let (tx, rx) = mpsc::channel::<io::Result<Vec<u8>>>();
    std::thread::spawn(move || {
        let mut buf = Vec::new();
        let result = io::stdin().read_to_end(&mut buf).map(|_| buf);
        let _ = tx.send(result);
    });

    match rx.recv_timeout(timeout) {
        Ok(result) => result,
        Err(mpsc::RecvTimeoutError::Timeout) => Err(io::Error::new(
            io::ErrorKind::TimedOut,
            "stdin read timed out (stdin may not have been closed)",
        )),
        Err(mpsc::RecvTimeoutError::Disconnected) => {
            Err(io::Error::other("stdin reader thread disconnected"))
        }
    }
}

pub(super) fn assert_stdin_prompt(
    out: &mut impl Write,
    expected_prompt: &str,
    timeout: Duration,
) -> io::Result<bool> {
    let stdin = match read_stdin_to_end_with_timeout(timeout) {
        Ok(stdin) => stdin,
        Err(err) => {
            emit_jsonl(
                out,
                &format!(r#"{{"type":"error","message":"failed to read stdin: {err}"}}"#),
            )?;
            return Ok(false);
        }
    };

    let mut expected = expected_prompt.as_bytes().to_vec();
    expected.push(b'\n');

    if stdin == expected {
        return Ok(true);
    }

    emit_jsonl(
        out,
        &format!(
            r#"{{"type":"error","message":"stdin prompt mismatch: expected {} bytes, got {} bytes"}}"#,
            expected.len(),
            stdin.len()
        ),
    )?;
    Ok(false)
}

pub(super) fn emit_add_dirs_runtime_rejection(out: &mut impl Write) -> io::Result<()> {
    emit_jsonl(
        out,
        &format!(
            r#"{{"type":"error","message":"{ADD_DIRS_RUNTIME_REJECTION_MESSAGE}","code":"add_dirs_runtime_rejection","details":{{"raw_path":"{ADD_DIR_RAW_PATH_SECRET}","stdout":"{ADD_DIR_STDOUT_SECRET}","stderr":"{ADD_DIR_STDERR_SECRET}"}}}}"#
        ),
    )?;
    {
        let mut err = io::stderr().lock();
        writeln!(err, "{ADD_DIR_STDERR_SECRET}")?;
        err.flush()?;
    }
    Ok(())
}

pub(super) fn runtime_rejection_exit_code() -> io::Result<i32> {
    match env::var("FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE") {
        Ok(raw) => raw.parse::<i32>().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE: {err}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(1),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to read FAKE_CODEX_RUNTIME_REJECTION_EXIT_CODE: {err}"),
        )),
    }
}

fn read_buffered_event_count() -> io::Result<usize> {
    match env::var("FAKE_CODEX_BUFFERED_EVENT_COUNT") {
        Ok(raw) => raw.parse::<usize>().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FAKE_CODEX_BUFFERED_EVENT_COUNT: {err}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(4096),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to read FAKE_CODEX_BUFFERED_EVENT_COUNT: {err}"),
        )),
    }
}

fn read_buffered_event_padding_bytes() -> io::Result<usize> {
    match env::var("FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES") {
        Ok(raw) => raw.parse::<usize>().map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES: {err}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(2048),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to read FAKE_CODEX_BUFFERED_EVENT_PADDING_BYTES: {err}"),
        )),
    }
}

fn read_buffered_event_sleep_every() -> io::Result<Option<usize>> {
    match env::var("FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY") {
        Ok(raw) => raw.parse::<usize>().map(Some).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY: {err}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to read FAKE_CODEX_BUFFERED_EVENT_SLEEP_EVERY: {err}"),
        )),
    }
}

fn read_buffered_event_sleep_millis() -> io::Result<Option<u64>> {
    match env::var("FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS") {
        Ok(raw) => raw.parse::<u64>().map(Some).map_err(|err| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                format!("invalid FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS: {err}"),
            )
        }),
        Err(env::VarError::NotPresent) => Ok(None),
        Err(err) => Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("failed to read FAKE_CODEX_BUFFERED_EVENT_SLEEP_MILLIS: {err}"),
        )),
    }
}

fn buffered_emit_sleep_config() -> io::Result<Option<(usize, Duration)>> {
    let Some(every) = read_buffered_event_sleep_every()? else {
        return Ok(None);
    };
    let delay = read_buffered_event_sleep_millis()?.unwrap_or(0);
    Ok(Some((every, Duration::from_millis(delay))))
}

fn maybe_sleep_during_buffered_emit(
    idx: usize,
    sleep_config: Option<(usize, Duration)>,
) -> io::Result<()> {
    let Some((every, delay)) = sleep_config else {
        return Ok(());
    };
    if every == 0 || delay.is_zero() || (idx + 1) % every != 0 {
        return Ok(());
    }

    std::thread::sleep(delay);
    Ok(())
}

pub(super) fn emit_buffered_turn_events(out: &mut impl Write, thread_id: &str) -> io::Result<()> {
    let count = read_buffered_event_count()?;
    let padding = "x".repeat(read_buffered_event_padding_bytes()?);
    let sleep_config = buffered_emit_sleep_config()?;

    for idx in 0..count {
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"turn.started","thread_id":"{thread_id}","turn_id":"buffered-turn-{idx}","padding":"{padding}"}}"#
            ),
        )?;
        maybe_sleep_during_buffered_emit(idx, sleep_config)?;
    }

    Ok(())
}

pub(super) fn emit_buffered_transport_errors(
    out: &mut impl Write,
    final_message: &str,
) -> io::Result<()> {
    let count = read_buffered_event_count()?;
    let padding = "x".repeat(read_buffered_event_padding_bytes()?);
    let sleep_config = buffered_emit_sleep_config()?;

    for idx in 0..count {
        emit_jsonl(
            out,
            &format!(
                r#"{{"type":"error","message":"transient transport failure {idx}","code":"transport_error","padding":"{padding}"}}"#
            ),
        )?;
        maybe_sleep_during_buffered_emit(idx, sleep_config)?;
    }

    emit_jsonl(
        out,
        &format!(r#"{{"type":"error","message":"{final_message}"}}"#),
    )?;

    Ok(())
}
