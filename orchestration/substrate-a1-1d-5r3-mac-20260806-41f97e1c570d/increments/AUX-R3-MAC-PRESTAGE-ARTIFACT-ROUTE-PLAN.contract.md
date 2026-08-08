# AUX-R3-MAC-PRESTAGE-ARTIFACT-ROUTE-PLAN contract

## Binding

- Orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- Existing task: `019fdfa4-ba96-70e0-b68d-1d0a4cf32a83` on `local`
- Worktree: `/Users/spensermcconnell/.codex/worktrees/2f78/substrate`
- Dispatch nonce: `2f49b83ec6bc31fd4da0830fad86aa9c165e999f135c48dadb8a23a8c464d4af`
- Exact base/tree: `a06b3fd26ceee4ece7e5bd1481981755eafcedb5` / `f4c396d0e28a43405a9a2b1d49a450d6df545c84`
- Target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- Amendment: `orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/authority-amendments/0045-r3-mac-prestage-artifact-route-plan.json`
- Amendment SHA-256: `sha256:2f88a293e5c0b39865532b09e4ce0165ee81db1cd82555b04bea87e8de759098`

Rebind the exact clean, zero-staged worktree and live remote before analysis. Stop on drift. Never
mutate the protected checkout. This packet is read-only planning: no repository byte may change.

## Validated blocker input

Evidence attempt 9 is terminal and independently validated:

- receipt: `orchestration/.../receipts/EVIDENCE-R3-MAC-IMP-01.attempt-9.blocked.json`, SHA-256 `8205c07fc87ce3dd71cced0db10939941fa273f384e4918604fe68fb3d7feb96`
- artifact: `/Users/spensermcconnell/.codex/evidence/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/EVIDENCE-R3-MAC-IMP-01/12a0f9442772518e2d665f0b7ac3bf26f4ac38a5562ce97bb1eed98cf7eabf3b/artifact.r3-native-evidence.json`, SHA-256 `8aada728476a5fbfe8f833d19388671e82740fa1c1afb6fb0bd4d5fd89ea0d75`
- discovery records: sibling `discovery/006-stage1-source-artifact-route.txt`, `007-installer-to-stageone-contract-contradiction.txt`, and `008-lima-warm-carrier-failclosed-source.txt`.

The external AArch64 `substrate-lifecycle-linux` build passed. Native mutation did not start and
restoration is exact. The current source nevertheless requires five prefix artifacts while the
installer calls `lima-warm.sh` without build evidence, publisher request, or signed Stage-1
authorization. All required prefix paths are absent and the existing cache runs only after Stage-1.

## Required analysis

Load `.agents/skills/using-agent-skills/SKILL.md` first, then apply source-driven-development,
api-and-interface-design, security-and-hardening, planning-and-task-breakdown,
doubt-driven-development, code-review-and-quality, and orchestrate-top-level-tasks. Read the exact
runtime-refactor recovery SPEC/PLAN/TASKS and current contracts before proposing a correction.

Re-derive and report:

1. The exact current call/data order from installer host build/copy/provenance through hidden direct
   `publisher-bootstrap`, signed `LimaStageOneAuthorizationV1`, `stage_one_create`, PM
   finalization, and post-PM guest projection.
2. The exact evidence identities. Current code derives `ExecutorBuildEvidenceV1` from the host
   macOS privileged executor, while the packet text and evidence build a Linux guest executor.
   Define separate, non-overloaded authority for the fixed Linux guest bundle if required.
3. The exact artifact role set and producer for each role. Specifically audit
   `mac.lima.guest-binary(world)`: Cargo metadata currently exposes `world` only as a library.
   Never propose a fabricated copy or alias. If the role is not required, identify every contract,
   role-table, receipt-plan, executor, and test consequence of removing it from the current closed
   manifest while preserving the allowed role universe.
4. One closed typed bundle and staging design: fixed role enum/order, source commit/tree/ref,
   Cargo.lock/toolchain/target/build argv, per-artifact SHA-256 and opened-file physical identity,
   external build-root rules, canonical prefix targets, no-follow descriptor checks, atomic
   all-or-nothing publication, retry convergence, cleanup/restoration, and retained root-owned
   provenance joins.
5. One executable canonical order. It must explain how host installation finishes enough to admit
   direct publisher bootstrap, how the controlling-terminal confirmation occurs, how the returned
   signed Stage-1 authorization reaches only the fixed Stage-1 wrapper, and why no script/caller
   becomes a bootstrap authority.
6. Exact implementation packet: completion claim, changed-path and symbol allowlist, explicitly
   frozen surfaces, TDD RED/GREEN cases, deterministic MAC-only checks, external AArch64 build
   proof, causal cascading review budget, publication gates, and the complete fresh evidence retry
   command sequence.

## Decision standard

Prefer the minimal proper version, not the smallest textual patch. The plan must close the circular
prerequisite rather than bypassing it. It may propose a new closed typed contract only when every
field has one authoritative producer and consumer. It must not authorize generic paths, roles,
selectors, transports, endpoints, principals, or ambient discovery.

Before returning, run three fresh read-only reviews over the proposed plan:

- authority/security;
- lifecycle/convergence and crash/retry ordering;
- allowlist/evidence sufficiency and platform containment.

Resolve review findings in the plan text. Return `PLANNED_CLEAN` only with no unresolved P1/P2.
If a real architecture decision remains, return `BLOCKED_CONTRADICTION` with exactly the minimum
choices requiring meta/user adjudication. Do not edit or publish source and do not dispatch any
successor.

## Receipt

Send one `codex.auxiliary-source-correction-plan-receipt.v1` as the final tool action. Include
binding, exact source observations, chosen artifact role set, chosen authority/staging/order design,
implementation path/symbol fence, test/review/evidence plan, review identities/findings, and a
complete successor implementation prompt. Also include zero repository/native mutations and clean
worktree/remote proof.
