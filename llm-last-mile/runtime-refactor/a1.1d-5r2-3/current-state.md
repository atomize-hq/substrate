**Kind:** current-state projection
**Stable ID:** `A1.1d-5R2-3-family`
**Canonical for:** A1.1d-5R2-3 closeout-status projection
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`00-README.md#a11d-5r2-3-closeout-status`](../00-README.md#a11d-5r2-3-closeout-status) lines 979–1022
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-3 closeout-status projection

## A1.1d-5R2-3 closeout status

`R2-3ZP2` is accepted landed source, but `R2-3ZP3` is permanently deferred from the blocking R2-3
path. Its unpublished commits `2cb796ffef68c2b049376984a90ce0382e5f3980`,
`9fa3fe0d4ed2933521dfcd67919d91aa9da6a499`, `50948dbeb921582515a34bb6b7be21c46f29d008`, and
`868994efaf132bb04c6cdd7c82da9433333c94e5` are diagnostic evidence only. They are not accepted
product source and must not be cherry-picked, pushed, or represented as landed.

The tracked canonical shell-wall Python runner remains historical proof-infrastructure evidence
outside the accepted closeout gate. Its authenticated `1322 discovered / 1277 passed / 45 failed /
0 ignored` result is diagnostic only because the runner still finalized provenance-ineligible with
`evidence_write_failed` and `mount_teardown_failed`, leaving authenticated stage-A completion and
hidden backing-path teardown unproven. For this exact source, the sole normative broad shell-wall
entrypoints are `make shell-lib-wall` and `make shell-lib-wall-serial`; they validate the private
environment, run the exact Cargo argv, stream Cargo output unchanged, and leave GNU Make's
standard public exit mapping intact.

The accepted closeout chain is `R2-3ZT1 -> R2-3ZH1 -> R2-3ZM5 -> fresh native macOS and Windows
evidence -> R2-3Z`, with landed commits `56e0a8582d562bd7e60e8f4348b4d596e1b2b36e`,
`610db8a9350c9b52496954f5c93232d885f439d9`, and
`c583c5f293644fab75d8d42bd3bcad63f114d4fe`. The accepted direct shell-library proof at that final
source is `1322 discovered / 1274 passed / 48 failed / 0 ignored`, with failure-name SHA-256
`c6de1349137dcb16d03b87be5364dc50d74a5052565e2c8d40dfed303592bed9` and normalized-signature
SHA-256 `2a0df9b340cc7e5e1b6e4f76e60e6f937b7442008142d78a7ae24bbcd2f60a90`. The historical 45-failure
inventory remains present, and the only additional failures are the separately classified,
non-R2-3 world-deps/report expectations
`builtins::shim_doctor::report::tests::world_deps_fixture_cannot_establish_runtime_health_or_cross_a`,
`builtins::shim_doctor::report::tests::world_deps_section_forwards_authenticated_a_under_conflicting_ambient_b`,
and `builtins::world_deps::tests::doctor_snapshot_uses_authenticated_a_under_conflicting_ambient_b_without_mutation`.
Count-only equivalence remains insufficient: any additional failure, missing frozen failure, or
changed normalized signature still blocks the closeout truth. `R2-3ZH1` owns only the host-inbox
trusted-root test helper. `R2-3ZM5` owns only the macOS contextless-constructor removal and typed
pre-R3 smoke contract. The refreshed source-bound native evidence receipts validated clean at
receipt digests `sha256:3b44f6387070b7aaea4306ae58d7f280b1cee3e163ee59219d4900ce3f53dfaf` and
`sha256:4ee942690655c1fac185244438d14e2561df52c306dea7e5428d556b530fd28c`, with artifact digests
`sha256:64a726d45b8bb6724fe43f566903d7b5e5ed6ef16119f14d30cd5a03edcc7e50` and
`sha256:2c36a8dae9f3bcfa1c240c62a1e44f93aee0798702ae78a0076d43778b93d46e`, for gates
`R2-DIAG-01`, `R2-MAP-MAC-01`, and `R2-MAP-WIN-01` without any lifecycle/provisioning claim.
R2-3 therefore closes only the explicitly listed PI rows and leaves its R2-3-owned exceptions
PI-059 harness-only and PI-077/PI-078 byte-frozen fail-closed guards; PI-050 as the R2-4 guardrail; PI-080 satisfied by earlier R2-2 Linux restart-scope
work and not reopened here; every R3 lifecycle/forwarding/provisioning/cleanup/rollback/
convergence item open; and privileged product smoke, direct-member adoption, and the broader
runtime-refactor backlog later.

**Source provenance:**
- extracted from [`00-README.md#a11d-5r2-3-closeout-status`](../00-README.md#a11d-5r2-3-closeout-status), lines 979–1022; baseline span SHA-256 `de067765f2719303e96e34a6f441aa8253a8b9bfcf179e575e9aff2df247f335`
