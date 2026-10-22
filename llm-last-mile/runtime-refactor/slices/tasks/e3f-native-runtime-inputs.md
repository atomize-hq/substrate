# E3-F native runtime inputs prerequisite

**Kind:** task evidence within the existing E3-F control plane.
**Status:** component implementation and local proof recorded; material review is controlled
by `review-control/e3f-native-runtime-inputs-review-cycle.json`. Not E3-F acceptance or closure.

## Context capsule / specification

- **Objective/completion claim:** implement the projection service's native preparation,
  exact live-owner recovery and terminal native cleanup component. Given the original
  ReadyClosed publication/capability, pin the source/artifacts/workspace, create and
  read back one fixed native realization, return its sealed inputs once, and retain
  exact cleanup custody through failure and original-lease release. No V2 admission,
  child launch, Active/Registered, installed or bridge claim follows.
- **Authority:** [projection callable joins](../../contracts/agent-config-projection-v1.md#e3-f-retained-ownership-and-callable-joins),
  native realization/manifest specification in that same contract, its I1/I2 and future
  symbol fence, [gateway cleanup](../../contracts/managed-gateway-adoption-v1.md#e3-f-retained-successor-and-cleanup-joins),
  and [development review contract](../../contracts/development-review-and-remediation-contract.md).
  The explicit 2026-10-21 user invocation authorizes this single continuation.
- **Baseline:** Linux x86_64, Rust/cargo 1.89.0; clean branch
  `feat/runtime-refactor-e3-f-retained-codex-implementation`, HEAD
  `3eccd8c39c6a08e2d2365542f9b0834110c5938e`, tree
  `635b5bb577d8c51cbff2e618d80aeff983449e50`; upstream and observed remote tip equal.
  Checkpoint `eeb17d952` remains an ancestor with parent `c7e08c3a3`, original tree
  `8b9ecfbac594d403e7c8eb38cb9357e4006ac8ce` and full-index binary delta SHA-256
  `efcd97c433e33bc8a5182d7fc4a8f7db4a1946625dfeed3ab252291e6785c8c5`.
  All eight checkpoint paths match HEAD in object ID and native mode.
- **Current semantic status:** immutable source rendering/publication and original
  retained-lineage guards exist; native runtime inputs/realization/recovery do not.
  `take_for_v2` exists but has no production caller. Service V2 remains rejected;
  manager injection, tagged ownership, adapter, accounting, final gate and production
  registration remain absent/incomplete. Component existence is not runtime adoption.
- **Source/test fence:** `crates/config-projection/src/lib.rs` adds only the specified
  realization manifest; `service.rs` adds the two sealed input types, private publication
  progress and cataloged `prepare_codex_launch` / `revoke_retained_runtime` with necessary
  private validation; `registry.rs` adds `realize_native_root`,
  `recover_native_realization` and necessary private descriptor/recovery/cleanup helpers.
  Existing publication construction initializes progress. Directly necessary colocated
  tests only. This evidence, raw review and review-cycle evidence complete the fence.
- **Transitions:** Unprepared -> publication-owned partial inputs/tree -> validated
  realization -> exactly one returned input owner. Errors preserve original resources,
  names, timestamp and progress. Equal pre-transfer retry reconciles only that attempt;
  no second successful transfer. Cancellation is terminal cleanup, never re-preparation.
  Terminal proof/Revoked boundary -> frozen release timestamp -> exact original Released
  lease -> native deletion -> complete; post-release errors permit same-Released cleanup
  only. Restart is registry/OS recovery, never Active/root adoption. The gateway retains
  all namespace/cgroup/exclusion custody; no namespace/group is removed here.
- **Trust boundary:** accepted authority and exact descriptor observations determine
  output. Caller paths, mutable runtime contents and a successful process are not authority.
  Reject wrong store/series/fence/lease/E2/source/artifact, symlinks, mount escapes,
  substituted inode/owner/mode/hash, extra entries, incomplete/conflicting trees and
  premature cleanup. No credentials/config copying from the host.
- **Non-goals:** all shell/REPL/bridge, B01/B03, actual V2 take/launch, Active publication,
  wrapper, adapter, I1/I2 implementation, new registry/supervisor/schema, D1-D3/E4,
  other agents/platforms, deployments, persistent-service changes and successors.
- **Stop conditions:** required proof/facility unavailable; inconsistent authority;
  required out-of-fence changes; custody/identity cannot be proved; bounded review cannot
  become CLEAN. Preserve failures; never weaken tests or activate V2 to manufacture proof.

## Dependency mapping and sequence

| Operation | Owner / present source and proof | Remaining work / blocking effect |
|---|---|---|
| Configured service/registry/exclusion injection | `WorldService::new_linux`, preparation manager share configured objects; member manager receives exclusion only | Pair injection and fail-closed pre-transfer validation later; blocks V2, not this component |
| One-time transfer | Preparation manager `take_for_v2`; E3-E custody retained unchanged | Production caller later; never edit here |
| Required ownership tag | I1 contract; existing common non-E3 fields remain | Tagged retained/provisional/cleanup owner later; blocks admission |
| Native inputs | Current source manifest and retained authority validator / original Held guard exist | **Selected component**: realization, pinning, one transfer, cleanup/retry |
| Local retained Codex adapter | Cataloged, absent | Consume this component, actual child/turn/event/cancel custody later |
| Gateway accounting/denial/cleanup | E3-E owner exists; I2 retained Codex history and early-denial joins absent | Exact original accounting, all historical sets, namespace-before-cgroup, exclusion-last later |
| Wrapper setup/final gate | E3-D primitive exists; Codex loader/fingerprint branch incomplete | Descriptor/mount/loader checks and final release later |
| Active/Registered production order | Contract bound, production route absent | Real admission/launch/cleanup readiness before enabling V2; bridge proof only afterwards |

Ordered work: this native component; retained adapter with wrapper and gateway accounting
dependencies; coherent tagged ownership/admission/cleanup integration; actual service production
proof; separately selected producer/bridge and integrated proof. Later packets may refine their
fences; this is not automatic successor authorization.

## Greenfield mechanism decisions

| Mechanism | Required capability / owner | Independent purpose or disposition | Prerequisite / proof |
|---|---|---|---|
| Accepted-home immutable source and original Held lease | E3 projection authority | Retain: source integrity and exact one-attempt ownership, independent of compatibility | Existing lineage/source guards plus new negative/retry cases |
| Separate native realization | E3 native owner | Retain: mutable per-worker state cannot modify authority; descriptor identities differ from source | Creation, exact readback, substitution and cleanup tests |
| Same-publication progress / sealed inputs | E3 retained joins | Retain: original resource custody across partial failure and one-time transfer | Duplicate transfer and failure/ACK tests |
| Root-owned fixed runtime parent / no-follow descriptor operations | E3 Linux security | Retain: deny caller-selected output and path substitution | Isolated component filesystem proof; installed runtime parent is a later facility |
| Secure credential-FD gateway, exclusion, historical process accounting | Gateway/E3-D/I2 | Consume unchanged prerequisite; no credentials or kernel owners enter this component | Existing E3-E assertions retained; this component does not prove Codex adoption |
| V1/UAA launcher, copied host auth/config and fallback | Runtime-family / E3 gateway adoption | No retention by this packet; out-of-fence replacement/removal debt. None may implement or satisfy the new component | Later purpose-selected runtime packet; no compatibility fixture dictates this design |

## Tasks, proof and review fence

1. Add behavior-first colocated tests for creation, exact retry, failures, substitutions,
   one-time transfer and cleanup ordering. Confirm RED against absent operations.
2. Implement native component in small compiling increments. Reuse registry descriptor
   helpers and canonical encoding. Rust 2021, four spaces; typed failures leave owners borrowed.
3. Verify component cases, existing config-projection suites and service rejection regression.
   Run `cargo +1.89.0 check -p world-service --locked`,
   `cargo +1.89.0 test -p config-projection --lib --locked`,
   `cargo +1.89.0 test -p config-projection --test agent_config_projection_v1 --locked`,
   and applicable world-service library/activation tests. Run package clippy with warnings
   denied, scoped rustfmt and `git diff --check`. Bind actual new selectors before claiming proof.
   Existing explicitly ignored privileged tests remain unavailable evidence, never silent passes.
4. Mechanical scope/format checks before SHA-256 material-subject discovery fingerprint.
   Fresh read-only built-in default `gpt-6.1-sol`, High (user override), default speed.
   One discovery/burst, one consolidated P1/P2 remediation, different-fresh closure;
   at most two supplemental causal cycles; CLEAN terminal. Machine record under existing
   `review-control`; no duplicate closeout reviewer. GitNexus detect-changes before local commit.

Pre-edit review `/root/e3f_prerequisite_review` completed the prerequisite/fence assessment.
Its subject was the packet proposal, not the material candidate. GitNexus publication type,
constructor and bind impacts were LOW with zero indexed direct callers/flows (known incomplete
graph coverage). Source inspection and retained E3 regressions supplement that result.

Baseline proof: `cargo +1.89.0 check -p world-service --locked` exit 0 (21 inherited shell
warnings); config-projection lib exit 0: 60 passed, 0 failed, 5 explicitly ignored.
Raw baseline log: `/tmp/e3f-3eccd8c-baseline-config-projection-lib.log`.

## Component proof and claim limits

Raw outputs are retained at
`/home/spenser/__Active_code/review-evidence/e3f-native-runtime-inputs-20261022/`.
`final-verification.json` and `final-world-*.log` retain discovery-candidate proof.
`closure-verification.json` binds exact commands, logs and exits after consolidated remediation.

| Command | Result |
|---|---|
| `cargo +1.89.0 test -p config-projection --lib --locked` | Discovery: exit 0, 64 passed/5 ignored. Remediated: exit 0, 65 passed/6 ignored; the new root selector is separately executed below |
| `sudo -n target/debug/deps/config_projection-32ce302862a984ee --exact registry::linux::tests::test_e3f_native_creator_owned_root_failure_retains_custody --ignored --nocapture` | Exit 0; 1 passed, 0 ignored; actual root-created temporary objects, no process identity switching |
| `cargo +1.89.0 test -p config-projection --test agent_config_projection_v1 --locked` | Exit 0; 6 passed |
| `cargo +1.89.0 test -p config-projection --doc --locked` | Exit 0; 1 passed |
| `cargo +1.89.0 clippy -p config-projection --all-targets --locked -- -D warnings` | Exit 0 |
| `cargo +1.89.0 check -p world-service --locked` | Exit 0; inherited shell warnings retained |
| `cargo +1.89.0 test -p world-service --lib --locked e3a_v2_execute_paths_fail_closed_before_world_budget_or_launch_effects` | Exit 0; 1 passed, 228 filtered |
| `cargo +1.89.0 test -p world-service --test e3_config_projection_activation_v1 --locked` | Exit 0; **0 run, 3 ignored: unavailable proof**, not an activation/security pass |
| Scoped `rustfmt +1.89.0 --edition 2021 --check` on the three source files; `git diff --check` | Exit 0 |

New actual selectors (all executed; the explicit root case runs separately with `--ignored`):

- `test_e3f_native_realization_exact_retry_and_substitution`
- `test_e3f_native_realization_interrupted_publication_and_cleanup`
- `test_e3f_native_realization_conflict_has_no_adoption_or_repair`
- `test_e3f_native_cleanup_original_lease_ack_and_partial_deletion`
- `test_e3f_native_cancel_temporary_removal_ack_retry`
- `test_e3f_native_creator_owned_root_failure_retains_custody`

The first RED run failed because the native operations did not exist. The matrix covers
creation/ownership, completed-write return loss, both sides of rename (including completed
rename before the in-memory success marker), exact retry with frozen timestamp, equal-byte
inode substitution, extra/symlink entries, no cold root adoption, one-time input transfer,
pre- and post-transfer cleanup, rejection before terminal/Revoked evidence, missing original
inputs, lease-head ACK loss, release-before-delete failure, partial deletion/root-removal
return loss, same-Released retry and accepted-home source preservation. Test descriptors for
custody are explicitly non-executable; no test invokes or impersonates Codex.

The same-subject discovery burst used `/root/e3f_discovery_custody` and
`/root/e3f_discovery_authority` (requested model `gpt-6.1-sol`, High, default speed).
The consolidated findings were `P2-CUSTODY-1` (lost removal return on a never-installed temporary)
and parent-validated `P2-PARENT-1` (root descriptor lost before UID/GID setup returns).
Both have retained failing reproductions, then passing guards. The single remediation moves
completed-removal readback ahead of rename reconciliation and retains the original pending-root
FD/creator ownership before fallible ownership setup. It quarantines an occupied name when no
original FD was retained; it never adopts that name. The actual root test verifies both equal
retry to intended UID/GID and cancellation of a still creator-owned inode, including lost removal
return. Only temporary filesystem objects are changed. Final verdict/subject lineage is in the
linked review-cycle record; raw reviews and RED/GREEN outputs remain separate in the evidence root.

These are source/component filesystem and custody tests. They do not positively exercise the
installed `prepare_codex_launch` path with real pinned artifacts, actual Codex, mount setup,
gateway kernel denial, service admission or adapter integration. Existing environment-conditional
artifact tests in the inherited suite are not counted as real-artifact proof. On this host the
fixed runtime parent and runtime artifact source store were observed absent; `/etc/codex` and
installed accepted-home bootstrap authority exist. No installation or persistent-service change
was made. Those missing facilities block dependent installed claims, not isolated component tests.

Production `execute_stream` still rejects V2. I1/I2 source integration, Active/Registered order,
real initial plus two resumed turns, bridge proof and every unproved F1–F14 obligation remain
with the existing E3-F owner. E3-E stays closed. The next eligible work is separately selected
retained adapter/wrapper/accounting work, after its own dependency and proof binding.
