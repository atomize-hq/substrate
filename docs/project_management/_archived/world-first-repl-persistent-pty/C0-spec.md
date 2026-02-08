# C0-spec — World-agent persistent session bootstrap + fail-closed preflight (server-side)

This slice introduces persistent REPL sessions on `/v1/stream` up to (and including) `start_session → ready`, and enforces fail-closed preconditions required by DR-22 and DR-23 before emitting `ready`.

Authoritative specs (do not diverge):
- `docs/project_management/_archived/world-first-repl-persistent-pty/PROTOCOL.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/decision_register.md` (DR-21, DR-22, DR-23)
- `docs/project_management/_archived/world-first-repl-persistent-pty/driver_loop_design.md`
- `docs/project_management/_archived/world-first-repl-persistent-pty/drain_design.md`

Out of scope for C0:
- Per-submission execution (`exec` → `command_complete`) — C1.
- Host-side persistent session client and REPL behavior — C2/C3/C4.
- Non-interactive `-c/--command` behavior — C5.

## Contract (C0 deliverable)

World-agent `/v1/stream` MUST support persistent sessions for the interactive REPL:
- First client frame MUST be `start_session` (or legacy `start` for one-shot mode); any other first frame MUST fail closed with `error.code=bad_request` (fatal).
- For persistent-session mode, world-agent MUST implement and validate:
  - `start_session` (client → agent)
  - `ready` (agent → client)
  - `error` (agent → client; fatal)
  - `close` (client → agent)
  - `exit` (agent → client)
- Version negotiation MUST be fail-closed:
  - `ready.protocol_version` MUST be `1`.
  - If the client requests an unsupported protocol, world-agent MUST fail closed (fatal `error`), and MUST NOT emit `ready`.

Legacy compatibility:
- Existing one-shot `/v1/stream` behavior (`start` → PTY bytes → `exit`) MUST remain supported and unchanged.

Fail-closed preflight (locked):
- DR-23 ordering barrier precondition:
  - Watermark-query support for the Session PTY MUST be validated during `start_session` before emitting `ready`.
  - v1 requires Linux `ioctl(FIONREAD)`; if unsupported, world-agent MUST fail closed and MUST NOT emit `ready`.
- DR-22 control-plane handle privacy precondition:
  - World-agent MUST fail closed during `start_session` if it cannot guarantee that evaluator processes will not be able to access session infrastructure/control-plane handles/endpoints.
  - This guarantee is a start-session precondition. It MUST be established before `ready`, not discovered after an `exec`.

Session metadata:
- World-agent MUST return `ready.session_nonce` as `hex32` and MUST freshly generate it per session.
- `ready.session_nonce` is observability-only and MUST NOT be treated as a capability/credential (see `PROTOCOL.md`).

## Acceptance criteria
- Persistent-session handshake and fail-closed behavior match `PROTOCOL.md` for:
  - first-frame enforcement,
  - protocol version validation,
  - `ready` schema,
  - and fatal `error` framing.
- DR-22 and DR-23 preconditions are validated before `ready`, with fail-closed errors when not satisfiable.
- Legacy one-shot `start` behavior remains supported.

## Validation (C0-test scope)
Add world-agent tests that cover, at minimum:
- Protocol framing:
  - first frame enforcement for persistent session (`start_session` only),
  - legacy one-shot `start` remains accepted.
- Version negotiation:
  - unsupported version is fatal (no `ready`).
- Preflight fail-closed:
  - DR-23 watermark capability is validated before `ready`.
  - DR-22 privacy guarantee precondition is validated before `ready`.
- `ready.session_nonce` format and per-session uniqueness.

## Cross-platform notes
- Persistent-session behavior is implemented by world-agent (Linux). macOS hosts run through Lima (Linux guest); the same world-agent behavior is required.
