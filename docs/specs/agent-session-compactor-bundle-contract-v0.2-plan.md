# Plan: Agent Session Compactor Bundle Contract v0.2

## Scope

This plan implements
[agent-session-compactor-bundle-contract-v0.2-spec.md](/Users/spensermcconnell/.codex/worktrees/97a0/substrate/docs/specs/agent-session-compactor-bundle-contract-v0.2-spec.md:1).

The goal is to replace accidental path-heavy row serialization with an explicit `v0.2` bundle
contract that keeps repeated `source_file` strings out of every exported row while preserving the
current analyzer semantics.

This slice should:

- keep the existing five-file compactor bundle shape
- replace per-row and per-ref path duplication with compact stable file ids
- introduce export-facing DTOs instead of serializing internal compactor structs directly
- cut `agent-drift-analyzer` over to the new contract in the same bounded slice
- preserve operator-readable analyzer outputs by resolving file ids back to paths at the input
  boundary

This slice should not:

- redesign normalization, dedupe semantics, or checkpoint scoring
- add compression or a sixth artifact file such as `files.json`
- force downstream sentinel work to learn a second temporary bundle contract
- widen analyzer checkpoint artifacts unless the `v0.2` loader proves that an existing invariant
  is actually missing

## Implementation Strategy

Build the cutover around the on-disk contract boundary rather than around internal row types.

Implementation order:

1. lock the `v0.2` contract decisions that are still open in the spec
2. add export-only DTOs and a deterministic manifest-owned file registry
3. migrate row and dedupe-audit emission to id-backed DTOs
4. cut analyzer input over by resolving ids into the existing path-backed internal types
5. re-run deterministic tests and bounded smoke against the full compactor-to-analyzer path

This ordering keeps the internal compactor/analyzer logic as stable as possible while replacing the
on-disk seam in one direct cutover.

## Contract Decisions Locked For This Slice

- the bundle-level file table lives in `manifest.json`; do not add a separate `files.json`
- the bundle remains a five-file export: `manifest.json`, `rows.archival.jsonl`,
  `rows.compact.jsonl`, `dedupe-audit.jsonl`, and `summary.md`
- file ids use `u32`
- the manifest file registry is derived from the paths actually referenced by exported rows and
  dedupe refs, ordered lexicographically by path
- internal `CompactionRow` and `dedupe::RowRef` remain path-backed in this slice; only export DTOs
  and the analyzer input boundary change
- analyzer checkpoint and evidence outputs continue to render resolved file paths for operator
  readability
- analyzer input does not guess across mixed contracts; compactor export and analyzer load logic
  cut over together to `v0.2`

## Major Components

### 1. Manifest And File Registry Contract

Deliver first:

- `BundleManifest` upgraded to `schema_version = "v0.2"`
- explicit manifest-owned file entries such as `BundleFileV0_2 { id, path }`
- replacement of the old manifest `source_files` list with the explicit file registry
- validation that every exported row and dedupe ref resolves through the manifest registry

Why first:

- row and dedupe DTOs need a settled file-id registry before their serialized form can be defined
- this is the narrowest place to lock the `manifest.json` versus `files.json` decision

### 2. Export DTO Boundary

Deliver second:

- `ExportRowV0_2` for archival and compact row JSONL
- `RowRefV0_2` plus any dedupe-audit export wrapper needed to keep refs explicit
- conversion helpers from internal `CompactionRow` and `DedupeGroup` into export DTOs

Critical rule:

- no export file should rely on deriving its schema from `Serialize` on internal row structs

### 3. Compactor Bundle Writer Migration

Deliver third:

- deterministic path-to-id registry assembly from exported rows and dedupe refs
- `rows.archival.jsonl` and `rows.compact.jsonl` emission with `source_file_id`
- `dedupe-audit.jsonl` emission with id-backed representative and duplicate refs
- unchanged atomic publish behavior with `manifest.json` still written last

Why third:

- this is where the actual payload shrink happens
- the prior artifact-finalization hardening should remain intact while the row contract changes

### 4. Analyzer Input Cutover

Deliver fourth:

- input-side `v0.2` DTO parsing
- manifest file-registry loading and id-to-path resolution
- reconstruction of existing internal `CompactionRow` and `RowRef` values before session grouping,
  sorting, surface validation, inference, and checkpoint export
- clear failure for unknown file ids, duplicate file ids, or incomplete registry state

Why fourth:

- most analyzer code already assumes path-backed rows and refs
- resolving ids at load time keeps the blast radius bounded to the input seam

### 5. Regression And Smoke Validation

Deliver fifth:

- compactor export tests asserting that row JSONL no longer contains `source_file`
- analyzer contract tests asserting successful `v0.2` load and explicit failure on bad ids
- bounded real-session compactor/analyzer smoke showing row-count stability and measurable bundle
  size reduction

Why fifth:

- the contract change is only worthwhile if the downstream analyzer behavior stays semantically
  stable

## Sequencing

Sequential work:

1. lock contract decisions
2. manifest/file-registry DTOs
3. row export migration
4. dedupe-audit export migration
5. analyzer input cutover
6. regression coverage
7. bounded smoke and evidence capture

Parallel-safe work after the manifest/file registry semantics stabilize:

- compactor export tests can proceed in parallel with analyzer input fixture updates
- bounded smoke scripting or doc updates can proceed in parallel with negative contract tests

## Risks And Mitigations

### Risk 1: File registry and row refs drift apart

Risk:

- rows or dedupe refs may serialize ids that do not exist in the manifest registry, or the
  registry may contain duplicates that destabilize lookup

Mitigation:

- derive the registry from the exported row/ref set rather than from a looser discovery list
- add validation on both export and input paths
- make unknown ids a hard error

### Risk 2: The cutover leaks into analyzer logic far beyond input loading

Risk:

- path-backed assumptions in sorting, evidence rendering, or checkpoint export could force wider
  analyzer changes than the spec intends

Mitigation:

- resolve ids into existing path-backed internal types immediately after parsing
- keep downstream analyzer modules unchanged unless a test proves a real contract gap

### Risk 3: Dedupe audit loses stable evidence identity

Risk:

- compacting file provenance could accidentally weaken representative/duplicate row references

Mitigation:

- keep `line_number`, `event_index`, and `row_ordinal` unchanged
- verify that all dedupe refs still point to archival rows that exist after the export migration

### Risk 4: The repo ends up carrying dual bundle contracts longer than intended

Risk:

- temporary `v0.1`/`v0.2` coexistence in analyzer input could become an unowned long-term
  compatibility burden

Mitigation:

- land compactor export and analyzer input in the same slice
- update fixtures and bounded smoke to the new contract immediately
- reject partial or ambiguous mixed-format bundles instead of guessing

## Verification Checkpoints

### Checkpoint BC-A: Contract Decisions Are Locked

Verify:

- the doc chain names one concrete `v0.2` contract shape rather than leaving the file-table and id
  decisions open

Evidence:

- spec + plan + tasks + implementation-order alignment

### Checkpoint BC-B: Compactor Export Is Deterministic

Verify:

- repeated runs emit the same manifest file table ordering and the same id-backed row/audit output

Evidence:

- `cargo test -p agent-session-compactor export_bundle -- --nocapture`
- `cargo test -p agent-session-compactor dedupe -- --nocapture`

### Checkpoint BC-C: Analyzer Resolution Preserves Semantics

Verify:

- analyzer input accepts the `v0.2` bundle, resolves ids early, and does not force downstream
  scoring or checkpoint logic changes beyond expected fixture updates

Evidence:

- `cargo test -p agent-drift-analyzer input_contract -- --nocapture`
- `cargo test -p agent-drift-analyzer -- --nocapture`

### Checkpoint BC-D: Bounded Smoke Proves The Cutover

Verify:

- a bounded real-session compactor run emits the `v0.2` bundle
- analyzer consumption remains successful
- bundle size drops without row-count regressions

Evidence:

- the documented compactor/analyzer `cargo run` smoke commands from the spec
- recorded row-count and output-size comparison notes in repo docs

BC7 smoke note:

- `2026-05-31`: session `019e79dc-456c-7e92-bcbc-3b677d9e8b3f` produced a `v0.2` bundle with
  `252` archival rows, `197` compact rows, `26` dedupe groups, and a single manifest file-table
  entry. `agent-drift-analyzer` consumed the bundle successfully and emitted `17` checkpoints. The
  five compactor files totaled `1,294,344` bytes versus an estimated `1,356,876` bytes for the
  equivalent inline-path `v0.1` shape, a measured reduction of `62,532` bytes.

## Handoff To Later Slices

This plan is complete enough for later sentinel or analyzer follow-up work only when all of the
following are true:

- the compactor/analyzer seam is explicit and versioned at `v0.2`
- row and dedupe refs are compact on disk but still resolvable to stable operator-facing paths
- no mixed-contract ambiguity remains in current bounded fixtures or smoke guidance
- downstream slices can consume freshly regenerated analyzer artifacts without planning around the
  old path-heavy row export

## Exit Criteria

The plan is complete when:

1. `manifest.json` owns an explicit deterministic file registry for the `v0.2` bundle
2. exported row JSONL omits repeated `source_file` paths and uses `source_file_id`
3. dedupe audit refs are id-backed and still resolve to archival rows deterministically
4. analyzer input loads `v0.2` bundles without fallback guessing and preserves current semantic
   behavior
5. bounded smoke confirms analyzer compatibility and a measurable bundle-size reduction
