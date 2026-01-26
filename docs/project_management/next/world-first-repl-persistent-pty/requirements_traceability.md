# Requirements Traceability — World-First REPL With Persistent World PTY (v1)

This document maps every MUST/SHOULD in the authoritative spec pack to:
- an explicit implementation task (from `docs/project_management/next/world-first-repl-persistent-pty/tasks.json`), and
- an explicit validation step (test or manual verification).

Authoritative spec pack (no drift allowed):
- `docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md`
- `docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md`
- `docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md`
- `docs/project_management/next/world-first-repl-persistent-pty/decision_register.md`
- `docs/project_management/next/world-first-repl-persistent-pty/plan.md`
- `docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md`
- `docs/project_management/next/world-first-repl-persistent-pty/drain_design.md`
- `docs/project_management/next/world-first-repl-persistent-pty/RESEARCH.md` (historical context only; decisions already locked)

## Task IDs (reference)
- C0: world-agent persistent session bootstrap (`C0-code`, `C0-test`, `C0-integ-*`)
- C1: world-agent per-submission exec (`C1-code`, `C1-test`, `C1-integ-*`)
- C2: shell persistent session client core (`C2-code`, `C2-test`, `C2-integ-*`)
- C3: interactive REPL routing + lifecycle (`C3-code`, `C3-test`, `C3-integ-*`)
- C4: interactive REPL byte-safe rendering + buffering (`C4-code`, `C4-test`, `C4-integ-*`)
- C5: non-interactive `-c` + stdin pipe mode routing (`C5-code`, `C5-test`, `C5-integ-*`)

Validation artifacts:
- Tests (per slice): owned by `C*-test` and asserted in `C*-integ-core` end checklists.
- Manual playbook: `docs/project_management/next/world-first-repl-persistent-pty/manual_testing_playbook.md`
- Smoke scripts: `docs/project_management/next/world-first-repl-persistent-pty/smoke/*`

## Canonical requirements (stable IDs)

The table below is canonical. The subsequent “Occurrence index” section lists every MUST/SHOULD line in the spec pack and maps it to one row here.

| Req ID | Level | Canonical requirement (summary) | Implemented by | Validated by |
|---|---|---|---|---|
| R-001 | MUST | Persistent REPL completion is explicit (`start_session → ready → exec → command_complete`); no stdout marker parsing; fail closed on protocol violations | `C0-code`, `C1-code`, `C2-code`, `C3-code` | `C0-test`, `C1-test`, `C2-test`, `C3-test` |
| R-002 | MUST/SHOULD | `ready.session_nonce` is observability-only (not a credential); MUST be freshly generated per session; SHOULD be recorded in trace metadata | `C0-code`, `C3-code` | `C0-test`, `C3-test` |
| R-003 | MUST | v1 persistence scope: MUST persist physical cwd + exported env mutations across submissions; other shell-local state is not guaranteed | `C1-code`, `C3-code` | `C1-test`; manual playbook “Interactive REPL world-first semantics” |
| R-004 | MUST | Out-of-band Session PTY `stdout` bytes MAY occur while idle/after completion; host MUST forward/render; v1 MUST NOT guess attribution | `C1-code`, `C4-code` | `C4-test` |
| R-005 | MUST/SHOULD | PTY bytes vs host structured output separation: structured host output MUST NOT be injected into PTY bytes; during passthrough it SHOULD be buffered and flushed after completion | `C4-code` | `C4-test` |
| R-006 | MUST/SHOULD | Evaluator shell: MUST use `/bin/bash --noprofile --norc`; world-agent SHOULD suppress in-world prompts (`PS1/PS2/PROMPT_COMMAND`) to avoid polluting the Session PTY stream | `C1-code` | `C1-test` |
| R-007 | MUST | No PS2 continuation: submissions are bounded program strings; incomplete constructs must error and return to idle; host MUST NOT rely on interactive continuation prompts | `C1-code`, `C3-code` | `C1-test`, `C3-test` |
| R-008 | MUST | Command/control separation: program text MUST NOT go over PTY stdin; PTY stdin is reserved for user keystrokes in passthrough mode | `C1-code` | `C1-test` |
| R-009 | MUST | v1 driver loop is world-agent owned and in-process; persistent session uses a trusted driver component with a private control plane | `C0-code`, `C1-code` | `C0-test`, `C1-test` |
| R-010 | MUST | DR-22 control-plane handle privacy: evaluator MUST NOT access session control-plane handles/endpoints; fail closed during `start_session` if this cannot be guaranteed; attempts to access non-stdio fds must not desync/spoof completion | `C0-code`, `C1-code` | `C0-test`, `C1-test` |
| R-011 | MUST | `SHIM_PARENT_CMD_ID` scoping: set per exec; MUST NOT persist as exported env across submissions | `C1-code`, `C3-code` | `C1-test`, `C3-test` |
| R-012 | MUST | `start_session.policy_snapshot` schema validation is fail-closed (`deny_unknown_fields`); invalid snapshot is `error.code=bad_request` (fatal) | `C0-code` | `C0-test` |
| R-013 | MUST | `start_session.env` is authoritative: evaluator starts from cleared env, MUST NOT inherit world-agent process env; MUST strip shim/runtime control vars from persisted session env | `C0-code`, `C1-code` | `C1-test` |
| R-014 | MUST | Startup cwd resolution: world-agent honors `start_session.cwd` only if valid under resolved session root; else starts at resolved root and reports `ready.cwd`; host treats `ready.cwd` as authoritative and reports cwd change | `C0-code`, `C3-code` | `C0-test`, `C3-test` |
| R-015 | MUST | Raw PTY output handling: `stdout` is bytes (may be non-UTF8); host MUST forward bytes unchanged and MUST render while Reedline is active via a byte-capable output path (not string-only printers) | `C2-code`, `C4-code` | `C4-test` |
| R-016 | MUST | Version negotiation + framing: `ready.protocol_version==1` only; unknown server frame types are fatal; host treats malformed frames as protocol errors and fails closed | `C0-code`, `C2-code`, `C3-code` | `C0-test`, `C2-test` |
| R-017 | MUST | Per-command correlation: host validates awaited `(seq, token_hex)`; mismatch is fatal protocol error | `C1-code`, `C2-code` | `C1-test`, `C2-test` |
| R-018 | MUST | No pipelining: only one `exec` in flight; concurrent `exec` is fatal protocol error | `C1-code`, `C2-code` | `C1-test`, `C2-test` |
| R-019 | MUST/SHOULD | Token redaction: host MUST NOT print full `token_hex` to operator terminal; SHOULD redact/hash in logs/traces | `C2-code`, `C3-code` | `C2-test` |
| R-020 | MUST/SHOULD | `command_complete`: exit code semantics reflect evaluator `$?` (signal exits SHOULD follow bash conventions); `cwd` is physical (`pwd -P`/`getcwd()`), absolute | `C1-code` | `C1-test` |
| R-021 | MUST | Path namespace: `ready.cwd`/`command_complete.cwd` are world-absolute; host MUST NOT require host-side existence (no `fs::canonicalize()` for snapshot/workspace resolution) | `C3-code`, `C5-code` | `C3-test`, `C5-test`; smoke `linux-smoke.sh` (C5 regression) |
| R-022 | MUST | Output ordering (DR-23): `command_complete` MUST NOT be emitted until all foreground PTY bytes are forwarded; v1 uses a watermark barrier (`ioctl(FIONREAD)`) and MUST fail closed if unsupported (validated during `start_session`) | `C0-code`, `C1-code` | `C0-test`, `C1-test` ordering tests |
| R-023 | MUST | Host forwarding: during passthrough, host forwards `stdin` bytes; host forwards terminal `resize`; host MAY forward host-originated signals | `C3-code`, `C4-code` | `C4-test` |
| R-024 | MUST | `stdin` acceptance rules: world-agent drops stdin unless a command is running with `stdin_mode=passthrough`; after `command_complete`, drops until next passthrough exec | `C1-code` | `C1-test` |
| R-025 | MUST | `signal` targeting: drop signals outside Running; deliver to Session PTY foreground process group; MUST NOT target session infrastructure; typed `Ctrl+C` is transported as `stdin` byte `0x03` (not a `signal` message) | `C1-code`, `C3-code` | `C1-test`, `C3-test` |
| R-026 | SHOULD/MUST | Shutdown semantics: host SHOULD send `close`; `exit` is graceful only when host is shutting down; unexpected `exit` is fatal (fail closed) | `C0-code`, `C2-code`, `C3-code` | `C0-test`, `C2-test`, `C3-test` |
| R-027 | MUST | `error` frames: schema is stable; `fatal` MUST be `true` for v1; host treats any error as fatal; malformed error is protocol error (fail closed) | `C0-code`, `C1-code`, `C2-code` | `C0-test`, `C1-test`, `C2-test` |
| R-028 | MUST | Fail-closed posture (no fallbacks): unexpected WS close, `error`, unexpected `exit`, or protocol errors terminate the REPL session (no host fallback) | `C2-code`, `C3-code` | `C3-test`; manual playbook “Protocol failure mode is fail-closed” |
| R-029 | MUST/SHOULD | Policy snapshot drift restart: before each command, host MUST compute effective snapshot hash + workspace root; on drift, MUST restart session; MUST emit an operator-visible restart message even if cwd continuity is preserved; SHOULD request cwd continuity; MUST report cwd change if continuity fails | `C3-code` | `C3-test` |
| R-030 | MUST/SHOULD | REPL UX model: prompt derives from `world_cwd` or `host_cwd`; prompt SHOULD distinguish contexts; multiline submissions are program text (not directives); directive tokenization requires `:host␠` / `:pty␠`; bare directive is an error | `C3-code` | `C3-test`; manual playbook “Interactive REPL world-first semantics” |
| R-032 | MUST | Startup invariants: host sends initial terminal size in `start_session` and keeps it updated; after `ready`, recomputes snapshot/workspace root and restarts immediately if inconsistent | `C3-code` | `C3-test` |
| R-037 | MUST | Host execution invariants: `:host` uses `host_cwd` and `host_env`; `:host cd` mutates `host_cwd`; `:host export/unset` mutates `host_env`; MUST NOT affect world session persistence | `C3-code` | `C3-test`; manual playbook “`:host` gating (disabled by default)” |
| R-038 | MUST | Fatal state handling: any entry into `Error` MUST be followed by shutdown; no degraded continuation | `C3-code` | `C3-test` |
| R-039 | MUST/SHOULD | Signal handling: host-originated `SIGINT` MUST be forwarded to world session while executing world commands; for host execution it SHOULD target the host child process | `C3-code` | `C3-test` |
| R-041 | MUST | Terminal resize: host MUST forward resize events to world-agent when a world session exists | `C3-code` | `C3-test` |
| R-042 | MUST/SHOULD | Observability: each submission MUST produce a trace command span with execution origin, exit code, cwd, policy snapshot hash/world id; cmd_id MUST be propagated as `SHIM_PARENT_CMD_ID`; bootstrap commands MUST NOT be recorded; out-of-band output SHOULD emit a trace event | `C3-code`, `C4-code` | `C3-test`, `C4-test` |
| R-043 | MUST | Non-interactive routing (ADR-0016): when world enabled, `-c/--command` MUST run in world; `cd/pwd/export/unset` MUST NOT be host-only builtins; `:host` MUST NOT be recognized in `-c` | `C5-code` | `C5-test`; smoke `linux-smoke.sh` (C5) |
| R-044 | MUST | `:host` gating (ADR-0016 + DR-10): REPL-only; disabled by default; requires explicit startup opt-in; if disabled, reject and do not execute on host or world; REPL-only setting MUST NOT be honored in non-interactive flows | `C3-code`, `C5-code` | `C3-test`, `C5-test` |
| R-045 | MUST | `:pty` semantics (DR-12/DR-18): when world enabled, runs inside the persistent session; when `--no-world`, runs on host PTY; when world enabled but unavailable, fail closed | `C3-code` | `C3-test`; manual playbook “`:pty` shares persistent session state (world enabled)” |
| R-050 | MUST | No fallbacks / no legacy switch (DR-06): no hidden switches or compat mode restoring legacy REPL routing | `C3-code`, `C5-code` | `C3-test`, `C5-test` |

## Occurrence index (exhaustive)

Every MUST/SHOULD occurrence in the spec pack is indexed below and mapped to a canonical requirement row above.

Format:
- `<path>:<line>` → `<Req ID>`

Notes:
- The same requirement may appear in multiple documents; each occurrence still appears here.
- `decision_register.md` includes non-selected options for historical context; only “Selected” options are implemented in v1.

- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:53  →  R-043
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:74  →  R-043
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:101  →  R-022
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:102  →  R-005
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:103  →  R-015
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:105  →  R-003
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:108  →  R-029
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:109  →  R-010
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:117  →  R-043
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:118  →  R-043
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:120  →  R-043
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:123  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:125  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:126  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:149  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:150  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:196  →  R-044
- docs/project_management/next/ADR-0016-world-first-repl-persistent-pty.md:197  →  R-044
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:32  →  R-002
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:55  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:57  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:58  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:62  →  R-004
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:63  →  R-004
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:67  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:71  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:72  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:77  →  R-028
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:81  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:84  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:87  →  R-006
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:90  →  R-006
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:95  →  R-006
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:96  →  R-006
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:97  →  R-003
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:104  →  R-008
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:105  →  R-008
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:113  →  R-009
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:117  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:125  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:129  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:133  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:134  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:135  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:138  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:141  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:145  →  R-011
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:146  →  R-011
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:147  →  R-003
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:151  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:152  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:153  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:155  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:160  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:163  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:166  →  R-012
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:174  →  R-012
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:175  →  R-012
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:181  →  R-013
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:182  →  R-013
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:184  →  R-013
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:193  →  R-014
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:198  →  R-009
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:201  →  R-014
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:203  →  R-014
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:204  →  R-014
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:207  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:210  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:213  →  R-002
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:214  →  R-002
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:220  →  R-016
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:221  →  R-016
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:224  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:235  →  R-019
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:239  →  R-018
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:240  →  R-018
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:242  →  R-009
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:250  →  R-007
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:253  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:257  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:258  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:261  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:262  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:266  →  R-014
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:267  →  R-021
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:271  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:272  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:277  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:279  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:280  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:281  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:286  →  R-017
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:290  →  R-017
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:293  →  R-023
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:295  →  R-023
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:304  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:305  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:306  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:307  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:308  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:316  →  R-024
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:319  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:320  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:321  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:322  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:323  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:330  →  R-026
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:334  →  R-026
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:335  →  R-026
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:350  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:353  →  R-027
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:354  →  R-027
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:371  →  R-028
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:377  →  R-028
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:386  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:389  →  R-029
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:392  →  R-029
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:393  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:399  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:404  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/PROTOCOL.md:436  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:29  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:41  →  R-032
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:43  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:46  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:47  →  R-032
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:51  →  R-028
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:54  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:56  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:67  →  R-004
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:68  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:69  →  R-015
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:74  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:81  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:85  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:100  →  R-028
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:110  →  R-029
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:111  →  R-029
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:113  →  R-030
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:114  →  R-029
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:121  →  R-018
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:135  →  R-021
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:157  →  R-018
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:176  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:183  →  R-037
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:184  →  R-037
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:187  →  R-037
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:188  →  R-037
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:210  →  R-038
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:216  →  R-025
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:223  →  R-039
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:225  →  R-039
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:226  →  R-039
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:235  →  R-041
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:238  →  R-042
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:243  →  R-042
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:245  →  R-042
- docs/project_management/next/world-first-repl-persistent-pty/STATE_MACHINE.md:248  →  R-042
- docs/project_management/next/world-first-repl-persistent-pty/decision_register.md:91  →  R-050
- docs/project_management/next/world-first-repl-persistent-pty/decision_register.md:196  →  R-043
- docs/project_management/next/world-first-repl-persistent-pty/decision_register.md:452  →  R-050
- docs/project_management/next/world-first-repl-persistent-pty/decision_register.md:487  →  R-050
- docs/project_management/next/world-first-repl-persistent-pty/decision_register.md:489  →  R-050
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:12  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:14  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:15  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:92  →  R-042
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:93  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:97  →  R-039
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:121  →  R-020
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:123  →  R-019
- docs/project_management/next/world-first-repl-persistent-pty/plan.md:124  →  R-011
- docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md:33  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md:69  →  R-010
- docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md:305  →  R-001
- docs/project_management/next/world-first-repl-persistent-pty/driver_loop_design.md:366  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:9  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:15  →  R-005
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:17  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:120  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:147  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:155  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:157  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:250  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:252  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:267  →  R-022
- docs/project_management/next/world-first-repl-persistent-pty/drain_design.md:328  →  R-022
