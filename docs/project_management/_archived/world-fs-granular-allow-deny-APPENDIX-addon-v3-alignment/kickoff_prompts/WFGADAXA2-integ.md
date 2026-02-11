# Kickoff: WFGADAXA2-integ (integration final)

## Scope
- Merge any platform fixes and complete the slice closeout gate.
- Spec: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-spec.md`
- Closeout: `docs/project_management/_archived/world-fs-granular-allow-deny-APPENDIX-addon-v3-alignment/WFGADAXA2-closeout_report.md`

## Requirements
- Do not edit planning docs inside the worktree.
- Ensure CP1 is complete and recorded in `session_log.md` before finalizing.
- Run:
  - `cargo fmt`
  - `cargo clippy --workspace --all-targets -- -D warnings`
  - relevant tests
  - `make integ-checks`
- Fill out `WFGADAXA2-closeout_report.md`.

