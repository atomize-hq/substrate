//! Private test process hosting the actual Responses handler and OAuth adapter.
use super::*;
use crate::auth::codex_auth_context::{
    integrated_codex_auth_test_lock, CodexIntegratedAuthHandoff, InstalledHandoffGuard,
};
use crate::cli::ModelMapping;
use crate::providers::ProviderRegistry;
use crate::server::openai_conformance_test_support::ConformanceHarness;
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::time::{Duration, Instant};

struct Responder(Child);
impl Drop for Responder {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

async fn wait_json(path: &Path) -> Value {
    let deadline = Instant::now() + Duration::from_secs(15);
    loop {
        if let Ok(bytes) = fs::read(path) {
            if let Ok(value) = serde_json::from_slice(&bytes) {
                return value;
            }
        }
        assert!(
            Instant::now() < deadline,
            "missing readiness: {}",
            path.display()
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

async fn serve(root: &Path, preflight: bool) {
    fs::create_dir_all(root).unwrap();
    let log = fs::File::create(root.join("responder.log")).unwrap();
    let _responder = Responder(
        Command::new("/usr/bin/python3")
            .arg(
                Path::new(env!("CARGO_MANIFEST_DIR"))
                    .join("tests/fixtures/codex0156_native_responder.py"),
            )
            .arg("--root")
            .arg(root)
            .env_clear()
            .env("PATH", "/usr/bin:/bin")
            .stdin(Stdio::null())
            .stdout(log.try_clone().unwrap())
            .stderr(log)
            .spawn()
            .expect("Python ssl responder"),
    );
    let ready = wait_json(&root.join("tls-ready.json")).await;
    let port = ready["port"].as_u64().unwrap() as u16;
    assert_ne!(port, 0);
    let certificate =
        reqwest::Certificate::from_pem(&fs::read(root.join("certificate.pem")).unwrap()).unwrap();
    let client = Client::builder()
        .no_proxy()
        .resolve("chatgpt.com", ([127, 0, 0, 1], port).into())
        .add_root_certificate(certificate)
        .redirect(reqwest::redirect::Policy::none())
        .timeout(Duration::from_secs(40))
        .build()
        .unwrap();
    let _lock = integrated_codex_auth_test_lock().lock().await;
    let _auth = InstalledHandoffGuard::set(Some(CodexIntegratedAuthHandoff::new(
        Some("native0156-synthetic-account".into()),
        secrecy::SecretString::new("native0156-synthetic-token".into()),
    )));
    let mut provider = OpenAIProvider::with_headers(OpenAIProviderConfig {
        name: "native0156-fixture".into(),
        api_key: String::new(),
        base_url: "https://unused.invalid".into(),
        models: vec!["gpt-6-astra".into()],
        custom_headers: vec![],
        oauth_provider: Some("native0156-fixture".into()),
        token_store: None,
        codex_auth_source: Some(CodexAuthSource::Integrated),
    });
    assert_eq!(
        provider.effective_base_url(),
        "https://chatgpt.com/backend-api"
    );
    provider.responses_client = client;
    let mut registry = ProviderRegistry::new();
    registry.insert_provider_for_tests("native0156-fixture", Box::new(provider));
    let harness = ConformanceHarness::with_registry(
        "gpt-6-astra",
        registry,
        vec![ModelMapping {
            priority: 1,
            provider: "native0156-fixture".into(),
            actual_model: "openai-codex/gpt-6-astra".into(),
        }],
    );
    // A diagnostic can retain the exact observed loopback URL/config. This
    // changes only the private fixture listener, never the OAuth upstream URL.
    let bind_port = fs::read_to_string(root.join("gateway-bind-port"))
        .map(|value| value.trim().parse::<u16>().expect("fixture port"))
        .unwrap_or(0);
    let listener = tokio::net::TcpListener::bind((std::net::Ipv4Addr::LOCALHOST, bind_port))
        .await
        .unwrap();
    let address = listener.local_addr().unwrap();
    let server = tokio::spawn(async move {
        axum::serve(listener, harness.router()).await.unwrap();
    });
    fs::write(
        root.join("gateway-ready.json"),
        serde_json::to_vec(&json!({
            "url": format!("http://{address}/v1"), "pid": std::process::id(), "tls_port": port,
        }))
        .unwrap(),
    )
    .unwrap();
    if preflight {
        fs::write(root.join("scenario.json"), r#"{"mode":"preflight"}"#).unwrap();
        let response = Client::builder()
            .no_proxy()
            .build()
            .unwrap()
            .post(format!("http://{address}/v1/responses"))
            .json(&json!({"model":"gpt-6-astra", "input":"TLS preflight", "stream":true}))
            .send()
            .await
            .unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        fs::write(root.join("preflight-response.sse"), &body).unwrap();
        assert!(status.is_success(), "{status}: {body}");
        assert!(body.contains("native0156 TLS verified"), "{body}");
        assert!(body.contains("response.completed"), "{body}");
        let request = wait_json(&root.join("request-000.json")).await;
        assert_eq!(request["synthetic_auth"], true);
        assert_eq!(request["synthetic_account"], true);
        fs::write(
            root.join("tls-preflight-passed.json"),
            serde_json::to_vec_pretty(&json!({
                "fixed_host": "chatgpt.com", "nonzero_resolver_port": port,
                "normal_hostname_verification": true, "redirects": false, "proxy": false,
                "actual_oauth_adapter": true, "actual_responses_router": true,
            }))
            .unwrap(),
        )
        .unwrap();
    } else {
        let deadline = Instant::now() + Duration::from_secs(240);
        while !root.join("stop").exists() {
            assert!(Instant::now() < deadline, "fixture deadline");
            tokio::time::sleep(Duration::from_millis(25)).await;
        }
    }
    server.abort();
    let _ = server.await;
}

#[tokio::test]
#[ignore = "requires Python ssl, openssl and loopback; explicit native qualification"]
async fn codex0156_local_https_adapter_preflight() {
    let root = PathBuf::from(
        std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR")
            .expect("attempt directory required"),
    );
    serve(&root, true).await;
}

#[tokio::test]
#[ignore = "private fixture worker, launched by the manager unit test"]
async fn codex0156_actual_gateway_fixture_worker() {
    let root = PathBuf::from(
        std::env::var_os("SUBSTRATE_NATIVE_CODEX_0156_ATTEMPT_DIR")
            .expect("attempt directory required"),
    );
    serve(&root, false).await;
}
