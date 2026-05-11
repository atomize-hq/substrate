#![cfg(any(target_os = "linux", target_os = "macos"))]

#[path = "support/mod.rs"]
mod support;

use std::env;
use std::path::PathBuf;

#[test]
#[ignore = "dev-support helper invoked explicitly by validation scripts"]
fn authoritative_inbox_dev_support_persists_pending_item_from_env() {
    let substrate_home = PathBuf::from(
        env::var("SUBSTRATE_HOME").expect("SUBSTRATE_HOME must be set for dev-support helper"),
    );
    let orchestration_session_id = env::var("SUBSTRATE_DEV_SUPPORT_SESSION_ID")
        .expect("SUBSTRATE_DEV_SUPPORT_SESSION_ID must be set");
    let item_id = env::var("SUBSTRATE_DEV_SUPPORT_ITEM_ID")
        .expect("SUBSTRATE_DEV_SUPPORT_ITEM_ID must be set");
    let message = env::var("SUBSTRATE_DEV_SUPPORT_MESSAGE").ok();

    support::persist_runtime_alert_for_substrate_home(
        &substrate_home,
        &orchestration_session_id,
        &item_id,
        message,
    );
}
