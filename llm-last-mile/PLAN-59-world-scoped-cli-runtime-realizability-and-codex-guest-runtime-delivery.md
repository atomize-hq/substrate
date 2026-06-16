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
3. prod/dev installer-time provisioning support,
4. and end-to-end proof that the world runtime is guest-visible and not inherited from host NVM/npm state.

## Plan Summary

The repo’s current semantic bug is not selector naming; it is fake world launchability truth.

So the correct sequence is:

1. freeze the runtime-realizability contract,
2. land the fail-closed validator/materialization wall,
3. land the world-deps Codex package or runtime bundle,
4. land installer support and docs,
5. prove the path end-to-end,
6. only after that, let Slice 58 change the config/selector shape.

Slice 59 therefore **precedes** Slice 58 implementation even though Slice 58 is already specified.

## Locked Decisions

### What changes

1. World-scoped CLI launchability becomes guest-visible truth instead of host `which` truth.
2. Missing guest runtime becomes an early validator/materialization error with remediation.
3. Codex guest runtime delivery uses a Substrate-owned world-deps script package that pulls official release artifacts.
4. Prod and dev installers gain the generic public flag shape `--provision-agent-runtime <runtime_family>`, even though this slice only implements the `codex` value.

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
3. prove whether the Linux release binary is self-contained or whether the package must widen into a runtime bundle.

Primary touch surface:

1. world-deps inventory/package definitions under Substrate-managed inventory,
2. package install scripts,
3. world-deps inventory/surface tests,
4. operator docs for authoring/runtime flows.

Required changes:

1. define the Codex package or bundle name,
2. install under `/var/lib/substrate/world-deps/<package>`,
3. expose `/var/lib/substrate/world-deps/bin/codex`,
4. fetch official release artifacts from the chosen source,
5. document the verified self-contained-vs-bundle outcome.

Verification checkpoint:

1. the package/bundle installs idempotently,
2. the guest-visible `codex` entrypoint resolves from the world-deps bin prefix,
3. the implementation records which runtime-dependency posture was proven.

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
3. Phase 3 must land before final operator-facing verification is complete.
4. Slice 58 implementation remains blocked on Phase 4 being green.

Limited parallelism that is acceptable:

1. artifact-source research and package-script drafting may happen while validator contract wording is being finalized,
2. prod/dev installer flag naming can be designed while package authoring is underway,
3. but code landing remains sequential because the runtime truth wall should exist before package and installer work claim success.

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

### Risk 4: Slice 59 drifts into Slice 58

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
