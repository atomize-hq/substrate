// Private independently executed service fixture. It never uses installed
// constructors, installed sockets, privileged setup or production overrides.
use super::*;
use axum::{
    body::Body,
    http::Request,
    middleware::{self, Next},
    routing::post,
    Router,
};
use std::io::Write as _;
use std::time::Duration;
use tokio_stream::wrappers::UnixListenerStream;

#[tokio::test(flavor = "current_thread")]
#[ignore = "owned subprocess of the shell ordinary World component integration"]
async fn ordinary_configured_service_worker() {
    // Keep raw JSON-RPC stderr disabled. Only the two closed safe diagnostic
    // targets add DEBUG records to this isolated worker's existing WARN filter.
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::new(
            "off,codex=warn,codex::mcp::safe_lifecycle=debug,substrate::ordinary_home_io=debug",
        ))
        .with_timer(tracing_subscriber::fmt::time::uptime())
        .with_ansi(false)
        .with_writer(std::io::stderr)
        .try_init()
        .unwrap();
    let root = PathBuf::from(
        std::env::var_os("SUBSTRATE_TEST_ORDINARY_ROOT").expect("owned fixture root"),
    );
    assert!(root.is_absolute());
    let carrier: transport_api_types::InstallBootstrapContextCarrierV1 =
        serde_json::from_slice(&std::fs::read(root.join("carrier.json")).unwrap()).unwrap();
    let configured = Arc::new(config_projection::OrdinaryConfiguredAcceptedHomeAuthorityV1::from_component_source_for_test(
        &root.join("bootstrap"), &carrier,
    ).unwrap());
    let authority = Arc::new(
        substrate_shell::OrdinarySelectionAuthorityV1::from_ordinary_configured_accepted_home(
            configured,
        )
        .unwrap(),
    );
    let _world_root = world::SessionWorld::override_shared_root_dir_for_tests(root.join("worlds"));
    let exclusion = E3PrivilegedChildExclusionV1::new_recovering().unwrap();
    exclusion.finish_recovery().unwrap();
    std::fs::create_dir(root.join("turn-state")).unwrap();
    std::fs::set_permissions(
        root.join("turn-state"),
        std::fs::Permissions::from_mode(0o700),
    )
    .unwrap();
    let member_turn_join = Some(MemberTurnJoinRegistry::open(&root.join("turn-state")).unwrap());
    let replay = RuntimeReplayRegistry::with_durable_e2(member_turn_join.clone());
    let backend = Arc::new(world::LinuxLocalBackend::new());
    let fixture_digest = std::fs::read_to_string(root.join("native-fixture.sha256")).unwrap();
    let native_name = std::fs::read_to_string(root.join("native-fixture-name")).unwrap();
    assert!(matches!(native_name.as_str(), "codex" | "claude"));
    let service = WorldService {
        host_gateway_turns: Arc::new(crate::gateway_runtime::HostGatewayTurnOwnerV1::new()),
        ordinary_selection_authority: Some(authority),
        accepted_home_authority: None,
        config_projection_service: None,
        e3_projection_preparations: None,
        backend: backend.clone(),
        linux_backend: backend,
        gateway_runtime: Arc::new(GatewayRuntimeManager::new(Arc::clone(&exclusion))),
        member_runtime: Arc::new(
            MemberRuntimeManager::with_replay_and_e3_projection_service(
                replay.clone(),
                member_turn_join.clone(),
                Arc::clone(&exclusion),
            )
            .with_checked_ordinary_native_fixture_for_test(
                &root.join("workspace").join(&native_name),
                &fixture_digest,
            )
            .unwrap(),
        ),
        runtime_replay: replay,
        member_turn_join,
        pending_diff_origin: Arc::new(RwLock::new(HashMap::new())),
        privileged_child_exclusion: exclusion,
        worlds: Arc::new(RwLock::new(HashMap::new())),
        budgets: Arc::new(RwLock::new(HashMap::new())),
        last_policy_resolution_mode: Arc::new(AtomicU8::new(0)),
        last_netfilter_requested: Arc::new(AtomicU8::new(0)),
        last_netfilter_failure_reason: Arc::new(RwLock::new(None)),
    };
    let capture_root = root.clone();
    let app = Router::new()
        // Private component composition enters the actual internal service,
        // never the production Root/peer-qualified initial HTTP route. These
        // synthetic observations remain ineligible as Root/native proof.
        .route(
            "/v1/execute/stream",
            post(
                |axum::extract::State(service): axum::extract::State<WorldService>,
                 body: axum::body::Bytes| async move {
                    let request =
                        serde_json::from_slice::<transport_api_types::ExecuteRequest>(&body)
                            .map_err(|_| {
                                (
                                    axum::http::StatusCode::BAD_REQUEST,
                                    "component invalid request".to_string(),
                                )
                            })?;
                    service.execute_stream(request).await.map_err(|error| {
                        (
                            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                            error.to_string(),
                        )
                    })
                },
            ),
        )
        .route("/v1/execute/cancel", post(crate::handlers::execute_cancel))
        .route(
            "/v1/member_turn/stream",
            post(crate::handlers::member_turn_stream),
        )
        .route(
            "/v1/ordinary/member_turn/stream",
            post(crate::handlers::ordinary_member_turn_stream),
        )
        .layer(middleware::from_fn(
            move |request: Request<Body>, next: Next<Body>| {
                let capture_root = capture_root.clone();
                async move {
                    // Fixture-local synthetic account control. The parent does
                    // not mutate this process's account environment implicitly.
                    let control = capture_root.join("service-account-control.json");
                    if control.is_file() {
                        let command: serde_json::Value =
                            serde_json::from_slice(&std::fs::read(&control).unwrap()).unwrap();
                        let sequence = command["sequence"].as_u64().unwrap();
                        let state = command["state"].as_str().unwrap();
                        let account = match state {
                            "revoked" => "synthetic-service-revoked-account",
                            "original" => "synthetic-ordinary-account",
                            _ => panic!("closed component account control"),
                        };
                        std::env::set_var(
                            substrate_common::SUBSTRATE_LLM_BACKEND_AUTH_CLI_CODEX_ACCOUNT_ID,
                            account,
                        );
                        std::fs::write(
                            capture_root.join("service-account-ack.json"),
                            serde_json::to_vec(&serde_json::json!({
                                "service_pid":std::process::id(),"sequence":sequence,
                                "state":state,"component_only":true
                            }))
                            .unwrap(),
                        )
                        .unwrap();
                    }
                    let (parts, body) = request.into_parts();
                    let bytes = hyper::body::to_bytes(body).await.unwrap();
                    assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-ordinary-token"));
                    assert!(!String::from_utf8_lossy(&bytes)
                        .contains("synthetic-ordinary-anthropic-key"));
                    let body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    let mut output = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(capture_root.join("received.jsonl"))
                        .unwrap();
                    writeln!(
                        output,
                        "{}",
                        serde_json::json!({"path":parts.uri.path(),"body":body})
                    )
                    .unwrap();
                    let response_path = parts.uri.path().to_owned();
                    let response = next
                        .run(Request::from_parts(parts, Body::from(bytes)))
                        .await;
                    if response.status().is_success() {
                        return response;
                    }
                    // Only finite synthetic-fixture errors are observed here;
                    // successful native streams retain their real streaming body.
                    let (parts, body) = response.into_parts();
                    let bytes = hyper::body::to_bytes(body).await.unwrap();
                    assert!(!String::from_utf8_lossy(&bytes).contains("synthetic-ordinary-token"));
                    assert!(!String::from_utf8_lossy(&bytes)
                        .contains("synthetic-ordinary-anthropic-key"));
                    let error_body: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
                    let mut errors = std::fs::OpenOptions::new()
                        .create(true)
                        .append(true)
                        .open(capture_root.join("response-errors.jsonl"))
                        .unwrap();
                    writeln!(
                        errors,
                        "{}",
                        serde_json::json!({"path":response_path,
                        "status":parts.status.as_u16(),"body":error_body})
                    )
                    .unwrap();
                    axum::response::Response::from_parts(
                        parts,
                        axum::body::boxed(Body::from(bytes)),
                    )
                }
            },
        ))
        .with_state(service.clone());
    let socket = root.join("world.sock");
    let listener = tokio::net::UnixListener::bind(&socket).unwrap();
    let stopping_root = root.clone();
    let server = axum::Server::builder(hyper::server::accept::from_stream(
        UnixListenerStream::new(listener),
    ))
    .serve(app.into_make_service())
    .with_graceful_shutdown(async move {
        // Match the existing shell component outer budget; this fixture must
        // remain available through all three native turns and their postlude.
        let extra = if std::env::var("SUBSTRATE_TEST_ORDINARY_EXTRA_TURN").as_deref() == Ok("1") {
            9 * 30 + 150 + 60
        } else {
            0
        };
        let deadline = tokio::time::Instant::now()
            + Duration::from_secs(3 * (9 * 30 + 150 + 20) + 180 + extra);
        while tokio::time::Instant::now() < deadline {
            if stopping_root.join("stop-service").is_file() {
                return;
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        panic!("component owner failed to stop service within 1500-second component lifetime");
    });
    std::fs::write(root.join("service-ready"), b"candidate service ready").unwrap();
    server.await.unwrap();
    drop(service);
    std::fs::remove_file(socket).unwrap();
    eprintln!("independent configured component service completed; no installed/provider/kernel qualification");
}
