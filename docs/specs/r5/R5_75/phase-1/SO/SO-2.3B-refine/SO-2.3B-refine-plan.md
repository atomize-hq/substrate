# Plan: SO-2.3B-refine Structured Objective Bridge Honesty

Status: PLAN artifact created on 2026-06-17 after reconciling the live `R5.75-1` code path, the
revised `R5.75` map, and the structured-objective design authorities. This packet exists to close a
small but important honesty gap before `SO-3` compatibility/comparison work and before `SO-4`
acceptance harness work.

## Objective

Land a bounded corrective packet that:

1. preserves the structured-objective sidecar through checkpoint narrowing,
2. prevents `target` from being fabricated from weak goal clauses,
3. aligns verification-role evidence with verification-command extraction,
4. proves each corrected behavior in the same packet that changes it,
5. uses the final audit packet only for missing combined-case coverage or proof gaps.

## Planning Decisions Locked For This Packet

1. This packet remains inside `R5.75-1`; it is not `R5.75-2`, not a new phase family, and not a
   downstream migration packet.
2. `SO-2.3B-refine` must land before `SO-3`. Compatibility rendering and `comparison_key`
   derivation should rest on preserved, honest structured state.
3. `SO-2.3B-refine` must also land before `SO-4`. The acceptance harness should validate corrected
   semantics rather than silently codify the current bridge/target/verification gaps.
4. Scope stays analyzer-local: `checkpoint/mod.rs`, `context/objective.rs`, and
   `tests/checkpoints.rs` are the intended seams.
5. Unknowns are success. If explicit target evidence is absent, the packet should preserve a real
   goal with `target == None`, not invent a conceptual target.
6. Because the packet prompt process commits each behavior packet before review, the core regression
   for that behavior should land in the same packet rather than being deferred to a later omnibus
   regression packet.
7. The packet should avoid widening into compatibility rendering policy, `comparison_key`
   derivation, fixture-family work, or downstream consumers unless a tiny compile-safe bridge is
   unavoidable.

## Why This Packet Exists Before SO-3 And SO-4

The live crate is already materially past the old string-only stopgap: structured extraction,
section/clause decomposition, and grounding identifiers exist. But the checkpoint path still loses
that richer state by overwriting `context.objective` with a compatibility-only summary, while the
structured assembly still overclaims `target` from weak goal clauses and under-locks verification
role grounding.

That creates a bad dependency order if `SO-3` or `SO-4` starts first:

- `SO-3` would render compatibility text and derive `comparison_key` on top of a bridge that still
  erases the current semantic authority.
- `SO-4` would either encode known-bad behavior into the acceptance wall or remain too skeletal to
  guard the semantics that actually matter.

It also creates a packet-mechanics problem if the proof is deferred to a later omnibus regression
packet: the review for the earlier behavior packet is forced to judge intent rather than proof.
This plan removes that mismatch by pairing each behavior packet with its own proving regression.

## Dependency Graph

```text
packet docs lock
  -> B1.1 preserve structured state through checkpoint narrowing + prove it
  -> B2.1 tighten vague-target extraction + prove unknown behavior
  -> B2.2 preserve explicit concrete targets + prove the matrix
  -> B3.1 align unheaded verifier role grounding + prove it
  -> B3.2 prefer clause-grounded verifier extraction + prove it
  -> B4.1 audit final proof surface + add any missing combined-case regressions
  -> rerun targeted + full analyzer wall
  -> hand off to SO-3.1 / SO-3.2

explicitly deferred:
  -> compatibility rendering from structured state
  -> deterministic comparison_key derivation
  -> objective_acceptance harness and fixtures
  -> TaskFrame / working_set / checkpoint/progress migration
  -> classifier/runtime experiments
```

## Recommended Landing Sequence

## B0: Docs Lock (This SPEC / PLAN / TASKS)

### Scope

- create the packet-local `SPEC` / `PLAN` / `TASKS` set
- make the packet boundary and ordering explicit before implementation begins
- keep the session doc-only until these artifacts exist

### Verification

Manual review only.

## B1: Preserve Structured Objective Through Checkpoint Narrowing

### Scope

- update `narrowed_objective_summary(...)` so it does not reduce a richer structured summary to
  `ObjectiveSummary::compatibility(...)`
- keep the current checkpoint narrowing behavior only as a fallback when structured extraction truly
  cannot provide structured state
- preserve evidence-bearing semantics (`structured`, `verification_commands`, `unknowns`, and the
  relevant evidence sources) through the bridge
- add the focused regression in the same packet proving structured state survives checkpoint narrowing

### Primary Files

```text
crates/agent-drift-analyzer/src/checkpoint/mod.rs
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why First

This is the highest-priority semantic defect because the richer sidecar already exists but is still
being erased in the main checkpoint path.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## B2: Stop Target Fabrication From Weak Goal Clauses

### B2.1 Scope

- split goal selection from target extraction
- add a dedicated target-evidence gate so `ObjectiveTarget` is emitted only when explicit anchors
  exist (file path, crate/package, spec/doc, test/verifier, instruction surface, workspace ref,
  or similarly concrete evidence)
- keep vague review/analyze/fix asks grounded as goals while leaving `target` unknown
- add the vague-target regression in the same packet so the unknown behavior is reviewable and proven

### B2.1 Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### B2.2 Scope

- preserve explicit concrete targets after the unknown gate is tightened
- add the explicit-target preservation regression matrix in the same packet so the tightening cannot
  silently regress file/instruction-surface/spec-doc/test/workspace targets

### B2.2 Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why After B1

The checkpoint path should preserve whatever the structured assembly decides. Once that bridge is
honest, tightening target assembly becomes durable rather than locally correct but globally erased.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## B3: Align Verification-Role Grounding With Verification Extraction

### B3.1 Scope

- ensure command-like verifier clauses can be recognized as verification-bearing even without an
  explicit `Verification` heading
- stop depending exclusively on `top_role(...) == Verification` where that loses meaningful role
  evidence
- add the bullet-only / inline verifier-role regression in the same packet

### B3.1 Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### B3.2 Scope

- prefer clause-grounded extraction to candidate-row fallback whenever a verification-bearing clause
  exists
- add the clause-grounded extraction regression in the same packet so the preference is proven when
  the behavior lands

### B3.2 Primary Files

```text
crates/agent-drift-analyzer/src/context/objective.rs
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why After B2

This keeps the structured assembly honest across both target and verification semantics while
ensuring each committed behavior packet already carries its own proof before review.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## B4: Final Coverage Audit And Combined-Case Regression Sweep

### Scope

- audit the proof surface after B1-B3 land
- identify any missing combined-case regressions or residual packet-proof gaps
- add only the missing regressions needed to make the overall packet family fully reviewable and
  durable
- keep this packet centered on `crates/agent-drift-analyzer/tests/checkpoints.rs` unless a tiny
  packet-scoped fix is required to make the final coverage audit honest

### Primary Files

```text
crates/agent-drift-analyzer/tests/checkpoints.rs
```

### Why After B3

Once each behavior packet already carries its own proof, B4 no longer has to establish the core
behavior claims. It only closes any remaining combined-case or residual coverage gaps.

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
```

## B5: Packet Closeout And SO-3 Handoff

### Scope

- rerun the packet verification wall
- confirm the packet stayed analyzer-local and did not silently widen into `SO-3`, `SO-4`, or
  downstream migration work
- record `SO-3.1` / `SO-3.2` as the next packet boundary after closeout

### Verification

```bash
cargo test -p agent-drift-analyzer checkpoints -- --nocapture
cargo test -p agent-drift-analyzer -- --nocapture
```

## Risks And Mitigations

### Risk: the bridge preserves structured state but changes the visible compatibility text unexpectedly

Mitigation:

- keep this packet focused on preserving structured state first,
- add the bridge-preservation regression in B1.1 so the packet proves its intent before review,
- defer compatibility rendering policy changes to `SO-3`.

### Risk: target extraction becomes too strict and drops legitimate concrete targets

Mitigation:

- pair B2.1 and B2.2 so the unknown gate and explicit-target preservation are both proven,
- treat file paths, crate/package names, spec/doc names, instruction surfaces, verifier targets,
  and workspace refs as accepted explicit anchors.

### Risk: verification grounding stays coupled to a single top-role heuristic

Mitigation:

- allow verification discovery through role-candidate presence rather than only top-role status,
- cover bullet-only / inline verifier role in B3.1 and clause-grounded extraction in B3.2.

### Risk: the packet widens into SO-3/SO-4 work because the files are adjacent

Mitigation:

- keep the packet boundary explicit in the task ledger,
- do not add `objective_acceptance` files,
- do not derive `comparison_key` here,
- do not rewrite compatibility rendering policy here.

## Out Of Scope

- rendering `ObjectiveSummary.text` from structured state
- deriving deterministic `comparison_key`
- adding `tests/objective_acceptance.rs` or fixture families
- migrating `TaskFrame`, `working_set`, `checkpoint/mod.rs` semantic predicates beyond the bridge,
  or `checkpoint/progress.rs`
- classifier/runtime augmentation
