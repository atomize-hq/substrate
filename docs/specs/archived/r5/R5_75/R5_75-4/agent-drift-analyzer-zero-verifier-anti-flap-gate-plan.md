# Plan: Agent Drift Analyzer Zero-Verifier Anti-Flap Gate (R5.75-4)

Status: draft plan created on 2026-06-23 after re-reading the live `R5.75-4` packet in
`docs/specs/r5/R5_75/MAP.md`, the root `structured-objective-bug-map.md`, and the current analyzer
progress seams. This plan is reviewable: it should be possible to read it and say “yes, that packet
boundary” or “no, change X” before any code lands.

## Objective

Land one bounded analyzer-local guard so long exploratory zero-verifier sessions stop escalating into
troubleshooting/dead-end-ish posture without decisive evidence, while preserving both real
verifier-backed troubleshooting signals and the delegated parent-visible stability already landed in
`R5.75-3`.

## Planning Decisions Locked For This Draft

1. The packet boundary is the `session_progress` layer only. The implementation may tune how
   `progress.rs` chooses or caps a progress posture, but it must not retune scorer families, redesign
   `dead_end_thrash`, or change replay/export/schema contracts.
2. Existing evidence signals must drive the gate. The packet should prefer current verifier density,
   concrete edit progress, explicit failure evidence, and delegated-parent visibility cues over adding a
   new telemetry surface.
3. Structured `primary_intent` is explicitly not a live input here. If the fix would require wiring
   structured state into progress selection, stop and route that as a later packet instead of smuggling
   `R5.75-6`-adjacent work into this one.
4. Canonical zero-verifier exploratory witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185` are required smoke gates,
   but adapted committed fixture-family work still belongs to `R5.75-5`. Any committed acceptance proof
   in this packet must stay bounded to existing native/synthetic corpus structure unless a narrowly
   justified packet-local case is required.
5. `R5.75-3` must remain true after this landing. In particular, `da59436e63915185` may stay on a
   conservative `parent_visible_orchestration` bar where delegation markers justify it; the new packet
   is about anti-flap conservatism, not collapsing delegated parent-visible evidence back into generic
   noise.
6. Packet prompts remain out of scope for this pass because the user asked for spec/plan/tasks only.

## Why This Packet Exists

The live map now routes `R5.75-4` as the next pre-`R6` hardening seam because manual adapted evidence
still shows a remaining analyzer-honesty gap: long exploratory sessions with zero verifier density can
look more decisive than the evidence warrants. `R5.75-3` stabilized delegated parent-visible behavior,
but that packet deliberately did not own the broader “keep zero-verifier sessions boring” rule.

This packet exists to close that gap without widening into scorer work or the later structured-native
consumer migration.

## Dependency Graph

```text
docs lock (this SPEC / PLAN / TASKS)
  -> characterize the canonical zero-verifier exploratory witness and the mixed delegated/exploratory smoke witness
  -> land analyzer-local anti-flap guard in progress.rs
  -> add fast checkpoint regressions that prove both conservatism and non-regression
  -> optionally add one bounded progress_acceptance proof if needed
  -> rerun automated gates
  -> rerun adapted smoke and inspect outputs
  -> update MAP only after promotion is honestly earned

explicitly deferred / out of scope:
  -> adapted committed fixture family (`R5.75-5`)
  -> structured-goal consumer wiring (`R5.75-6`)
  -> scorer / dead_end_thrash retuning (`R6`)
```

## Major Components And Dependencies

1. **Exploratory smoke characterization**
   - dependency: packet scope from `docs/specs/r5/R5_75/MAP.md`
   - outcome: the tasks ledger records exact expected lane/status/confidence/evidence behavior for
     `097d97e914ca220f` and `da59436e63915185`

2. **Analyzer-local anti-flap guard**
   - dependency: characterization clarifies the exact overclaim shape
   - outcome: `progress.rs` prefers conservative planning / insufficient-evidence posture when verifier,
     concrete edit, and explicit failure signals are absent

3. **Fast checkpoint regression wall**
   - dependency: the desired conservative boundary is explicit
   - outcome: synthetic and bundle-shaped tests prove the anti-flap guard without regressing real
     troubleshooting or delegated parent-visible semantics

4. **Bounded semantic acceptance proof (only if needed)**
   - dependency: stable packet-local behavior exists and needs a committed proof beyond fast regressions
   - outcome: `progress_acceptance.rs` stays honest without starting the adapted fixture-family packet

5. **Packet closeout validation**
   - dependency: implementation and tests complete
   - outcome: shared analyzer gates plus adapted smoke review prove the packet is ready for promotion to
     `R5.75-5`

## Implementation Order

1. Commit the docs triplet so the packet boundary is explicit before any implementation work begins.
2. Characterize canonical zero-verifier exploratory witness `097d97e914ca220f` and mixed delegated/exploratory witness `da59436e63915185`, recording exact packet-local expectations in
   the tasks ledger before claiming to know what the fix is.
3. Land the minimal analyzer-local anti-flap rule in `crates/agent-drift-analyzer/src/checkpoint/progress.rs`.
4. Add or refresh fast regressions in `crates/agent-drift-analyzer/tests/checkpoints.rs`.
5. Only if the packet would otherwise lack an honest committed semantic proof, add one bounded
   `progress_acceptance` update that stays inside the existing corpus contract.
6. Run the automated gate ladder.
7. Re-run the canonical zero-verifier exploratory smoke witness and mixed delegated/exploratory smoke witness, then inspect `summary.md` plus `checkpoints.jsonl`.
8. Update `docs/specs/r5/R5_75/MAP.md` only after the smoke gate is satisfied and the packet is truly
   promoted.

This order matters because the packet should first define what “boring and conservative” means on the
named repros, then encode that behavior in code/tests, then prove it under smoke before updating the
routing authority.

## Risks And Mitigations

### Risk 1: the anti-flap guard suppresses legitimate troubleshooting

- Risk: a broad conservative cap could erase real verifier-backed or explicit-failure-backed
  troubleshooting progression.
- Mitigation: gate only when decisive signals are absent, and add regression coverage showing that real
  verifier/failure evidence still advances troubleshooting.

### Risk 2: the packet regresses `R5.75-3` delegated parent-visible behavior

- Risk: treating long exploratory sessions conservatively could accidentally collapse legitimate
  parent-visible delegation back into generic planning noise.
- Mitigation: keep `da59436e63915185` as an explicit smoke witness and add regression coverage that
  preserves conservative `parent_visible_orchestration` behavior where delegation evidence exists.

### Risk 3: the packet silently widens into scorer or structured-state work

- Risk: once anti-flap behavior is under review, it becomes tempting to wire in structured intent or
  retune `dead_end_thrash` / broader failure scoring.
- Mitigation: treat any need for structured inputs or scorer redesign as a stop condition that opens a
  later packet rather than widening `R5.75-4`.

### Risk 4: the packet accidentally starts `R5.75-5`

- Risk: adding adapted committed fixtures here could blur the line between behavioral hardening and the
  adapted robustness-family packet.
- Mitigation: adapted sessions stay smoke-only unless a narrowly justified packet-local proof is truly
  required, and even then the committed corpus must remain bounded and honest.

## Verification Checkpoints

### Checkpoint A: Smoke characterization is explicit

Pass when:

- the tasks ledger records exact expected lane/status/confidence/evidence behavior for
  `097d97e914ca220f` and `da59436e63915185`
- the ledger explicitly preserves `R5.75-3` ownership of the delegated parent-visible bar for
  `da59436e63915185`

### Checkpoint B: The anti-flap rule lands

Pass when:

- long zero-verifier exploratory intervals default to conservative planning / insufficient-evidence
  behavior when decisive signals are absent
- real verifier/failure-driven troubleshooting still advances when the evidence warrants it

### Checkpoint C: Fast regressions prove both sides of the boundary

Pass when:

- broad scans / read-output / tool-output-heavy exploratory shapes stay low-confidence and conservative
- verifier-backed troubleshooting still escalates appropriately
- delegated parent-visible shapes do not regress

### Checkpoint D: Any semantic acceptance update stays bounded

Pass when either:

- no `progress_acceptance` update was needed, or
- one bounded packet-local acceptance proof landed without admitting adapted committed fixtures or
  changing corpus shape dishonestly

### Checkpoint E: Automated gates are green

Pass when:

- `cargo test -p agent-drift-analyzer checkpoints -- --nocapture` is green
- `cargo test -p agent-drift-analyzer --test progress_acceptance -- --nocapture` is green
- `cargo test -p agent-drift-analyzer -- --nocapture` is green

### Checkpoint F: Adapted smoke proves live correctness

Pass when:

- `097d97e914ca220f` stays boring/conservative and does not surface troubleshooting-frontier or similar
  strong failure posture without decisive evidence
- `da59436e63915185` preserves the conservative delegated parent-visible bar from `R5.75-3` while no
  longer flapping into unrelated stronger failure posture elsewhere in the run

### Checkpoint G: Routing authority is updated honestly

Pass when:

- `docs/specs/r5/R5_75/MAP.md` marks `R5.75-4` promoted and `R5.75-5` active only after Checkpoints
  A-F are complete

## Parallelism And Sequencing

- **Must stay sequential**
  - characterization before final acceptance wording
  - anti-flap guard before final smoke verdict
  - automated gates before routing update
  - routing update after manual smoke, not before
- **Can happen together inside one edit cycle**
  - `progress.rs` anti-flap implementation and fast checkpoint regression updates
  - bounded `progress_acceptance` adjustments with their matching corpus-contract assertions, if needed

## Packet Split

`R5.75-4` remains one packet with six internal work blocks:

1. `R5.75-4A` — docs lock + exploratory smoke characterization
2. `R5.75-4B` — analyzer-local anti-flap guard in `progress.rs`
3. `R5.75-4C` — fast checkpoint regressions
4. `R5.75-4D` — bounded semantic acceptance proof (only if needed)
5. `R5.75-4E` — automated gate ladder + adapted smoke review
6. `R5.75-4F` — MAP promotion / routing update
