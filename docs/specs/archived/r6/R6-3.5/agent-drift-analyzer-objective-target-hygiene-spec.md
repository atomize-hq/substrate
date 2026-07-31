# R6-3.5 SPEC — Objective Target Hygiene For Semantic Goal Drift

Status: OPEN (kickoff 2026-07-04). Packet-scoped to `crates/agent-drift-analyzer`.

Authority / cross-references:
- `docs/specs/r6/FINDINGS-r6-3-real-world-drift-validation.md` — the 110-session batch charter this
  packet executes (Step 1 "extraction hardening", Step 2 shadow-eval re-run). This SPEC is the
  concrete design for that charter's Step 1.
- `docs/specs/r6/MAP.md` — item 6 (R6-3) and the deferred `R6-3.X.2` (graduated distance) /
  `R6-3.X.3` (eligibility-bar loosening) debts, which stay deferred behind this packet.
- Live code: `crates/agent-drift-analyzer/src/context/objective.rs` (target extraction),
  `crates/agent-drift-analyzer/src/scoring/semantic_goal_drift.rs` (scorer + eligibility).

## Problem

On 110 real Codex sessions (882 checkpoints), `semantic_goal_drift` fired 6 times, all false
positives, all caused by **garbage target extraction**: a GraphQL server-startup log
(`0.0.0.0:4000`, `5s`, `/graphql`) parsed as file-path "targets", plus escaped-`\n` blobs, bare
model tokens (`GPT-5.4`), prose enumerations (`linux/mac/windows`, `closeout/review-ready`), and
truncated external paths (`/Users/.../Library/Application`). Observed live precision 0%.

Root cause: `context/objective.rs` admits non-target text as an `ObjectiveTarget` /
`comparison_key` term, and `scoring/semantic_goal_drift.rs` trusts those terms without a stability
guard, so a junk-only term set can be "eligible" and fire.

Locked order (do NOT reorder — see FINDINGS "Decision"): (1) extraction hardening FIRST **(this
packet)**, (2) graduated distance later (`R6-3.X.2`), (3) eligibility-bar loosening last
(`R6-3.X.3`). Phase is **deterministic** — no ML/model dependency.

## Design: two-stage typed target-anchor classifier

Introduce an internal, deterministic classifier shared by extraction and the scorer:

```
enum TargetAnchorQuality { Stable, Weak, Junk }
```

Classification is **grammar-first, then variable-mask** (a token that validates a typed grammar is
never re-masked as junk):

```
fn anchor_quality(raw_token) -> TargetAnchorQuality {
    if let Some(_kind) = stable_target_anchor_kind(raw_token) { Stable }
    else if is_variable_noise_token(raw_token)               { Junk }
    else                                                     { Weak }
}
```

### Stage B — typed-slot grammar validation (schema-guided DST pattern)

Design input (deterministic transfer only, no model): Schema-Guided Dialogue slot typing
([arXiv 1909.05855](https://arxiv.org/pdf/1909.05855)), FastSGT span validation
([arXiv 2008.12335](https://arxiv.org/pdf/2008.12335)); the R5 objective classifier taxonomy already
cites this family. Pattern transferred: a candidate is a target only if it validates a **typed
anchor grammar**; otherwise it is out-of-schema and does not become a scoring target.

`stable_target_anchor_kind(token)` returns `Some(kind)` iff the token validates one grammar:

- **RepoRelativePath** — leaf segment has a recognized extension (primary signal, root-agnostic):
  `.rs .md .toml .json .yaml .yml .py .ts .tsx .js .jsx .sh .lock .html .txt .cfg .rlib`; OR starts
  with `./` `../`; OR first segment is a conventional repo root
  (`crates src lib libs app apps pkg pkgs packages cmd internal docs doc spec specs test tests
  bench benches example examples fixtures proto schema scripts tools config .github .claude .codex`).
- **RecognizedExtensionFile** — a rootless filename ending in a recognized extension (`README.md`,
  `Cargo.lock`).
- **WellKnownRootlessFile** — extension-less but conventional: `README Makefile Dockerfile LICENSE
  CHANGELOG CONTRIBUTING SECURITY Cargo AGENTS CLAUDE`.
- **WindowsPath** — `C:\...` drive-absolute or backslash-separated multi-segment path.
- **MarkdownLinkTarget** — extracted from `[label](path)`; the path must itself validate a path
  grammar above.
- **RustSymbolRef** — contains `::` between identifier segments (`foo::bar`, `Type::method`).
- **CrateOrPackageRef** — cue-anchored ("crate X" / "package X") OR bare token matching workspace
  package shape (all-lowercase kebab, ≥2 `-`-joined alphabetic segments, no digits/dots/uppercase):
  admits `agent-drift-analyzer`, rejects `GPT-5.4`.
- **SpecOrDocName** — cue-anchored (`spec|design|doc|plan|tasks` + named artifact), existing grammar.
- **TestOrVerifierRef** — existing grammar.
- **WorkItemIdentifier** — existing `looks_like_work_item_identifier` grammar (`R6-3.5`, `SO-2.3B`).
- **WorkspaceRef** — leading `@` (`@workspace-ref`).
- **InstructionSurface** — existing `extract_named_artifacts` allowlist (`AGENTS.md`, `<skill>`, …).

Masks are **whole-value**, never "contains": `v2.3.1/notes.md`, `2024-report.md`, `0xdeadbeef.rs`,
and `docs/graphql/overview.md` all stay Stable because they validate a path grammar first.

### Stage A — variable/noise token masking (log-template pattern)

Design input: log-template parsing preprocessing masks — Drain/LogPai
([Preprocessing is All You Need, arXiv 2412.05254](https://arxiv.org/pdf/2412.05254);
[Drain3](https://github.com/logpai/Drain3)), LogPPT
([arXiv 2302.07435](https://arxiv.org/abs/2302.07435)). Pattern transferred: separate stable
template keywords from **dynamic parameters** by masking variable-value token classes. The 6 false
positives are a log line whose parameters were parsed as a template.

`is_variable_noise_token(token)` is true iff the **whole token** is one of:

- URL / API endpoint: `http(s)://…`, or a bare `/word` endpoint that is not a validated path
  (e.g. `/graphql`).
- IP or `host:port` / `ip:port`: `0.0.0.0:4000`, `localhost:4000`, `127.0.0.1:8080`.
- Pure numeric or coordinate run: all digits and separators (`.` `_` `-` `:` `x`), e.g.
  `0_0_0_0_4000`, `1920x1080`, `12.34.56`.
- Hex blob: `0x…`-free long hex run with no recognized extension.
- Timestamp / duration: `5s`, `120ms`, `2025-10-03T11:58:13`, bare `HH:MM:SS`.
- Escaped-control residue: literal `\n` / `\t` sequences, or a token that is only a stray control
  letter (`n`) left by escaped-newline splitting.
- Degenerate: empty, single-char, or all-separator.

### Weak routing

`Weak` tokens are **display/debug-only**: they may inform the human-readable `target.display` but
are **never** materialized into `comparison_key`, scorer term sets, or eligibility. Only `Stable`
tokens seed target `paths`/`symbols`/`named_artifacts`/`workspace_refs` specifics and the
`comparison_key` target segments. A goal whose only anchors are `Weak`/`Junk` yields
`target = None` for scoring purposes (`unknown_target`).

## Scorer backstop (defense-in-depth, single shared taxonomy)

`scoring/semantic_goal_drift.rs` gets a stable-term guard built from the **same** taxonomy (shared
predicates, not a second classifier — the review's key hazard):

- `push_goal_term` rejects a normalized term that is not a stable goal term (numeric/coordinate run,
  version token, hex, control residue, single char) via a shared `is_stable_goal_term`.
- `eligible_current_goal` requires **≥ 1 stable target-derived term** after filtering. A junk-only
  target term set → not eligible → no fire, preserving the strict posture but making it semantically
  meaningful. Constraint-derived terms (platform/scope boundaries) do **not** count toward this
  target-stability requirement and are **not** junk-filtered (constraint separation).
- Applied **symmetrically** to current, previous, and anchor sides.

## Delegation guardrail (bounded; not full R7 subagent support)

Uses the already-computed `CheckpointAnalysis.delegation`. Scope is limited to preventing
over-claim on parent-only traces; full parent/child semantics stay in `R7`.

- `DelegatingParent` + `Opaque` child visibility → hard limited-evidence `no_claim` unless **both**
  compared goals carry a stable target anchor.
- `Partial` / `MixedOrAmbiguous` → rely on the stable-term backstop (already prevents junk fires);
  no hard suppression (avoids killing genuine parent-side drift — review §5).

## Normalization

One canonical form for anchors before comparison: trim wrapping punctuation/quotes/backticks; strip
a single trailing sentence period; collapse `./x`→`x`; preserve internal slashes; case-preserving
for display, lowercased for term comparison (matches existing `normalize_goal_term`).

## Acceptance

- The 6 witnessed junk classes yield no stable target and cannot make a checkpoint eligible.
- False-negative guard (review §1/§4): `Cargo.lock`, `.github/workflows/ci.yml`, `README`,
  `agent-drift-analyzer` (bare crate), `foo::bar`, `C:\repo\x.rs`, `v2.3.1/notes.md`,
  `2024-report.md`, `docs/graphql/overview.md`, `[overview](docs/architecture_overview.md)` all
  still produce a stable target.
- R6-2 / R6-3 acceptance fixtures stay green; a valid target change still fires; a junk-only current
  goal does not.
- Batch re-run (Step 2): 6 → 0 fires; 100%-junk current-bar checkpoints 8 → 0; metrics reported
  stratified by delegation category.

## Non-Goals

- Graduated/weighted distance (`R6-3.X.2`); eligibility-bar loosening (`R6-3.X.3`).
- Full parent/child delegated-session semantics (`R7`).
- `TaskFrame` / `working_set` / `progress` migration; compactor normalization; sentinel behavior.
- Any ML/model dependency.
