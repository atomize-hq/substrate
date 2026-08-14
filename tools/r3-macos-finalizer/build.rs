use std::{env, fs, path::PathBuf};

use sha2::{Digest, Sha256};

fn literal(name: &str, default: &str) -> String {
    let value = env::var(name).unwrap_or_else(|_| default.to_string());
    assert!(
        !value.is_empty() && !value.contains(['\0', '\n', '\r']),
        "{name} is not a safe build literal"
    );
    println!("cargo:rerun-if-env-changed={name}");
    value
}

fn main() {
    let coordinator_sha256 = literal("R3_COORDINATOR_SHA256", &"0".repeat(64));
    let coordinator_cdhash = literal("R3_COORDINATOR_CDHASH", &"0".repeat(40));
    let coordinator_requirement = literal(
        "R3_COORDINATOR_REQUIREMENT",
        "identifier com.atomize.substrate.r3-macos-evidence-coordinator.v2 and cdhash H\"0000000000000000000000000000000000000000\"",
    );
    let coordinator_uid = literal("R3_COORDINATOR_UID", "501");
    let coordinator_gid = literal("R3_COORDINATOR_GID", "20");
    let coordinator_account = literal("R3_COORDINATOR_ACCOUNT", "spensermcconnell");
    let coordinator_source_commit = literal("R3_COORDINATOR_SOURCE_COMMIT", &"0".repeat(40));
    let coordinator_source_tree = literal("R3_COORDINATOR_SOURCE_TREE", &"0".repeat(40));
    let coordinator_source_hashes_sha256 =
        literal("R3_COORDINATOR_SOURCE_HASHES_SHA256", &"0".repeat(64));
    let coordinator_build_inputs_sha256 =
        literal("R3_COORDINATOR_BUILD_INPUTS_SHA256", &"0".repeat(64));
    let source_commit = literal("R3_SOURCE_COMMIT", &"0".repeat(40));
    let source_tree = literal("R3_SOURCE_TREE", &"0".repeat(40));
    let source_identity_sha256 = literal("R3_SOURCE_IDENTITY_SHA256", &"0".repeat(64));
    let build_inputs_sha256 = literal("R3_BUILD_INPUTS_SHA256", &"0".repeat(64));
    let capability_bytes = include_bytes!("capability-v2.json");
    let capability_digest = format!("{:x}", Sha256::digest(capability_bytes));
    if let Ok(supplied) = env::var("R3_CAPABILITY_DIGEST") {
        assert_eq!(
            supplied, capability_digest,
            "R3_CAPABILITY_DIGEST differs from canonical capability-v2.json"
        );
    }
    println!("cargo:rerun-if-changed=capability-v2.json");
    println!("cargo:rerun-if-env-changed=R3_CAPABILITY_DIGEST");

    let body = format!(
        "pub const EXPECTED_COORDINATOR_SHA256: &str = {coordinator_sha256:?};\n\
         pub const EXPECTED_COORDINATOR_CDHASH: &str = {coordinator_cdhash:?};\n\
         pub const EXPECTED_COORDINATOR_REQUIREMENT: &str = {coordinator_requirement:?};\n\
         pub const EXPECTED_COORDINATOR_UID: u32 = {coordinator_uid};\n\
         pub const EXPECTED_COORDINATOR_GID: u32 = {coordinator_gid};\n\
         pub const EXPECTED_COORDINATOR_ACCOUNT: &str = {coordinator_account:?};\n\
         pub const EXPECTED_COORDINATOR_SOURCE_COMMIT: &str = {coordinator_source_commit:?};\n\
         pub const EXPECTED_COORDINATOR_SOURCE_TREE: &str = {coordinator_source_tree:?};\n\
         pub const EXPECTED_COORDINATOR_SOURCE_HASHES_SHA256: &str = {coordinator_source_hashes_sha256:?};\n\
         pub const EXPECTED_COORDINATOR_BUILD_INPUTS_SHA256: &str = {coordinator_build_inputs_sha256:?};\n\
         pub const SOURCE_COMMIT: &str = {source_commit:?};\n\
         pub const SOURCE_TREE: &str = {source_tree:?};\n\
         pub const SOURCE_IDENTITY_SHA256: &str = {source_identity_sha256:?};\n\
         pub const BUILD_INPUTS_SHA256: &str = {build_inputs_sha256:?};\n\
         pub const CAPABILITY_DIGEST: &str = {capability_digest:?};\n"
    );
    let out = PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    fs::write(out.join("frozen_identity.rs"), body).expect("write frozen build identity");
}
