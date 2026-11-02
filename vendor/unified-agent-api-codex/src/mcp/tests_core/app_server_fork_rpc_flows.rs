use super::super::test_support::{prelude::*, *};
use super::super::*;

async fn read_rpc_log(path: &PathBuf) -> Vec<Value> {
    let Ok(contents) = fs::read_to_string(path) else {
        return Vec::new();
    };

    contents
        .lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<Value>(line).ok())
        .collect()
}

fn requests_by_method<'a>(messages: &'a [Value], method: &str) -> Vec<&'a Value> {
    messages
        .iter()
        .filter(|message| message.get("method").and_then(Value::as_str) == Some(method))
        .collect()
}

async fn wait_for_method_count(path: &PathBuf, method: &str, expected: usize) -> Vec<Value> {
    for _ in 0..100 {
        let messages = read_rpc_log(path).await;
        if requests_by_method(&messages, method).len() >= expected {
            return messages;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    read_rpc_log(path).await
}

#[tokio::test]
async fn fork_v1_initialize_sets_experimental_api_capability() {
    let (_dir, server, rpc_log) = start_fake_app_server_fork_v1().await;

    let messages = wait_for_method_count(&rpc_log, "initialize", 1).await;
    let init = requests_by_method(&messages, "initialize")
        .first()
        .copied()
        .expect("initialize request should be logged");

    assert_eq!(
        init.get("params")
            .and_then(|params| params.get("capabilities"))
            .and_then(|caps| caps.get("experimentalApi"))
            .and_then(Value::as_bool),
        Some(true)
    );

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn thread_list_paging_and_last_selection_is_deterministic() {
    let (_dir, server, rpc_log) = start_fake_app_server_fork_v1().await;

    let cwd = PathBuf::from("/tmp/codex-wrapper-test-cwd");
    let selected = server
        .select_last_thread_id(cwd.clone())
        .await
        .expect("select last thread id");
    assert_eq!(selected.as_deref(), Some("t-c"));

    let messages = wait_for_method_count(&rpc_log, METHOD_THREAD_LIST, 2).await;
    let calls = requests_by_method(&messages, METHOD_THREAD_LIST);
    assert_eq!(calls.len(), 2);

    let cwd_str = cwd.to_str().expect("cwd should be valid utf-8");

    let first_params = calls[0].get("params").expect("thread/list params");
    assert_eq!(
        first_params.get("cwd").and_then(Value::as_str),
        Some(cwd_str)
    );
    assert_eq!(
        first_params.get("sortKey").and_then(Value::as_str),
        Some("updated_at")
    );
    assert_eq!(first_params.get("limit").and_then(Value::as_i64), Some(100));
    assert!(
        first_params
            .get("cursor")
            .map(|value| value.is_null())
            .unwrap_or(false),
        "first thread/list call must include cursor=null"
    );

    let second_params = calls[1].get("params").expect("thread/list params");
    assert_eq!(
        second_params.get("cursor").and_then(Value::as_str),
        Some("cursor-1")
    );

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn thread_fork_parses_result_thread_id() {
    let (_dir, server, rpc_log) = start_fake_app_server_fork_v1().await;

    let response = server
        .thread_fork(ThreadForkParams {
            thread_id: "t-c".to_string(),
            cwd: None,
            approval_policy: None,
            sandbox: None,
            persist_extended_history: None,
        })
        .await
        .expect("thread fork");
    assert_eq!(response.thread.id, "forked-t-c");

    let messages = wait_for_method_count(&rpc_log, METHOD_THREAD_FORK, 1).await;
    let call = requests_by_method(&messages, METHOD_THREAD_FORK)
        .first()
        .copied()
        .expect("thread/fork request should be logged");

    let params = call.get("params").expect("thread/fork params");
    assert_eq!(params.get("threadId").and_then(Value::as_str), Some("t-c"));

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn turn_start_v2_maps_prompt_to_pinned_input_shape() {
    let (_dir, server, rpc_log) = start_fake_app_server_fork_v1().await;

    let forked_id = "forked-t-c".to_string();
    let prompt = "hello fork";

    let handle = server
        .turn_start_v2(TurnStartParamsV2 {
            thread_id: forked_id.clone(),
            input: vec![UserInputV2::text(prompt)],
            approval_policy: Some("never".to_string()),
            cwd: None,
        })
        .await
        .expect("turn/start");

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("turn/start response timeout")
        .expect("turn/start response recv");
    let _ = response.expect("turn/start response ok");

    let messages = wait_for_method_count(&rpc_log, METHOD_TURN_START, 1).await;
    let call = requests_by_method(&messages, METHOD_TURN_START)
        .first()
        .copied()
        .expect("turn/start request should be logged");

    let params = call.get("params").expect("turn/start params");
    assert_eq!(
        params.get("threadId").and_then(Value::as_str),
        Some(forked_id.as_str())
    );
    assert_eq!(
        params.get("approvalPolicy").and_then(Value::as_str),
        Some("never")
    );

    let expected_input = serde_json::json!([
        {"type":"text","text": prompt,"text_elements":[]}
    ]);
    assert_eq!(params.get("input"), Some(&expected_input));

    let _ = server.shutdown().await;
}

#[tokio::test]
async fn cancel_sends_cancel_request_and_maps_cancelled_error() {
    let (_dir, server, rpc_log) = start_fake_app_server_fork_v1().await;

    let handle = server
        .turn_start_v2(TurnStartParamsV2 {
            thread_id: "t-c".to_string(),
            input: vec![UserInputV2::text("cancel me")],
            approval_policy: None,
            cwd: None,
        })
        .await
        .expect("turn/start");

    server.cancel(handle.request_id).expect("send cancel");

    let response = time::timeout(Duration::from_secs(2), handle.response)
        .await
        .expect("turn/start response timeout")
        .expect("turn/start response recv");
    assert!(matches!(response, Err(McpError::Cancelled)));

    let messages = wait_for_method_count(&rpc_log, METHOD_CANCEL, 1).await;
    let cancel_calls = requests_by_method(&messages, METHOD_CANCEL);
    assert!(
        cancel_calls.iter().any(|call| {
            call.get("params")
                .and_then(|params| params.get("id"))
                .and_then(Value::as_u64)
                == Some(handle.request_id)
        }),
        "$/cancelRequest must include params.id == request_id"
    );

    let _ = server.shutdown().await;
}
