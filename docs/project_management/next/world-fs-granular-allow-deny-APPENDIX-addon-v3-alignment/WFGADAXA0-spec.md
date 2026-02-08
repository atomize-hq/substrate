# WFGADAXA0-spec — Effective policy display (Appendix A.6)

## Scope
- Make `substrate policy show` output compliant with Appendix A.6 and the Appendix “no backwards compatibility” posture.
- Primary mismatch to fix: effective policy serialization is still V2-shaped (legacy keys printed).

## Behavior
Authoritative requirements:
- Output contract: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` §1.3
- No backwards compatibility: `docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/contract.md` §5

Required behavior:
1. `substrate policy show` (YAML) and `substrate policy show --json` (JSON) output are V3-shaped:
   - V2 keys are not rendered as operator-facing output keys:
     - `world_fs.mode`, `world_fs.isolation`, `world_fs.require_world`, `world_fs.enforcement`
     - `world_fs.read_allowlist`, `world_fs.write_allowlist`
2. When the effective policy has `world_fs.host_visible=false`, output MUST include:
   - `world_fs.discover`, `world_fs.read`, and `world_fs.write`,
   - each with both `allow_list` and `deny_list`,
   - and empty deny lists MUST be rendered explicitly as `[]` (YAML + JSON).
3. When `discover` is defaulted from `read`, output still shows `discover` explicitly with its effective lists.

## Acceptance criteria
- Tests:
  - Add deterministic tests that fail if:
    - V2 keys appear in `substrate policy show` output, or
    - `deny_list` is omitted when empty in full isolation output.
- Commands (integration gate):
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - `cargo test -p substrate-shell --tests -- --nocapture`
  - `make integ-checks`

## Out of scope
- Changing the Appendix authoritative docs (`docs/project_management/next/world-fs-granular-allow-deny-APPENDIX/*`) unless a true ambiguity is discovered and resolved explicitly (decision register + doc update).
- Changing policy patch semantics (defaults/validation) beyond what is required to meet the output contract.
