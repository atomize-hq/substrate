use agent_api_types::{
    GatewayLifecycleRequestV1, GatewayStatusV1, PolicySnapshotV3,
    PolicySnapshotWorldFsFailClosedV3, PolicySnapshotWorldFsV3, PolicySnapshotWorldFsWriteV3,
};
use axum::body::Bytes;
use axum::extract::State;
use world_agent::handlers;
use world_agent::WorldAgentService;

fn minimal_policy_snapshot() -> PolicySnapshotV3 {
    PolicySnapshotV3 {
        schema_version: 3,
        net_allowed: Vec::new(),
        world_fs: PolicySnapshotWorldFsV3 {
            host_visible: true,
            fail_closed: PolicySnapshotWorldFsFailClosedV3 { routing: false },
            deny_enforcement: None,
            caged_required: false,
            discover: None,
            read: None,
            write: PolicySnapshotWorldFsWriteV3::default(),
        },
    }
}

fn gateway_request() -> GatewayLifecycleRequestV1 {
    GatewayLifecycleRequestV1 {
        profile: None,
        cwd: None,
        env: None,
        agent_id: "gateway-test".to_string(),
        policy_snapshot: minimal_policy_snapshot(),
        world_network: None,
    }
}

#[tokio::test]
async fn gateway_runtime_status_route_returns_unavailable_without_client_wiring() {
    let service = match WorldAgentService::new() {
        Ok(service) => service,
        Err(err) => {
            eprintln!("skipping gateway status route test: service init failed: {err}");
            return;
        }
    };
    let body = Bytes::from(
        serde_json::to_vec(&gateway_request()).expect("serialize gateway lifecycle request"),
    );

    let response = handlers::gateway_status(State(service), body)
        .await
        .expect("gateway status");

    assert_eq!(response.0.status, GatewayStatusV1::Unavailable);
    assert!(response.0.client_wiring.is_none());
}

#[tokio::test]
async fn gateway_runtime_sync_route_uses_typed_response_shape() {
    let service = match WorldAgentService::new() {
        Ok(service) => service,
        Err(err) => {
            eprintln!("skipping gateway sync route test: service init failed: {err}");
            return;
        }
    };
    let body = Bytes::from(
        serde_json::to_vec(&gateway_request()).expect("serialize gateway lifecycle request"),
    );

    let response = handlers::gateway_sync(State(service), body)
        .await
        .expect("gateway sync");

    assert_eq!(response.0.status, GatewayStatusV1::Unavailable);
    assert!(response.0.client_wiring.is_none());
}

#[tokio::test]
async fn gateway_runtime_restart_route_preserves_component_unavailable_classification() {
    let service = match WorldAgentService::new() {
        Ok(service) => service,
        Err(err) => {
            eprintln!("skipping gateway restart route test: service init failed: {err}");
            return;
        }
    };
    let body = Bytes::from(
        serde_json::to_vec(&gateway_request()).expect("serialize gateway lifecycle request"),
    );

    let response = handlers::gateway_restart(State(service), body)
        .await
        .expect("gateway restart");

    assert_eq!(response.0.status, GatewayStatusV1::Unavailable);
    assert!(response.0.client_wiring.is_none());
}
