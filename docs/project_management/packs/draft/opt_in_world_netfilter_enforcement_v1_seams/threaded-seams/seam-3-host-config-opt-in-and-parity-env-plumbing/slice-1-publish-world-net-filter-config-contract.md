---
slice_id: S1
seam_id: SEAM-3
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to config schema merge/patch behavior"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-03
contracts_produced:
  - C-04
contracts_consumed: []
open_remediations:
  - REM-004
candidate_subslices: []
---
### S1 - Publish `world.net.filter` as the authoritative host config contract

- **User/system value**: the host has one explicit, back-compat-safe answer for whether it may request netfilter enforcement at all.
- **Scope (in/out)**:
  - In:
    - Add `world.net.filter: bool` under `WorldConfig` with default `false`.
    - Add `config set` / `config reset` support for the key.
    - Include the key in `config current show --explain`.
  - Out:
    - No-workspace override env (`S2`)
    - export parity env (`S2`)
    - operator docs/examples (`S3`)
- **Acceptance criteria**:
  - `world.net.filter` exists in the config schema and follows the existing replace semantics for world booleans.
  - `substrate config set world.net.filter=true|false` and `substrate config reset world.net.filter` mutate the workspace/global patch correctly.
  - `substrate config current show --explain` reports the correct source layer for `world.net.filter`.
  - The default effective value is `false`.
- **Dependencies**:
  - `../../threading.md` (`C-04`, `THR-03`)
- **Verification**:
  - Add tests in `crates/shell/tests/config_set.rs` for set/reset behavior.
  - Add tests in `crates/shell/tests/config_show.rs` for effective precedence and explain provenance.
- **Review surface refs**:
  - `review.md` (precedence drift hotspot)

Checklist:
- Implement: config schema + patch/update/reset support
- Test: set/reset + current-show explain coverage
- Validate: default remains `false`
- Cleanup: avoid introducing one-off precedence rules for this key
