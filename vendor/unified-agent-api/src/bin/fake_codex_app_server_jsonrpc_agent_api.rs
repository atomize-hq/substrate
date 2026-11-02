use std::{
    env,
    fs::OpenOptions,
    io::{self, BufRead, Write},
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
    time::Duration,
};

use serde_json::{json, Value};

fn write_json(out: &mut impl Write, value: &Value) -> io::Result<()> {
    writeln!(out, "{}", serde_json::to_string(value).unwrap_or_default())
}

fn write_result(out: &mut impl Write, id: &Value, result: Value) -> io::Result<()> {
    write_json(out, &json!({"jsonrpc":"2.0","id":id,"result":result}))
}

fn write_error(
    out: &mut impl Write,
    id: &Value,
    code: i64,
    message: &str,
    data: Option<Value>,
) -> io::Result<()> {
    let mut err = json!({"code": code, "message": message});
    if let Some(data) = data {
        err["data"] = data;
    }
    write_json(out, &json!({"jsonrpc":"2.0","id":id,"error":err}))
}

fn maybe_log_request_method(method: &str) {
    let Ok(path) = env::var("FAKE_CODEX_APP_SERVER_REQUEST_LOG") else {
        return;
    };

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .expect("open FAKE_CODEX_APP_SERVER_REQUEST_LOG");
    writeln!(file, "{method}").expect("append FAKE_CODEX_APP_SERVER_REQUEST_LOG");
}

fn main() -> io::Result<()> {
    // Cross-platform fake `codex app-server` JSON-RPC binary used by `agent_api` integration tests.
    //
    // Scenario is selected via `FAKE_CODEX_APP_SERVER_SCENARIO`. Each scenario asserts request
    // shapes and ordering, and fails loudly on drift.
    let args: Vec<String> = env::args().collect();
    if !args.iter().any(|arg| arg == "app-server") {
        eprintln!("expected argv to include \"app-server\"");
        std::process::exit(2);
    }

    let scenario = env::var("FAKE_CODEX_APP_SERVER_SCENARIO").unwrap_or_default();
    if scenario.trim().is_empty() {
        eprintln!("missing required env var: FAKE_CODEX_APP_SERVER_SCENARIO");
        std::process::exit(2);
    }

    let expect_prompt = env::var("FAKE_CODEX_APP_SERVER_EXPECT_PROMPT").ok();
    let expect_source_thread_id = env::var("FAKE_CODEX_APP_SERVER_EXPECT_SOURCE_THREAD_ID").ok();
    let expect_cwd = env::var("FAKE_CODEX_APP_SERVER_EXPECT_CWD").ok();
    let expect_thread_fork_sandbox =
        env::var("FAKE_CODEX_APP_SERVER_EXPECT_THREAD_FORK_SANDBOX").ok();
    let secret_sentinel = env::var("FAKE_CODEX_APP_SERVER_SECRET_SENTINEL").ok();

    let cancel_seen = Arc::new(AtomicBool::new(false));
    if matches!(
        scenario.as_str(),
        "approval_required_during_turn_start" | "block_until_cancel"
    ) {
        let cancel_seen = Arc::clone(&cancel_seen);
        thread::spawn(move || {
            thread::sleep(Duration::from_secs(4));
            if !cancel_seen.load(Ordering::SeqCst) {
                eprintln!("expected $/cancelRequest but never saw it");
                std::process::exit(2);
            }
        });
    }

    let stdin = io::stdin();
    let mut out = io::stdout().lock();

    let mut forked_thread_id: Option<String> = None;
    let mut in_flight_turn_start_id: Option<Value> = None;
    let mut thread_list_calls: usize = 0;

    for line in stdin.lock().lines() {
        let line = line?;
        if line.trim().is_empty() {
            continue;
        }
        let msg: Value = serde_json::from_str(&line)
            .unwrap_or_else(|err| panic!("failed to parse stdin as json: {err} (line={line:?})"));

        let method = msg
            .get("method")
            .and_then(Value::as_str)
            .unwrap_or_default()
            .to_string();
        maybe_log_request_method(&method);

        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);

        if id.is_none() {
            match method.as_str() {
                "$/cancelRequest" => {
                    let target = params.get("id").cloned().expect("cancel params.id");
                    if in_flight_turn_start_id.as_ref() != Some(&target) {
                        eprintln!("cancelRequest id mismatch: got {target:?}, expected {in_flight_turn_start_id:?}");
                        std::process::exit(2);
                    }

                    cancel_seen.store(true, Ordering::SeqCst);
                    let Some(turn_id) = in_flight_turn_start_id.take() else {
                        continue;
                    };
                    write_error(&mut out, &turn_id, -32800, "cancelled", None)?;
                }
                "exit" => break,
                _ => {}
            }
            continue;
        }

        let id = id.expect("id present");
        match method.as_str() {
            "initialize" => {
                let experimental = params
                    .get("capabilities")
                    .and_then(|caps| caps.get("experimentalApi"))
                    .and_then(Value::as_bool)
                    .unwrap_or(false);
                if !experimental {
                    eprintln!("expected initialize.params.capabilities.experimentalApi == true");
                    std::process::exit(2);
                }
                write_result(&mut out, &id, json!({"ready": true}))?;
            }
            "thread/list" => {
                if scenario != "fork_last_success_paged" && scenario != "fork_last_empty" {
                    eprintln!("thread/list called in unexpected scenario: {scenario}");
                    std::process::exit(2);
                }

                let expected_cwd = expect_cwd
                    .as_deref()
                    .unwrap_or_else(|| panic!("missing FAKE_CODEX_APP_SERVER_EXPECT_CWD"));
                let cwd = params
                    .get("cwd")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                if cwd != expected_cwd {
                    eprintln!("thread/list cwd mismatch: got {cwd:?} expected {expected_cwd:?}");
                    std::process::exit(2);
                }

                let sort_key = params.get("sortKey").and_then(Value::as_str);
                if sort_key != Some("updated_at") {
                    eprintln!("thread/list sortKey mismatch: got {sort_key:?}");
                    std::process::exit(2);
                }

                let limit = params.get("limit").and_then(Value::as_i64);
                if limit != Some(100) {
                    eprintln!("thread/list limit mismatch: got {limit:?}");
                    std::process::exit(2);
                }

                thread_list_calls += 1;
                if scenario == "fork_last_empty" {
                    let cursor = params.get("cursor").cloned().unwrap_or(Value::Null);
                    if !cursor.is_null() {
                        eprintln!("thread/list expected cursor null on first call, got {cursor:?}");
                        std::process::exit(2);
                    }
                    write_result(
                        &mut out,
                        &id,
                        json!({"data": [], "nextCursor": Value::Null}),
                    )?;
                    continue;
                }

                if thread_list_calls == 1 {
                    let cursor = params.get("cursor").cloned().unwrap_or(Value::Null);
                    if !cursor.is_null() {
                        eprintln!("thread/list expected cursor null on first call, got {cursor:?}");
                        std::process::exit(2);
                    }
                    write_result(
                        &mut out,
                        &id,
                        json!({
                            "data": [
                                {"id":"t-0","createdAt":0,"updatedAt":0,"cwd": expected_cwd},
                                {"id":"t-1","createdAt":5,"updatedAt":9,"cwd": expected_cwd}
                            ],
                            "nextCursor": "cursor-1"
                        }),
                    )?;
                } else if thread_list_calls == 2 {
                    let cursor = params.get("cursor").and_then(Value::as_str);
                    if cursor != Some("cursor-1") {
                        eprintln!("thread/list expected cursor \"cursor-1\" on second call, got {cursor:?}");
                        std::process::exit(2);
                    }
                    write_result(
                        &mut out,
                        &id,
                        json!({
                            "data": [
                                {"id":"t-a","createdAt":1,"updatedAt":10,"cwd": expected_cwd},
                                {"id":"t-b","createdAt":1,"updatedAt":10,"cwd": expected_cwd}
                            ],
                            "nextCursor": Value::Null
                        }),
                    )?;
                } else {
                    eprintln!("thread/list called more than twice");
                    std::process::exit(2);
                }
            }
            "thread/fork" => {
                let thread_id = params
                    .get("threadId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let approval_policy = params.get("approvalPolicy").and_then(Value::as_str);
                if approval_policy != Some("never") {
                    eprintln!("thread/fork approvalPolicy mismatch: got {approval_policy:?}");
                    std::process::exit(2);
                }

                if let Some(expected) = expect_thread_fork_sandbox.as_deref() {
                    let sandbox = params.get("sandbox").and_then(Value::as_str);
                    if sandbox != Some(expected) {
                        eprintln!(
                            "thread/fork sandbox mismatch: got {sandbox:?} expected {expected:?}"
                        );
                        std::process::exit(2);
                    }
                }

                match scenario.as_str() {
                    "fork_id_success"
                    | "fork_id_success_oversize_thread_id"
                    | "fork_id_success_thread_id_len_1024"
                    | "approval_required_during_turn_start"
                    | "block_until_cancel" => {
                        let expected = expect_source_thread_id.as_deref().unwrap_or_else(|| {
                            panic!("missing FAKE_CODEX_APP_SERVER_EXPECT_SOURCE_THREAD_ID")
                        });
                        if thread_id != expected {
                            eprintln!("thread/fork threadId mismatch: got {thread_id:?} expected {expected:?}");
                            std::process::exit(2);
                        }
                    }
                    "fork_last_success_paged" => {
                        if thread_id != "t-b" {
                            eprintln!(
                                "thread/fork expected selected threadId \"t-b\", got {thread_id:?}"
                            );
                            std::process::exit(2);
                        }
                    }
                    "fork_id_not_found" => {
                        let mut data = json!({"note": "not found"});
                        if let Some(secret) = secret_sentinel.as_deref() {
                            data["secret"] = Value::String(secret.to_string());
                        }
                        write_error(&mut out, &id, -32000, "thread not found", Some(data))?;
                        continue;
                    }
                    other => {
                        eprintln!("thread/fork called in unexpected scenario: {other}");
                        std::process::exit(2);
                    }
                }

                let new_thread_id = match scenario.as_str() {
                    "fork_id_success_oversize_thread_id" => "a".repeat(1025),
                    "fork_id_success_thread_id_len_1024" => "a".repeat(1024),
                    _ => "forked-1".to_string(),
                };
                forked_thread_id = Some(new_thread_id.clone());
                write_result(&mut out, &id, json!({"thread": {"id": new_thread_id}}))?;
            }
            "turn/start" => {
                let expected_prompt = expect_prompt
                    .as_deref()
                    .unwrap_or_else(|| panic!("missing FAKE_CODEX_APP_SERVER_EXPECT_PROMPT"));

                let thread_id = params
                    .get("threadId")
                    .and_then(Value::as_str)
                    .unwrap_or_default();
                let expected_thread = forked_thread_id.as_deref().unwrap_or("forked-1");
                if thread_id != expected_thread {
                    eprintln!("turn/start threadId mismatch: got {thread_id:?} expected {expected_thread:?}");
                    std::process::exit(2);
                }

                let approval_policy = params.get("approvalPolicy").and_then(Value::as_str);
                if approval_policy != Some("never") {
                    eprintln!("turn/start approvalPolicy mismatch: got {approval_policy:?}");
                    std::process::exit(2);
                }

                if expect_thread_fork_sandbox.is_some() && params.get("sandbox").is_some() {
                    eprintln!("turn/start unexpected sandbox field");
                    std::process::exit(2);
                }

                let input = params
                    .get("input")
                    .and_then(Value::as_array)
                    .map(Vec::as_slice)
                    .unwrap_or(&[]);
                if input.len() != 1 {
                    eprintln!("turn/start expected input length 1, got {}", input.len());
                    std::process::exit(2);
                }
                let first = &input[0];
                let input_type = first.get("type").and_then(Value::as_str);
                let input_text = first.get("text").and_then(Value::as_str);
                let text_elements_ok = first
                    .get("text_elements")
                    .and_then(Value::as_array)
                    .is_some_and(|arr| arr.is_empty());
                if input_type != Some("text")
                    || input_text != Some(expected_prompt)
                    || !text_elements_ok
                {
                    eprintln!("turn/start input mapping mismatch");
                    std::process::exit(2);
                }

                match scenario.as_str() {
                    "fork_id_success"
                    | "fork_id_success_oversize_thread_id"
                    | "fork_id_success_thread_id_len_1024"
                    | "fork_last_success_paged" => {
                        write_json(
                            &mut out,
                            &json!({"jsonrpc":"2.0","method":"turn/started","params": {}}),
                        )?;
                        write_result(&mut out, &id, json!({"turnId":"turn-1"}))?;
                    }
                    "approval_required_during_turn_start" => {
                        in_flight_turn_start_id = Some(id.clone());
                        write_json(
                            &mut out,
                            &json!({"jsonrpc":"2.0","method":"codex/event","params":{"type":"approval_required","approval_id":"ap-1","kind":"exec"}}),
                        )?;
                        // Response is sent when we receive $/cancelRequest.
                    }
                    "block_until_cancel" => {
                        in_flight_turn_start_id = Some(id.clone());
                        write_json(
                            &mut out,
                            &json!({"jsonrpc":"2.0","method":"turn/started","params": {}}),
                        )?;
                        // Response is sent when we receive $/cancelRequest.
                    }
                    other => {
                        eprintln!("turn/start called in unexpected scenario: {other}");
                        std::process::exit(2);
                    }
                }
            }
            "shutdown" => {
                write_result(&mut out, &id, json!({"ok": true}))?;
            }
            other => {
                eprintln!("unknown method: {other}");
                std::process::exit(2);
            }
        }
    }

    Ok(())
}
