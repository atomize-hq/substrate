use serde::de::DeserializeOwned;
use serde::Serialize;

use super::canonical_json;
use super::hash::{
    canonical_object_bytes, canonical_sha256, store_hmac_sha256, verify_store_hmac_sha256,
    CanonicalObjectHashInputV1, SensitiveDomainV1,
};
use super::schema::*;
use super::validation::{CanonicalHashInputV1, ValidatedCanonicalV1};

const TEST_KEY: [u8; 32] = [
    0x00, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x09, 0x0a, 0x0b, 0x0c, 0x0d, 0x0e, 0x0f,
    0x10, 0x11, 0x12, 0x13, 0x14, 0x15, 0x16, 0x17, 0x18, 0x19, 0x1a, 0x1b, 0x1c, 0x1d, 0x1e, 0x1f,
];
const STORE_ID: &str = "as_11111111111111111111111111111111";
const INTENT_ID: &str = "intent-vector";
const RUN_ID: &str = "run-vector";

fn assert_canonical_vector<T>(bytes: &[u8], digest: &str)
where
    T: CanonicalHashInputV1 + DeserializeOwned + Serialize,
{
    let decoded: T = canonical_json::from_slice(bytes).expect("typed canonical fixture");
    assert_eq!(canonical_json::to_vec(&decoded).unwrap(), bytes);
    assert_eq!(canonical_sha256(&decoded).unwrap(), digest);
}

#[test]
fn every_named_canonical_wrapper_matches_committed_bytes_and_sha256() {
    assert_canonical_vector::<AuthorityObjectRefV1>(
        include_bytes!("testdata/typed-ref.json"),
        "7b6bb46e174c4543ca34297ea42fb23fa88cd200ab39f3a67e3821456d70d809",
    );
    assert_canonical_vector::<AgentDescriptorHashInputV1>(
        include_bytes!("testdata/agent-descriptor.json"),
        "53987fff3fd68515d2ca74b99edbd0bf00464d3f2c498a686973e79e36d3360f",
    );
    assert_canonical_vector::<HostAttachContractHashInputV1>(
        include_bytes!("testdata/host-attach-contract.json"),
        "f14321a64e1aed1289213842e039adeca973bf4445e194f2415308bf21aceeea",
    );
    assert_canonical_vector::<ResumeHandleHashInputV1>(
        include_bytes!("testdata/resume-handle.json"),
        "88a61b297ad8762f007bf43eefc72b2d8b6ce39294a5ff95ac7031818640b249",
    );
    assert_canonical_vector::<PolicyObjectHashInputV1>(
        include_bytes!("testdata/policy.json"),
        "29274cab6bbe39d937b44ac25309d9d86d4b22041d4ad1a3de10fc3a04fac583",
    );
    assert_canonical_vector::<RetainedWorkerObjectHashInputV1>(
        include_bytes!("testdata/retained-worker.json"),
        "c16ab8b56bd8faafd9723ca39979a2d74736afabdfe8c83ad62e2475661739ea",
    );
    assert_canonical_vector::<DurableSessionAuthorityHashInputV1>(
        include_bytes!("testdata/authority.json"),
        "6197cb7c195ad9004fbf728328ee343338d69c80e44b6292438130fe7af8b067",
    );
    assert_canonical_vector::<AuthoritativeLineageHashInputV1>(
        include_bytes!("testdata/lineage.json"),
        "7e9cf44d4cf1c3bdf0c80cb52ea08d6611fa60f46f98b0c903cbf2ff9cc851c9",
    );
    assert_canonical_vector::<HostSessionTransitionPayloadHashInputV1>(
        include_bytes!("testdata/payload-start.json"),
        "3f3ef582d50fb0498e0f353355a1c23a5609e795aefb5d6b9ddc610e7722aee2",
    );
    assert_canonical_vector::<HostSessionTransitionPayloadHashInputV1>(
        include_bytes!("testdata/payload-attach.json"),
        "85714e8c3a6209cf48fe56ddf776fce994619751a288526db4ea643da8883d57",
    );
    assert_canonical_vector::<HostSessionTransitionPayloadHashInputV1>(
        include_bytes!("testdata/payload-resume.json"),
        "4bfe223facc9cd3e6adcd3b8f1fece91c5b825dc106cf634bb13d81abf0e2f4a",
    );
    assert_canonical_vector::<ApplicationResultHashInputV1>(
        include_bytes!("testdata/application-initial.json"),
        "bf94b6cb2b53b15d59816389156097a0a218395b3934ee7a9dd540caf740483c",
    );
    assert_canonical_vector::<ApplicationResultHashInputV1>(
        include_bytes!("testdata/application-post-turn.json"),
        "2435946270e383e06450ebb255df81242c5b51d60f4c0eeeb143e03734fa9841",
    );
    assert_canonical_vector::<InputAcceptanceHashInputV1>(
        include_bytes!("testdata/input-acceptance.json"),
        "f0038bc270e7ec7b2c140d38c7077ac581db5c46b0feb8c4395f1b1ad2ee7a05",
    );
    assert_canonical_vector::<PostTurnCompletionHashInputV1>(
        include_bytes!("testdata/post-turn-completion.json"),
        "5f09e17c0e0822f97e04d13977fb4b009b62a49ad92703b629bb953d82ec704c",
    );
    assert_canonical_vector::<TerminalHandoffHashInputV1>(
        include_bytes!("testdata/terminal-handoff.json"),
        "394aec4394909d759243c93151b0a60a4b3ccb06e456ddb2e46be6fc8cb7f015",
    );
}

#[test]
fn all_sensitive_domains_match_independent_fixed_key_vectors() {
    let input = include_bytes!("testdata/transition-input.bin");
    let lease = include_bytes!("testdata/lease-token.bin");
    let transport = include_bytes!("testdata/transport-payload.json");

    assert_eq!(
        store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::TransitionInput,
            STORE_ID,
            INTENT_ID,
            Some(RUN_ID),
            input,
        )
        .unwrap(),
        "b36127c24806ea110d1ac175557f46741ebfd83a81b3e724de47cb714ca6ddd6"
    );
    assert_eq!(
        store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::ParticipantLeaseToken,
            STORE_ID,
            INTENT_ID,
            Some(RUN_ID),
            lease,
        )
        .unwrap(),
        "0d23a9e6fd93b181df7e5b04023a5eb695bc27e97f7dd8a1af63d36d417a852b"
    );
    assert_eq!(
        store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::RawTransportPayload,
            STORE_ID,
            INTENT_ID,
            Some(RUN_ID),
            transport,
        )
        .unwrap(),
        "e1c94b281a8a65afc076c41d78340f1337a76f2464aa864e7ac5abb4132d39d1"
    );
    let decoded: TransitionTransportPayloadObjectV1 =
        canonical_json::from_slice(transport).unwrap();
    decoded.validate().unwrap();
    assert_eq!(canonical_json::to_vec(&decoded).unwrap(), transport);
}

#[test]
fn sensitive_vectors_reject_absent_empty_or_different_run_binding() {
    for (domain, raw, expected) in [
        (
            SensitiveDomainV1::TransitionInput,
            include_bytes!("testdata/transition-input.bin").as_slice(),
            "b36127c24806ea110d1ac175557f46741ebfd83a81b3e724de47cb714ca6ddd6",
        ),
        (
            SensitiveDomainV1::ParticipantLeaseToken,
            include_bytes!("testdata/lease-token.bin").as_slice(),
            "0d23a9e6fd93b181df7e5b04023a5eb695bc27e97f7dd8a1af63d36d417a852b",
        ),
        (
            SensitiveDomainV1::RawTransportPayload,
            include_bytes!("testdata/transport-payload.json").as_slice(),
            "e1c94b281a8a65afc076c41d78340f1337a76f2464aa864e7ac5abb4132d39d1",
        ),
    ] {
        assert!(verify_store_hmac_sha256(
            expected, &TEST_KEY, domain, STORE_ID, INTENT_ID, None, raw,
        )
        .is_err());
        assert!(verify_store_hmac_sha256(
            expected,
            &TEST_KEY,
            domain,
            STORE_ID,
            INTENT_ID,
            Some(""),
            raw,
        )
        .is_err());
        assert!(verify_store_hmac_sha256(
            expected,
            &TEST_KEY,
            domain,
            STORE_ID,
            INTENT_ID,
            Some("different-run"),
            raw,
        )
        .is_err());
    }
}

#[test]
fn sensitive_domain_specific_payload_rules_fail_before_commitment() {
    for invalid_lease in [b"".as_slice(), [0xff_u8].as_slice()] {
        assert!(store_hmac_sha256(
            &TEST_KEY,
            SensitiveDomainV1::ParticipantLeaseToken,
            STORE_ID,
            INTENT_ID,
            Some(RUN_ID),
            invalid_lease,
        )
        .is_err());
    }

    let mut payload: TransitionTransportPayloadObjectV1 =
        canonical_json::from_slice(include_bytes!("testdata/transport-payload.json")).unwrap();
    payload.run_id = "other-run".to_string();
    let wrong_run = canonical_json::to_vec(&payload).unwrap();
    assert!(store_hmac_sha256(
        &TEST_KEY,
        SensitiveDomainV1::RawTransportPayload,
        STORE_ID,
        INTENT_ID,
        Some(RUN_ID),
        &wrong_run,
    )
    .is_err());

    payload.run_id = RUN_ID.to_string();
    payload.intent_id = "other-intent".to_string();
    let wrong_intent = canonical_json::to_vec(&payload).unwrap();
    assert!(store_hmac_sha256(
        &TEST_KEY,
        SensitiveDomainV1::RawTransportPayload,
        STORE_ID,
        INTENT_ID,
        Some(RUN_ID),
        &wrong_intent,
    )
    .is_err());
}

#[test]
fn named_hash_inputs_fail_validation_before_commitment() {
    let mut descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(include_bytes!("testdata/agent-descriptor.json")).unwrap();
    descriptor.schema_version = 2;
    assert!(canonical_sha256(&descriptor).is_err());
    descriptor.schema_version = 1;
    descriptor.descriptor.backend_id.clear();
    assert!(canonical_sha256(&descriptor).is_err());

    let mut policy: PolicyObjectHashInputV1 =
        canonical_json::from_slice(include_bytes!("testdata/policy.json")).unwrap();
    policy.canonical_policy_snapshot_sha256 = "A".repeat(64);
    assert!(canonical_sha256(&policy).is_err());

    let mut contract: HostAttachContractHashInputV1 =
        canonical_json::from_slice(include_bytes!("testdata/host-attach-contract.json")).unwrap();
    contract.contract.descriptor_ref.object_kind = AuthorityObjectKindV1::Policy;
    assert!(canonical_sha256(&contract).is_err());

    let mut retained: RetainedWorkerObjectHashInputV1 =
        canonical_json::from_slice(include_bytes!("testdata/retained-worker.json")).unwrap();
    retained.resume_handle_ref.object_kind = AuthorityObjectKindV1::AgentDescriptor;
    assert!(canonical_sha256(&retained).is_err());
}

#[test]
fn canonical_object_dispatch_rejects_cross_kind_bytes() {
    let descriptor: AgentDescriptorHashInputV1 =
        canonical_json::from_slice(include_bytes!("testdata/agent-descriptor.json")).unwrap();
    assert!(canonical_object_bytes(
        AuthorityObjectKindV1::AgentDescriptor,
        CanonicalObjectHashInputV1::AgentDescriptor(&descriptor),
    )
    .is_ok());
    assert!(canonical_object_bytes(
        AuthorityObjectKindV1::Policy,
        CanonicalObjectHashInputV1::AgentDescriptor(&descriptor),
    )
    .is_err());
}
