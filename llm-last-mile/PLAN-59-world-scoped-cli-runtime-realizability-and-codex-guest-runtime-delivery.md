# Plan: World-Scoped CLI Runtime Realizability And Codex Guest Runtime Delivery

Source spec: [SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md](./SPEC-59-world-scoped-cli-runtime-realizability-and-codex-guest-runtime-delivery.md)  
Related slice: [SPEC-58-placement-aware-agent-inventory-and-selector-contract.md](./SPEC-58-placement-aware-agent-inventory-and-selector-contract.md)  
Plan type: runtime-truth + guest runtime delivery + installer integration before placement-aware cutover  
Status: draft for review  
Implementation posture: spec-first, runtime truth first, no placement-aware selector migration in this slice

## Objective

Implement the approved runtime-realizability contract for world-scoped Codex so the repo stops claiming a world runtime is launchable when only a host-local runtime exists.

This plan must land:

1. fail-closed validation/remediation,
2. Substrate-owned guest runtime delivery through world-deps,
3. a remediation packet that closes unsupported-guest and host-runtime leakage gaps before installer work proceeds,
4. prod/dev installer-time provisioning support,
5. and end-to-end proof that the world runtime is guest-visible and not inherited from host NVM/npm state.

## Plan Summary

The repo’s current semantic bug is not selector naming; it is fake world launchability truth.

So the correct sequence is:

1. freeze the runtime-realizability contract,
2. land the fail-closed validator/materialization wall,
3. land the world-deps Codex package or runtime bundle,
4. land a remediation packet that makes unsupported-guest behavior and host-vs-world runtime separation review-clean,
5. land installer support and docs,
6. prove the path end-to-end,
7. only after that, let Slice 58 change the config/selector shape.

Slice 59 therefore **precedes** Slice 58 implementation even though Slice 58 is already specified.

## Locked Decisions

### What changes

1. World-scoped CLI launchability becomes guest-visible truth instead of host `which` truth.
2. Missing guest runtime becomes an early validator/materialization error with remediation.
3. Codex guest runtime delivery uses a Substrate-owned world-deps script package named `codex-runtime` that pulls official release artifacts.
4. Substrate resolves the validated Codex version through the public UAA Rust API (`unified-agent-api = "=0.3.6"`, Rust path `agent_api`) and keeps binary workflow ownership locally; `0.3.6` is the minimum published line for the Codex runtime-version API surface this slice needs.
5. Unsupported guest tuples must fail closed and must never be treated as satisfied by host Codex runtime truth.
6. Prod and dev installers gain the generic public flag shape `--provision-agent-runtime <runtime_family>`, even though this slice only implements the `codex` value.

### What does not change

1. Backend-id grammar stays unchanged.
2. Policy stays keyed on exact backend ids.
3. Placement-aware config/selector migration still belongs to Slice 58.
4. No new first-class artifact transport/checksum/archive system is introduced unless Option A fails.
5. No world refresh/restart requirement is added without new live evidence.

## Implementation Order

### Phase 1: Land The Fail-Closed Runtime Truth Wall

Goal:

1. stop treating host `which codex` as sufficient for world-scoped launchability,
2. reject bad world runtime materialization before retained worker bootstrap,
3. shape remediation around world-deps provisioning and sync.

Primary touch surface:

1. `crates/shell/src/execution/agent_runtime/validator.rs`
2. `crates/shell/src/execution/orchestrator_world_dispatch.rs`
3. `crates/world-service/src/member_runtime.rs` only for alignment/defense-in-depth wording if needed
4. nearby runtime/dispatch tests

Required changes:

1. add explicit world-scoped runtime truth gating,
2. distinguish host binary resolution from guest entrypoint truth,
3. emit stable diagnostics/remediation when guest runtime is missing,
4. ensure world dispatch does not serialize host-only runtime paths as authoritative world runtime truth.

Verification checkpoint:

1. world-scoped Codex fails early when guest runtime is absent,
2. the failure references the remediation path rather than surfacing only a late `127`,
3. host-scoped Codex behavior is unchanged.

### Phase 2: Land The Substrate-Owned Codex World-Deps Package Or Bundle

Goal:

1. make Codex guest-visible through world-deps script packaging,
2. pin the install prefix and entrypoint contract,
3. resolve the validated Codex version through UAA’s public runtime-support API after bumping the published `unified-agent-api` line to `=0.3.6`,
4. prove whether the Linux release binary is self-contained or whether the package must widen into a runtime bundle.

Primary touch surface:

1. world-deps inventory/package definitions under Substrate-managed inventory,
2. package install scripts,
3. Substrate dependency/runtime-selection code that calls into UAA’s public Rust API,
4. world-deps inventory/surface tests,
5. operator docs for authoring/runtime flows.

Required changes:

1. define the package/bundle as `codex-runtime`,
2. bump the published `unified-agent-api` dependency line to `=0.3.6` (plus any already-aligned exact sibling UAA pins in the touched manifests), keep the `codex` feature enabled, and use `agent_api::resolve_runtime_support("codex", target_triple)` to resolve the validated version,
3. keep artifact URL construction, checksum verification, extraction, cache, and install logic in Substrate,
4. install under `/var/lib/substrate/world-deps/<package>`,
5. expose `/var/lib/substrate/world-deps/bin/codex`,
6. fetch official release artifacts from the chosen source,
7. document the verified self-contained-vs-bundle outcome.

Verification checkpoint:

1. the package/bundle installs idempotently,
2. the guest-visible `codex` entrypoint resolves from the world-deps bin prefix,
3. the implementation records which runtime-dependency posture was proven,
4. Substrate resolves against the published `0.3.6` UAA surface and does not duplicate version-selection logic outside the public UAA API.

### Phase 2.5: Close The Unsupported-Guest And Host-Leakage Review Gap

Goal:

1. make the guest target derivation and fail-closed boundary explicit enough to resolve the Packet 2 review disagreement,
2. prove that unsupported guest tuples fail closed instead of silently widening to host-runtime truth,
3. preserve the separation between host-scoped Codex runtime truth and world-scoped Codex runtime truth on mixed-platform hosts.

Primary touch surface:

1. `crates/shell/src/builtins/world_deps/`
2. `crates/shell/src/execution/agent_runtime/validator.rs`
3. adjacent dispatch/runtime-selection tests
4. Slice 59 docs and prompt/task wording only where the new remediation gate must be explicit

Required changes:

1. make the world-runtime guest target derivation explicit and bounded to actual guest posture,
2. fail closed with stable unsupported-guest diagnostics when UAA has no validated support for the requested guest tuple,
3. add regression proof that host Codex presence on `PATH` cannot satisfy the world-runtime contract,
4. make the host-vs-world Codex runtime separation explicit enough that mixed-platform hosts are not misread as using one interchangeable binary.

Verification checkpoint:

1. unsupported guest tuples fail closed before the world runtime is treated as installed or launchable,
2. host Codex on `PATH` does not make world Codex runtime truth pass,
3. supported guest tuples remain green,
4. Packet 2’s remaining review finding is resolved without widening into installer or Slice 58 work.

### Phase 3: Add Installer-Time Runtime Provisioning And Doc It

Goal:

1. expose the new guest-runtime provisioning path through both prod and dev install flows,
2. keep the operator sequence explicit,
3. align docs/help with actual behavior.

Primary touch surface:

1. `scripts/substrate/install-substrate.sh`
2. `scripts/substrate/install.sh`
3. `scripts/substrate/dev-install-substrate.sh`
4. `scripts/substrate/world-enable.sh`
5. `docs/INSTALLATION.md`

Required changes:

1. add the generic installer flag shape `--provision-agent-runtime <runtime_family>` on prod install surfaces,
2. add the same public flag shape on dev install surfaces,
3. implement only the `codex` value in this slice and fail closed for unsupported runtime-family values,
4. make the flag provision/install the selected runtime as needed and then run `substrate world deps current sync`,
5. update help/docs/examples to match.

Verification checkpoint:

1. both surfaces advertise the new flag,
2. docs state that the flag provisions/install as needed and then runs sync,
3. operator truth is consistent across dev and prod.

### Phase 4: Prove The Runtime Path End-To-End And Stop Before Slice 58

Goal:

1. prove the world runtime is now semantically truthful,
2. preserve a clean handoff to Slice 58 placement-aware migration.

Verification wall:

1. validator/runtime tests are green,
2. world-deps tests are green,
3. installer parsing/help/docs are current,
4. manual/smoke proof shows guest runtime path use,
5. no placement-aware selector migration has been pulled into this slice.

Exit criteria:

1. fake world launchability is gone,
2. guest runtime delivery exists and is documented,
3. installers can provision it,
4. Slice 58 can follow without reopening runtime semantics.

## Sequencing And Parallelism

1. Phase 1 must land before any package delivery proof can be honest.
2. Phase 2 must land before installer flags can provision anything real.
3. Phase 2.5 must land before installer support can honestly claim review-clean world-runtime semantics.
4. Phase 3 must land before final operator-facing verification is complete.
5. Slice 58 implementation remains blocked on Phase 4 being green.

Limited parallelism that is acceptable:

1. artifact-source research and package-script drafting may happen while validator contract wording is being finalized,
2. UAA integration for validated version resolution can be developed while package authoring is underway,
3. broader UAA target-support research can happen in parallel with Packet 2.5 remediation design,
4. but code landing remains sequential because the runtime truth wall and remediation gate should exist before package and installer work claim success.

## Risks

### Risk 1: Validator and dispatch disagree about runtime truth

If validator rejects one runtime posture but world dispatch still serializes a different truth source, the repo will stay semantically inconsistent.

Mitigation:

1. keep one authoritative runtime-realizability contract,
2. add tests that exercise both validation and dispatch materialization.

### Risk 2: The package installs a binary that is not actually self-contained

If the package assumes the binary is standalone but the guest still needs Node or other pieces, the new path will fail later in a different way.

Mitigation:

1. treat self-contained status as a required verification seam,
2. widen into a runtime bundle immediately if verification disproves the assumption.

### Risk 3: Installer flags imply the wrong mental model

If install-time provisioning appears to make the runtime ready but still silently requires a later sync step, operator UX will drift from actual repo truth.

Mitigation:

1. make the flag include sync and write that behavior explicitly into help/docs,
2. test help text and examples, not just implementation behavior.

### Risk 4: Mixed-platform hosts blur host and guest Codex runtime truth

If macOS/Lima or another mixed-platform posture is treated as “Codex already exists on the host, so world Codex is effectively supported,” review and operator truth will drift away from the fail-closed slice contract.

Mitigation:

1. keep host and world runtime truth explicitly separate in code, tests, and docs,
2. fail closed on unsupported guest tuples even when host Codex is present,
3. treat any broader guest-target support expansion as a deliberate published-support decision, not an implicit local fallback.

### Risk 5: Slice 59 drifts into Slice 58

If this slice starts renaming backends or changing selector surfaces, it will mix runtime truth with config-shape migration and make rollback/review harder.

Mitigation:

1. keep all exact-id migration out of this slice,
2. refer to the target generically as the world-scoped Codex backend when needed.

## Non-Goals

1. Implementing the placement-aware inventory/selector cutover from Slice 58.
2. Renaming `cli:codex_world` to `cli:codex-world` in production code as part of this slice.
3. Generalizing a new artifact transport system for all world runtimes.
4. Widening public selector ergonomics or logical-agent shorthand.
5. Reopening world-binding, transport, or `SPEC-30` host-vs-world semantics.
