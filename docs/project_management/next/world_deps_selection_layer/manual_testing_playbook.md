# World Deps Selection Layer — Manual Validation Playbook

This document is the human-run validation checklist for the “world-deps selection layer” and install-class routing described in ADR-0002.

This playbook is intended to be:
- end-to-end (install → configure selection → status → sync),
- cross-platform (Linux/macOS Lima/Windows WSL where applicable),
- explicit about expected behavior (including failures),
- and easy to copy into PR descriptions as testing evidence.

## 0) Preconditions (all platforms)

1) Install Substrate and ensure `substrate` is on PATH:
   ```bash
   substrate --version
   which substrate
   ```
2) Ensure the world backend is healthy:
   ```bash
   substrate world doctor --json | jq .
   ```
3) Capture baseline health:
   ```bash
   substrate health --json | jq .
   ```

## 1) Selection gating (required behavior)

### 1.1 Unconfigured selection is a no-op with actionable guidance

1) Confirm selection is absent (workspace + global as specified by the spec):
   - Remove/rename the selection file(s) for the test workspace.
2) Run:
   ```bash
   substrate world deps status
   substrate world deps sync
   ```
3) Expectation:
   - No installs performed.
   - Clear guidance on how to configure selection.
   - Exit codes match the spec (warn vs fail-closed semantics).

### 1.2 Configured selection drives the tool set

1) Configure a minimal selection (example path per spec).
2) Run:
   ```bash
   substrate world deps status --json | jq .
   ```
3) Expectation:
   - Only selected tools are considered “in scope” for status/sync.
   - Output explicitly lists unselected-but-detected tools (if that’s part of the UX spec) as “not selected”.

## 2) Install class enforcement (required behavior)

### 2.1 `user_space` installs succeed (prefix writable)

1) Choose a tool declared as `user_space` in the inventory (e.g., a curl/tarball installer into `SUBSTRATE_WORLD_DEPS_GUEST_BIN_DIR`).
2) Run:
   ```bash
   substrate world deps sync
   substrate world deps status --json | jq .
   ```
3) Expectation:
   - Tool transitions to “guest=present” (or equivalent status).
   - No writes to OS package databases.

### 2.2 `system_packages` never installs at runtime via world-deps

1) Choose a tool declared as `system_packages`.
2) Run:
   ```bash
   substrate world deps sync
   ```
3) Expectation:
   - No attempt to call `apt`, `dnf`, `brew`, etc. as part of world-deps sync.
   - The CLI either:
     - routes you to the explicit provisioning-time command, or
     - fails with an explicit, actionable error if provisioning is unsupported on the platform.

### 2.3 `manual` installs never run

1) Choose a tool declared as `manual`.
2) Run:
   ```bash
   substrate world deps sync
   ```
3) Expectation:
   - No install attempt.
   - Output contains explicit manual steps and where to put the resulting binaries so Substrate detects them.

## 3) Platform-specific parity checks

### 3.1 macOS (Lima guest)

1) Verify Lima provisioning state:
   ```bash
   substrate world doctor --json | jq '.lima'
   ```
2) Run the selection + sync flows from sections 1–2.
3) Expectation:
   - `system_packages` behavior is handled via the provisioning-time route; no runtime apt/dpkg mutation from world-deps.
   - Status output is consistent with Linux/WSL for the same selection.

### 3.2 Linux (host world-agent)

1) Run the selection + sync flows from sections 1–2.
2) Expectation:
   - Host OS package mutation is not performed by default.
   - Any opt-in flow is explicit and aligned with policy/approval requirements.

### 3.3 Windows (WSL)

1) Warm/verify WSL backend per the WSL docs/scripts.
2) Run the selection + sync flows from sections 1–2.
3) Expectation:
   - Same UX semantics as macOS/Linux for selection gating and install classes.

## 4) Trace / logging expectations (spot checks)

1) Run one successful `world deps sync` and one `system_packages`-blocked attempt.
2) Inspect trace for clear, non-sensitive status fields:
   ```bash
   tail -n 50 ~/.substrate/trace.jsonl | jq .
   ```
3) Expectation:
   - Install-class decision and routing are observable (at least in CLI output; trace fields per spec).
   - No secrets recorded.

## 5) Evidence to capture in PRs (copy/paste)

- `substrate world doctor --json` (platform-specific section)
- `substrate world deps status --json`
- The outputs from:
  - unconfigured selection no-op
  - successful `user_space` install
  - `system_packages` runtime block + explicit routing guidance

