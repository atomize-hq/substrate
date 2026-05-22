# WPEP0 — spec — trace correctness + joinability foundation

## Scope (explicit)
- Fix span parent linkage correctness for shim spans:
  - `command_complete.parent_span` MUST equal the parent captured at span start (no finish-time env re-read).
- Tighten completion ergonomics:
  - completion spans include `duration_ms` and `policy_decision` when known.
  - deny completion spans include `outcome: "denied"`.
- Improve cross-component joinability:
  - shell `command_start`/`command_complete` events MUST include `span_id` when a shim span exists.
  - replay `replay_strategy` events include `span_id` explicitly.
- Preexec privacy posture:
  - canonical trace `builtin_command` records omit command bodies and include `command_omitted: true`.
  - raw command bodies are permitted only in an opt-in debug-only file (`SUBSTRATE_PREEXEC_RAW_LOG`).

## Acceptance (explicit)
- Span correctness:
  - shim completion spans never self-parent (`parent_span != span_id` when parent_span is present).
- Joinability:
  - shell `command_complete` MUST include `span_id` for shimmed executions.
  - replay `replay_strategy` includes `span_id`.
- Deny clarity:
  - deny completion spans include `outcome: "denied"`.
- Preexec safety:
  - canonical trace includes `builtin_command` with `command_omitted: true`.
  - canonical trace contains no `builtin_command_raw` records.

## Out of scope (explicit)
- Any world-service API additions for `process_events`.
- Any Linux ptrace capture implementation.
