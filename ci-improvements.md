# CI Redundancy Reduction (Advisory) — Spec Draft

## Problem Statement
We need to reduce redundant cross-platform CI (Linux/macOS/Windows, sometimes WSL) **without** sacrificing safety. The failures we must prevent are “green on my platform, failing later on another platform”, but we also want to avoid spending 30–60+ minutes re-running the same multi-OS jobs when there were no meaningful code changes since the last green run.

This document proposes an **advisory** mechanism (recommendations only) that is **separate from**:
- triad task scripts (`make triad-task-start*`, `make triad-task-finish`, etc.)
- existing CI dispatch scripts (`scripts/ci/dispatch_ci_testing.sh`, `make ci-compile-parity`, `make feature-smoke`, etc.)

The mechanism answers:
1. What OSes does this step *need* to be green on?
2. What OSes were *actually* green in the last CI run(s)?
3. Has anything changed since then that would justify re-running CI?

Operator decision remains final: the tool recommends “skip/run”, and prints evidence and reasoning.

---

## Evidence: C1 Ran Too Much CI
In slice C1, we ran multiple “CI Testing” and “Feature Smoke (behavior)” workflows against the same orchestration branch head(s), even when subsequent changes were docs/planning-only.

We specifically want to prevent patterns like:
- Running “compile parity” (which is effectively “CI Testing”) and then running “CI Testing quick” and later “CI Testing full”, for the same or near-identical SHAs.
- Re-running “Feature Smoke (behavior)” even when the diff since the last successful smoke is docs-only.

### Concrete Observation (C1)
After a point, changes were docs/planning-only (e.g. `docs/**`), yet multi-OS CI was dispatched again. Under the desired policy:
- **Docs/planning-only changes can skip all CI** (including smoke + final CI gate).

---

## Goals / Non-Goals

### Goals
- Reduce redundant CI runs when there are no relevant changes since a recent green run that already covers the required platforms.
- Provide a crystal-clear “what changed / what was last green / what is required” report.
- Stay conservative by default: if unsure, recommend running CI.
- Be **advisory** first: recommendations only, no enforcement.

### Non-Goals (for the initial advisory version)
- Not rewriting task scripts or CI scripts.
- Not changing workflow behavior or matrix definitions.
- Not auto-reducing OS coverage based on heuristics (optional future).

---

## Proposal: `ci-audit` (Advisory CI Coverage + Change Impact Checker)

### High-Level Idea
`ci-audit` is a small CLI (or script) that:
1. Determines the **required OS coverage** for the current action:
   - CI Testing typically requires: `linux,macos,windows`
   - Feature Smoke (behavior) requires: from feature pack metadata (usually `tasks.json meta.behavior_platforms_required`), sometimes includes `wsl`
2. Finds the most recent **successful run evidence** that matches:
   - same feature orchestration branch (or explicit workflow ref)
   - the workflow kind being audited (CI Testing vs Feature Smoke)
3. Computes what OSes/jobs actually passed in that run.
4. Computes `git diff` between the last-green “tested SHA” and current `HEAD`, and classifies change impact:
   - `docs_only` => recommend skip for **all CI and smoke**
   - `code_affecting` => recommend run
   - `unknown` => recommend run

The output is a recommendation: **SKIP** or **RUN**, plus a reasoned breakdown.

### Current Status
- Implemented v1 script: `scripts/ci-audit/ci_audit.sh`
- Advisory only (prints recommendation; does not dispatch CI)

---

## Concrete Spec (v1: Minimal, Advisory)

### CLI Shape
Proposed command (example):
```bash
scripts/ci-audit/ci_audit.sh \
  --feature-dir docs/project_management/next/<feature> \
  --orch-branch feat/<feature> \
  --kind feature-smoke \
  --head-sha "$(git rev-parse HEAD)"
```

Notes:
- v1 can be a bash script for speed of adoption, or Rust for long-term reliability.
- This is **not** a dispatcher. It does not run CI. It prints a recommendation and exits 0.

### Required Inputs
- `--feature-dir <path>`
  - Used to read `tasks.json` for behavior platform requirements.
- `--orch-branch <ref>`
  - Used to query GitHub Actions runs on that branch.
- `--kind <ci-testing|feature-smoke>`
- `--required-platforms <csv>` (optional override; useful if `tasks.json` isn’t available)
- `--head-sha <sha>` (default `git rev-parse HEAD`)
- `--baseline-sha <sha>` (optional override; otherwise uses last-green head SHA or merge-base with `origin/testing`)

Optional:
- `--remote origin` (default `origin`)
- `--repo atomize-hq/substrate` (default inferred by `gh`)

### Output Contract (Machine + Human Readable)
Human-friendly summary plus a stable key/value section:
```
RECOMMEND=skip|run
REASON=<short reason>
REQUIRED_PLATFORMS=<csv>
LAST_GREEN_RUN_ID=<id|empty>
LAST_GREEN_RUN_URL=<url|empty>
LAST_GREEN_HEAD_SHA=<sha|empty>
LAST_PASSED_PLATFORMS=<csv|empty>
DIFF_CLASS=docs_only|code_affecting|unknown
DIFF_FILES_COUNT=<n>
DIFF_LOC=<insertions,deletions>
```

### Platform/OS Coverage Rules
**CI Testing**
- `REQUIRED_PLATFORMS` default: `linux,macos,windows`
- Consider “passed” if the run has successful jobs matching:
  - `Lint & Test (ubuntu-*)` => linux
  - `Lint & Test (macos-*)` => macos
  - `Lint & Test (windows-*)` => windows

**Feature Smoke (behavior)**
- `REQUIRED_PLATFORMS` derived from:
  - `jq -r '.meta.behavior_platforms_required // [] | join(",")' "$FEATURE_DIR/tasks.json"`
- Consider “passed” if successful jobs exist matching:
  - `linux_*` => linux
  - `macos_*` => macos
  - `windows_*` => windows
  - `wsl` => wsl

### Change-Impact Classification (v1)
We want an extremely conservative classifier.

**docs_only** (recommend SKIP for all CI and smoke)
- All changed paths are under:
  - `docs/**`
  - `docs/project_management/**`
- (Optionally: allow other non-product paths if we decide: e.g. `README.md`, but keep v1 small.)

**code_affecting** (recommend RUN)
- Any change touches:
  - `crates/**`, `src/**`
  - `Cargo.toml`, `Cargo.lock`
  - `.github/**`
  - `scripts/**`
  - `Makefile`
  - Anything else not in the docs-only allowlist

**unknown** (recommend RUN)
- No last-green run found, or cannot determine “tested SHA” reliably.

### “Do We Even Need to Run CI?” Decision Rule (v1)
Given:
- `REQUIRED_PLATFORMS`
- last successful run’s `LAST_PASSED_PLATFORMS`
- `DIFF_CLASS`

Recommend:
- If `DIFF_CLASS=docs_only` => `RECOMMEND=skip` (even if there is no last-green run)
- Else if `LAST_PASSED_PLATFORMS` covers all `REQUIRED_PLATFORMS` AND `git diff` is empty => `RECOMMEND=skip`
- Else => `RECOMMEND=run`

Rationale: your stated policy explicitly allows skipping CI entirely when docs-only. For code, we require either “no changes since last full coverage” or we run.

---

## Spec Extension (v2: Evidence Ledger for Stronger Guarantees)

### Why a Ledger?
When dispatch scripts create throwaway branches, GitHub run metadata like `headSha` can represent the orchestration branch SHA rather than the exact “checkout SHA” validated by CI.

To make skip decisions safe, we want an explicit record of:
- what SHA we intended to validate
- which platforms/jobs passed

### Ledger Format (JSONL)
Location suggestion (survives `cargo clean`, and aligned with feature logs):
- `$FEATURE_DIR/logs/<slice>/ci-audit/ledger.jsonl`

Entry example:
```json
{
  "timestamp": "2026-01-27T12:34:56Z",
  "orch_branch": "feat/<feature>",
  "kind": "ci-testing",
  "mode": "quick",
  "tested_sha": "<sha>",
  "required_platforms": ["linux","macos","windows"],
  "passed_platforms": ["linux","macos","windows"],
  "run_id": "2110....",
  "run_url": "https://github.com/.../runs/....",
  "conclusion": "success"
}
```

### Current Status
- Implemented ledger recording helper: `scripts/ci-audit/ci_audit_record.sh`
- Implemented ledger consumption in `ci-audit` via `--ledger-path`:
  - `scripts/ci-audit/ci_audit.sh --ledger-path <path> ...`

### Ledger Write Policy
Because we are “separate from task/dispatch scripts”, v2 can work in either of these ways:
1. Operator runs `ci-audit record --run-id <id> --tested-sha <sha> ...` after dispatch finishes.
2. Wrapper command for humans (not used by automation) that does:
   - dispatch CI via existing script
   - then records ledger evidence

Either way, `ci-audit` can prefer ledger evidence over heuristics from GH run metadata.

---

## Recommended Operator Workflow (Advisory)

Before dispatching:
1. Run `ci-audit` for the relevant gate (CI Testing vs Feature Smoke).
2. If it says `RECOMMEND=skip`, you can confidently skip (especially for docs-only).
3. If it says `RECOMMEND=run`, dispatch normally.
4. (Optional v2) Record ledger evidence after the run completes to strengthen future recommendations.

---

## Future (Optional) Improvements
These are not part of v1, but are natural extensions:

1. **Platform-scoped recommendations**:
   - If changes are strictly under `crates/world-mac-lima/**`, recommend macOS-only.
   - Default remains “all 3” for shared crates.

2. **Workflow stamping**:
   - Update CI workflows to print `TESTED_SHA=<sha>` in job summaries or artifacts.
   - Makes automated audit more reliable without a ledger.

3. **Escalation path**:
   - Keep “recommended” now; later add an enforce mode (opt-in) if desired.

---

## Acceptance Criteria (for implementing v1 later)
- Given a docs-only diff, `ci-audit` recommends skip for both:
  - CI Testing
  - Feature Smoke
- Given a code-affecting diff, `ci-audit` recommends run.
- Given no diff since a recent run that covers required platforms, `ci-audit` recommends skip.
- Output is clear and copy/pastable into session logs.
