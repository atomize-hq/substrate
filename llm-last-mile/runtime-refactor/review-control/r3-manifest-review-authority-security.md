# R3 MANIFEST authority and security review

Terminal subject fingerprint:
`sha256:8b9a601620f69303c60d458977f48509f48c0ba8fd977afa713a39b57fb6e394`

The subject is the exact `A1.1d-5R3-MANIFEST` packet fence: the allowlisted root/common/shell/CI
and `R3-DOCS` files only. Post-subject review metadata in this directory is excluded from that
fingerprint so it can attest to the fixed packet bytes without self-reference.

A fresh independent read-only `gpt-5.4` reviewer using Extra High reasoning and standard/default
speed evaluated a bounded evidence bundle derived from the packet fence and returned:

> NO BLOCKING FINDINGS
> Residual assumptions: The bounded summary is complete and accurate, and the intentionally
> provider_unavailable platform stubs are acceptable interim behavior for this packet.

## Authority boundary

The landed MANIFEST subject remains non-destructive:

- `src/bin/substrate-lifecycle-control.rs` rejects submit authority domains other than
  `unix_a_local` and `windows_a_local`; `linux_system`, `mac_lima_guest`, `windows_wsl_guest`,
  `mac_host_shared`, and `windows_host_shared` fail closed until their dedicated platform packets
  land.
- `submit_managed_lifecycle_request_v1` requires `publisher_protected_state` before action-receipt
  or publisher-request submission and keeps provider execution behind preserving
  `provider_unavailable` stubs.
- No delete/replace/stop/kill/unregister/restore execution path is added. The packet records and
  validates manifest/bootstrap authority only.

## Join and signature checks

The subject closes the earlier receipt/publication trust gaps without widening authority:

- `validate_lifecycle_publisher_protected_state_v1` requires a prepared record, when present, to
  join `authority_domain`, `scope_id`, `manifest_generation`, `manifest_sha256`, signer identity,
  and counter to the current anchor.
- `resume_action_receipt_commit_v1` validates protected state, requires the prepared record,
  requires receipt signer equality with the prepared record, binds receipt fields back to the
  prepared record and selected manifest, then performs receipt write -> receipt-index CAS -> head
  CAS in order.
- `validate_publisher_request_binding_v1` binds control request, manifest, head, protected-state
  anchor, counter, anchor digest, and planned action before any provider request is accepted.

## Persistence safety

The packet preserves fail-closed publication behavior:

- exact-existing write retry fsyncs the parent directory before returning success;
- temp-file residue is rejected rather than rejoined; and
- provider channels remain stubs instead of inventing provisional cross-platform authority.

- P1: none.
- P2: none.
- P3: none.
- P4: none.

Protocol terminal verdict: `CLEAN`.

Separate increment publication gate: `PASS` with zero unresolved P1-P4.
