#![cfg(feature = "aider")]

use std::{collections::BTreeMap, fs, path::PathBuf, pin::Pin, time::Duration};

use agent_api::{
    backends::aider::{AiderBackend, AiderBackendConfig},
    AgentWrapperBackend, AgentWrapperEvent, AgentWrapperEventKind, AgentWrapperRunRequest,
};
use futures_core::Stream;
use serde_json::Value;
use tempfile::tempdir;

async fn drain_to_none(
    mut stream: Pin<&mut (dyn Stream<Item = AgentWrapperEvent> + Send)>,
    timeout: Duration,
) -> Vec<AgentWrapperEvent> {
    let mut out = Vec::new();
    let deadline = tokio::time::sleep(timeout);
    tokio::pin!(deadline);

    loop {
        tokio::select! {
            _ = &mut deadline => break,
            item = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)) => {
                match item {
                    Some(ev) => out.push(ev),
                    None => break,
                }
            }
        }
    }

    out
}

fn fake_aider_binary() -> PathBuf {
    PathBuf::from(env!("CARGO_BIN_EXE_fake_aider_stream_json_agent_api"))
}

#[tokio::test]
async fn aider_runtime_follow_on_default_tier_contract() {
    let temp = tempdir().expect("tempdir");
    let capture_path = temp.path().join("capture.json");
    let working_dir = temp.path().join("workspace");
    fs::create_dir_all(&working_dir).expect("create working dir");

    let backend = AiderBackend::new(AiderBackendConfig {
        binary: Some(fake_aider_binary()),
        env: BTreeMap::from([
            (
                "FAKE_AIDER_CAPTURE".to_string(),
                capture_path.display().to_string(),
            ),
            (
                "FAKE_AIDER_SCENARIO".to_string(),
                "capture_args".to_string(),
            ),
        ]),
        ..Default::default()
    });

    let handle = backend
        .run(AgentWrapperRunRequest {
            prompt: "Ship the runtime follow-on".to_string(),
            working_dir: Some(working_dir.clone()),
            extensions: [(
                "agent_api.config.model.v1".to_string(),
                Value::String("sonnet".to_string()),
            )]
            .into_iter()
            .collect(),
            ..Default::default()
        })
        .await
        .expect("run aider backend");

    let mut events = handle.events;
    let completion = handle.completion;
    let seen = drain_to_none(events.as_mut(), Duration::from_secs(2)).await;

    assert!(seen
        .iter()
        .any(|event| event.kind == AgentWrapperEventKind::Status));
    assert!(seen.iter().any(|event| {
        event.kind == AgentWrapperEventKind::TextOutput
            && event.text.as_deref() == Some("Hello from fake aider")
    }));

    let completion = tokio::time::timeout(Duration::from_secs(2), completion)
        .await
        .expect("completion resolves")
        .expect("successful completion");
    assert!(completion.status.success());
    assert_eq!(
        completion.final_text.as_deref(),
        Some("Hello from fake aider")
    );

    let capture: Value = serde_json::from_slice(&fs::read(&capture_path).expect("read capture"))
        .expect("parse capture");
    let argv = capture["argv"].as_array().expect("argv array");
    let argv = argv
        .iter()
        .map(|value| value.as_str().expect("argv string"))
        .collect::<Vec<_>>();
    assert_eq!(
        argv,
        vec![
            "--prompt",
            "Ship the runtime follow-on",
            "--message-format",
            "stream-json",
            "--model",
            "sonnet",
        ]
    );
    let captured_cwd = PathBuf::from(capture["cwd"].as_str().expect("captured cwd"));
    assert_eq!(
        captured_cwd.canonicalize().expect("canonical captured cwd"),
        working_dir.canonicalize().expect("canonical working dir")
    );
}
