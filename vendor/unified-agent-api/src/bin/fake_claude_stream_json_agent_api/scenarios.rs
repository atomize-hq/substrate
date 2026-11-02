use std::{
    env,
    io::{self, Write},
    thread,
    time::Duration,
};

use crate::fixtures::{
    first_nonempty_line, ASSISTANT_MESSAGE_TEXT, STREAM_EVENT_INPUT_JSON_DELTA,
    STREAM_EVENT_TOOL_RESULT_START, STREAM_EVENT_TOOL_USE_START, SYSTEM_INIT, USER_MESSAGE,
};
use crate::support::{
    assert_add_dirs, contains_ordered_subsequence, env_is_true, exit_add_dirs_runtime_rejection,
    fail, has_flag, has_flag_value, maybe_assert_cwd, maybe_assert_flag_presence,
    maybe_assert_model_mapping, maybe_log_invocation, maybe_write_env_snapshot, require,
    selector_assertion_subsequence, write_line, write_result_error_with_message,
    write_result_error_with_stderr_detail,
};

enum ScenarioKind {
    Assert {
        tail: Vec<String>,
        missing_subsequence_message: &'static str,
    },
    ModelRuntimeRejectionAfterInit {
        tail: Vec<String>,
        missing_subsequence_message: &'static str,
    },
    RuntimeRejection {
        tail: Vec<String>,
        missing_subsequence_message: &'static str,
    },
    DetailedResultError {
        tail: Vec<String>,
        missing_subsequence_message: &'static str,
        result_message: &'static str,
        detail_stderr: &'static str,
    },
    TerminalError {
        required_subsequence: Vec<String>,
        missing_subsequence_message: &'static str,
        result_message: &'static str,
    },
    StreamOnly(&'static str),
    Default,
}

pub(crate) fn run(args: Vec<String>) -> io::Result<()> {
    if has_flag(&args, "--help") {
        return handle_help();
    }

    let mut out = io::stdout().lock();
    if has_flag(&args, "--print") {
        prepare_print_invocation(&args, &mut out);
    }

    let scenario = env::var("FAKE_CLAUDE_SCENARIO").unwrap_or_else(|_| "two_events_delayed".into());
    dispatch_scenario(&scenario, &args, &mut out)
}

fn handle_help() -> io::Result<()> {
    maybe_log_invocation("help");

    if let Ok(delay_ms) = env::var("FAKE_CLAUDE_HELP_DELAY_MS") {
        let delay_ms = delay_ms
            .parse::<u64>()
            .unwrap_or_else(|_| panic!("invalid FAKE_CLAUDE_HELP_DELAY_MS={delay_ms}"));
        thread::sleep(Duration::from_millis(delay_ms));
    }

    if env_is_true("FAKE_CLAUDE_HELP_FAIL") {
        let secret = env::var("FAKE_CLAUDE_HELP_FAIL_SECRET")
            .unwrap_or_else(|_| panic!("missing required env var FAKE_CLAUDE_HELP_FAIL_SECRET"));
        eprintln!("{secret}");
        std::process::exit(1);
    }

    let supports_allow_flag = env_is_true("FAKE_CLAUDE_HELP_SUPPORTS_ALLOW_FLAG");
    println!("Usage: claude [options]");
    if supports_allow_flag {
        println!("  --allow-dangerously-skip-permissions");
    }
    Ok(())
}

fn prepare_print_invocation(args: &[String], out: &mut dyn Write) {
    maybe_log_invocation("print");

    if env_is_true("FAKE_CLAUDE_PRINT_SHOULD_NOT_RUN") {
        fail(out, "assertion failed: --print should not be spawned");
    }

    if !has_flag_value(args, "--permission-mode", "bypassPermissions") {
        fail(
            out,
            "assertion failed: missing --permission-mode bypassPermissions",
        );
    }

    maybe_assert_model_mapping(args, out);

    maybe_assert_flag_presence(
        args,
        "FAKE_CLAUDE_EXPECT_DANGEROUS_SKIP_PERMISSIONS",
        "--dangerously-skip-permissions",
        out,
    );
    maybe_assert_flag_presence(
        args,
        "FAKE_CLAUDE_EXPECT_ALLOW_DANGEROUSLY_SKIP_PERMISSIONS",
        "--allow-dangerously-skip-permissions",
        out,
    );
    maybe_assert_cwd(out);
    assert_add_dirs(args, out);
}

fn dispatch_scenario(scenario: &str, args: &[String], out: &mut dyn Write) -> io::Result<()> {
    let init = first_nonempty_line(SYSTEM_INIT);
    let user = first_nonempty_line(USER_MESSAGE);

    match scenario_kind(scenario) {
        ScenarioKind::Assert {
            tail,
            missing_subsequence_message,
        } => handle_assert(out, args, init, &tail, missing_subsequence_message),
        ScenarioKind::ModelRuntimeRejectionAfterInit {
            tail,
            missing_subsequence_message,
        } => handle_model_runtime_rejection(out, args, init, &tail, missing_subsequence_message),
        ScenarioKind::RuntimeRejection {
            tail,
            missing_subsequence_message,
        } => handle_runtime_rejection(out, args, init, &tail, missing_subsequence_message),
        ScenarioKind::DetailedResultError {
            tail,
            missing_subsequence_message,
            result_message,
            detail_stderr,
        } => handle_detailed_result_error(
            out,
            args,
            init,
            &tail,
            missing_subsequence_message,
            result_message,
            detail_stderr,
        ),
        ScenarioKind::TerminalError {
            required_subsequence,
            missing_subsequence_message,
            result_message,
        } => handle_terminal_error(
            out,
            args,
            init,
            &required_subsequence,
            missing_subsequence_message,
            result_message,
        ),
        ScenarioKind::StreamOnly(kind) => handle_stream_only(out, init, user, kind),
        ScenarioKind::Default => {
            write_line(out, &format!("{init}\n"))?;
            thread::sleep(Duration::from_millis(250));
            write_line(out, &format!("{user}\n"))?;
            Ok(())
        }
    }
}

fn scenario_kind(scenario: &str) -> ScenarioKind {
    match scenario {
        "fresh_assert" => ScenarioKind::Assert {
            tail: vec![
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fresh argv subsequence",
        },
        "model_runtime_rejection_after_init" => ScenarioKind::ModelRuntimeRejectionAfterInit {
            tail: vec![
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fresh argv subsequence",
        },
        "fork_last_assert" => ScenarioKind::Assert {
            tail: vec![
                "--continue".to_string(),
                "--fork-session".to_string(),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fork(last) argv subsequence",
        },
        "fork_id_assert" => ScenarioKind::Assert {
            tail: vec![
                "--fork-session".to_string(),
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fork(id) argv subsequence",
        },
        "resume_last_assert" => ScenarioKind::Assert {
            tail: vec![
                "--continue".to_string(),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing resume(last) argv subsequence",
        },
        "resume_id_assert" => ScenarioKind::Assert {
            tail: vec![
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing resume(id) argv subsequence",
        },
        "add_dirs_runtime_rejection_fresh" => ScenarioKind::RuntimeRejection {
            tail: vec![
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fresh argv subsequence",
        },
        "add_dirs_runtime_rejection_fork_last" => ScenarioKind::RuntimeRejection {
            tail: vec![
                "--continue".to_string(),
                "--fork-session".to_string(),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fork(last) argv subsequence",
        },
        "add_dirs_runtime_rejection_fork_id" => ScenarioKind::RuntimeRejection {
            tail: vec![
                "--fork-session".to_string(),
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fork(id) argv subsequence",
        },
        "add_dirs_runtime_rejection_resume_last" => ScenarioKind::RuntimeRejection {
            tail: vec![
                "--continue".to_string(),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing resume(last) argv subsequence",
        },
        "add_dirs_runtime_rejection_resume_id" => ScenarioKind::RuntimeRejection {
            tail: vec![
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing resume(id) argv subsequence",
        },
        "add_dirs_runtime_rejection_nested_detail_trap" => ScenarioKind::DetailedResultError {
            tail: vec![
                "--verbose".to_string(),
                require("FAKE_CLAUDE_EXPECT_PROMPT"),
            ],
            missing_subsequence_message: "assertion failed: missing fresh argv subsequence",
            result_message: "other failure",
            detail_stderr: "add_dirs rejected by runtime",
        },
        "add_dirs_runtime_rejection_resume_id_not_found_detail_trap" => {
            ScenarioKind::DetailedResultError {
                tail: vec![
                    "--resume".to_string(),
                    require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
                    "--verbose".to_string(),
                    require("FAKE_CLAUDE_EXPECT_PROMPT"),
                ],
                missing_subsequence_message:
                    "assertion failed: missing resume(id) argv subsequence",
                result_message: "add_dirs rejected by runtime",
                detail_stderr: "session not found",
            }
        }
        "fork_last_not_found" => ScenarioKind::TerminalError {
            required_subsequence: vec!["--continue".to_string(), "--fork-session".to_string()],
            missing_subsequence_message: "assertion failed: missing --continue --fork-session",
            result_message: "no session found",
        },
        "fork_id_not_found" => ScenarioKind::TerminalError {
            required_subsequence: vec![
                "--fork-session".to_string(),
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
            ],
            missing_subsequence_message: "assertion failed: missing --fork-session --resume <ID>",
            result_message: "session not found",
        },
        "fork_last_generic_error" => ScenarioKind::TerminalError {
            required_subsequence: vec!["--continue".to_string(), "--fork-session".to_string()],
            missing_subsequence_message: "assertion failed: missing --continue --fork-session",
            result_message: "permission denied",
        },
        "fork_id_generic_error" => ScenarioKind::TerminalError {
            required_subsequence: vec![
                "--fork-session".to_string(),
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
            ],
            missing_subsequence_message: "assertion failed: missing --fork-session --resume <ID>",
            result_message: "permission denied",
        },
        "resume_last_not_found" => ScenarioKind::TerminalError {
            required_subsequence: vec!["--continue".to_string()],
            missing_subsequence_message: "assertion failed: missing --continue",
            result_message: "no session found",
        },
        "resume_id_not_found" => ScenarioKind::TerminalError {
            required_subsequence: vec![
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
            ],
            missing_subsequence_message: "assertion failed: missing --resume <ID>",
            result_message: "session not found",
        },
        "resume_last_generic_error" => ScenarioKind::TerminalError {
            required_subsequence: vec!["--continue".to_string()],
            missing_subsequence_message: "assertion failed: missing --continue",
            result_message: "permission denied",
        },
        "resume_id_generic_error" => ScenarioKind::TerminalError {
            required_subsequence: vec![
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
            ],
            missing_subsequence_message: "assertion failed: missing --resume <ID>",
            result_message: "permission denied",
        },
        "resume_id_file_not_found_trap" => ScenarioKind::TerminalError {
            required_subsequence: vec![
                "--resume".to_string(),
                require("FAKE_CLAUDE_EXPECT_RESUME_ID"),
            ],
            missing_subsequence_message: "assertion failed: missing --resume <ID>",
            result_message: "file not found",
        },
        "block_until_killed" => ScenarioKind::StreamOnly("block_until_killed"),
        "single_event_then_exit" => ScenarioKind::StreamOnly("single_event_then_exit"),
        "two_events_long_delay" => ScenarioKind::StreamOnly("two_events_long_delay"),
        "final_text_and_tools" => ScenarioKind::StreamOnly("final_text_and_tools"),
        "claude_home_env_snapshot" => ScenarioKind::StreamOnly("claude_home_env_snapshot"),
        _ => ScenarioKind::Default,
    }
}

fn handle_assert(
    out: &mut dyn Write,
    args: &[String],
    init: &str,
    tail: &[String],
    missing_subsequence_message: &str,
) -> io::Result<()> {
    let expected_prompt = require("FAKE_CLAUDE_EXPECT_PROMPT");
    assert_prompt_and_verbose(args, out, &expected_prompt);

    let tail_refs: Vec<_> = tail.iter().map(String::as_str).collect();
    let subseq = selector_assertion_subsequence(&tail_refs);
    let subseq_refs: Vec<_> = subseq.iter().map(String::as_str).collect();
    if !contains_ordered_subsequence(args, &subseq_refs) {
        fail(out, missing_subsequence_message);
    }

    write_line(out, &format!("{init}\n"))?;
    Ok(())
}

fn handle_runtime_rejection(
    out: &mut dyn Write,
    args: &[String],
    init: &str,
    tail: &[String],
    missing_subsequence_message: &str,
) -> io::Result<()> {
    handle_assert(out, args, init, tail, missing_subsequence_message)?;
    exit_add_dirs_runtime_rejection(out);
}

fn handle_model_runtime_rejection(
    out: &mut dyn Write,
    args: &[String],
    init: &str,
    tail: &[String],
    missing_subsequence_message: &str,
) -> io::Result<()> {
    handle_assert(out, args, init, tail, missing_subsequence_message)?;

    let mut detail = require("FAKE_CLAUDE_MODEL_RUNTIME_REJECTION_SECRET");
    if let Ok(model) = env::var("FAKE_CLAUDE_EXPECT_MODEL") {
        if !model.is_empty() {
            detail.push(' ');
            detail.push_str(&model);
        }
    }

    write_result_error_with_stderr_detail(out, "model rejected by runtime", &detail)?;
    std::process::exit(1);
}

fn handle_detailed_result_error(
    out: &mut dyn Write,
    args: &[String],
    init: &str,
    tail: &[String],
    missing_subsequence_message: &str,
    result_message: &str,
    detail_stderr: &str,
) -> io::Result<()> {
    handle_assert(out, args, init, tail, missing_subsequence_message)?;
    write_result_error_with_stderr_detail(out, result_message, detail_stderr)?;
    std::process::exit(1);
}

fn handle_terminal_error(
    out: &mut dyn Write,
    args: &[String],
    init: &str,
    required_subsequence: &[String],
    missing_subsequence_message: &str,
    result_message: &'static str,
) -> io::Result<()> {
    let expected_prompt = require("FAKE_CLAUDE_EXPECT_PROMPT");
    assert_prompt_and_verbose(args, out, &expected_prompt);

    let required_refs: Vec<_> = required_subsequence.iter().map(String::as_str).collect();
    if !contains_ordered_subsequence(args, &required_refs) {
        fail(out, missing_subsequence_message);
    }

    write_line(out, &format!("{init}\n"))?;
    write_result_error_with_message(out, result_message)?;
    std::process::exit(1);
}

fn handle_stream_only(out: &mut dyn Write, init: &str, user: &str, kind: &str) -> io::Result<()> {
    match kind {
        "block_until_killed" => {
            write_line(out, &format!("{init}\n"))?;
            loop {
                thread::sleep(Duration::from_secs(1));
            }
        }
        "single_event_then_exit" => write_line(out, &format!("{init}\n")),
        "two_events_long_delay" => {
            write_line(out, &format!("{init}\n"))?;
            thread::sleep(Duration::from_millis(2000));
            write_line(out, &format!("{user}\n"))?;
            Ok(())
        }
        "final_text_and_tools" => {
            let tool_use_start = first_nonempty_line(STREAM_EVENT_TOOL_USE_START);
            let input_json_delta = first_nonempty_line(STREAM_EVENT_INPUT_JSON_DELTA);
            let tool_result_start = first_nonempty_line(STREAM_EVENT_TOOL_RESULT_START);
            let assistant_text = first_nonempty_line(ASSISTANT_MESSAGE_TEXT);

            write_line(out, &format!("{init}\n"))?;
            write_line(out, &format!("{tool_use_start}\n"))?;
            write_line(out, &format!("{input_json_delta}\n"))?;
            write_line(out, &format!("{tool_result_start}\n"))?;
            write_line(out, &format!("{assistant_text}\n"))?;
            Ok(())
        }
        "claude_home_env_snapshot" => {
            maybe_write_env_snapshot();
            let assistant_text = first_nonempty_line(ASSISTANT_MESSAGE_TEXT);

            write_line(out, &format!("{init}\n"))?;
            write_line(out, &format!("{assistant_text}\n"))?;
            Ok(())
        }
        other => panic!("unknown stream-only scenario: {other}"),
    }
}

fn assert_prompt_and_verbose(args: &[String], out: &mut dyn Write, expected_prompt: &str) {
    if args.last().map(String::as_str) != Some(expected_prompt) {
        fail(out, "assertion failed: prompt must be final argv token");
    }
    if !has_flag(args, "--verbose") {
        fail(out, "assertion failed: missing --verbose");
    }
}
