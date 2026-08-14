use sha2::{Digest, Sha256};
use substrate_common::macos_retirement_v2::{
    canonical_bytes_v2, parse_canonical_v2, MAC_R3_COORDINATOR_PATH_V2,
    MAC_R3_FINALIZER_ENDPOINT_V2, MAC_R3_FINALIZER_JOURNAL_ROOT_V2,
    MAC_R3_FINALIZER_LAUNCHD_LABEL_V2, MAC_R3_FINALIZER_PATH_V2,
    MAC_R3_FINALIZER_PROTOCOL_OWNER_V2, MAC_R3_FINALIZER_PROTOCOL_VERSION_V2,
};

#[test]
fn capability_manifest_is_canonical_and_is_the_compiled_digest_authority() {
    let bytes = include_bytes!("../capability-v2.json");
    let value: serde_json::Value = parse_canonical_v2(bytes).expect("canonical capability JSON");
    assert_eq!(canonical_bytes_v2(&value).unwrap(), bytes);
    assert_eq!(
        format!("{:x}", Sha256::digest(bytes)),
        substrate_r3_macos_finalizer::frozen_identity::CAPABILITY_DIGEST
    );
    assert_eq!(value["schema_version"], 2);
    assert_eq!(
        value["protocol"]["owner"],
        MAC_R3_FINALIZER_PROTOCOL_OWNER_V2
    );
    assert_eq!(
        value["protocol"]["version"],
        MAC_R3_FINALIZER_PROTOCOL_VERSION_V2
    );
    assert_eq!(
        value["protocol"]["finalizer_path"],
        MAC_R3_FINALIZER_PATH_V2
    );
    assert_eq!(
        value["protocol"]["coordinator_path"],
        MAC_R3_COORDINATOR_PATH_V2
    );
    assert_eq!(value["protocol"]["endpoint"], MAC_R3_FINALIZER_ENDPOINT_V2);
    assert_eq!(
        value["protocol"]["launchd_label"],
        MAC_R3_FINALIZER_LAUNCHD_LABEL_V2
    );
    assert_eq!(
        value["protocol"]["journal_root"],
        MAC_R3_FINALIZER_JOURNAL_ROOT_V2
    );
    assert_eq!(value["authority"]["private_signing_key"], false);
    assert_eq!(value["signer_access"]["owner_uid"], 4_294_967_295_u64);
    assert_eq!(value["signer_access"]["owner_gid"], 4_294_967_295_u64);
    assert_eq!(value["signer_access"]["owner_type"], 3);
    assert_eq!(
        value["process_identity"]["code_directory_flags"],
        substrate_r3_macos_finalizer::experiment::AD_HOC_HARDENED_RUNTIME_FLAGS_V2
    );
    assert_eq!(
        value["process_identity"]["effective_principal"],
        serde_json::json!({"account": "spensermcconnell", "gid": 20, "uid": 501})
    );
    assert!(value["process_identity"]
        .get("effective_uid_and_account")
        .is_none());
    assert_eq!(
        value["target_sets"]["disposable_exact_order"],
        serde_json::json!([
            "sign_control",
            "export_private_control",
            "replace_access_control",
            "delete_wrong_key_control",
            "protected_wrapper",
            "signing_key",
            "current_lock",
            "retirement_terminal_latch"
        ])
    );
}
