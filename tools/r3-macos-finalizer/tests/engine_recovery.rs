use substrate_r3_macos_finalizer::engine::AcceptedRequest;

#[test]
fn acceptance_token_cannot_be_fabricated_from_independent_fields() {
    assert!(AcceptedRequest::admit_canonical_request(
        b"canonical-authority-without-signed-request".to_vec(),
        b"peer-attestation".to_vec(),
        Some(1),
    )
    .is_err());
}

#[test]
fn noncanonical_or_duplicate_authority_fields_are_rejected_before_admission() {
    for bytes in [
        br#"{"schema_owner":"substrate.r3-macos-evidence-finalizer","schema_owner":"duplicate"}"#
            .as_slice(),
        br#"{ "schema_owner":"substrate.r3-macos-evidence-finalizer"}"#.as_slice(),
        br#"{}trailing"#.as_slice(),
    ] {
        assert!(AcceptedRequest::admit_canonical_request(
            bytes.to_vec(),
            b"peer-attestation".to_vec(),
            None,
        )
        .is_err());
    }
}
