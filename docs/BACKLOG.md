# Substrate Backlog

Status: living document capturing near-term and upcoming work.
Keep concise, actionable, and security-focused.

## Next

- **P0 – In-world process execution tracing parity (match host-level visibility)**
  - Problem: host execution is richly observable via the shim (per-process exec logging), but world execution is primarily observable at the “one command per world execute” level. This creates blind spots for world-deps installs and wrapper-based tools (e.g. `nvm` wrappers invoking `bash -lc ...`), where internal subprocesses are not recorded as structured events.
  - Goal: in-world activity is traceable at the same granularity as the host: every spawned process (argv/env redaction/exit code/timing) is captured and attached to spans, so debugging and policy auditing do not depend on stdout/stderr inference.
  - Work:
    - Define an in-world execution event schema (align with `crates/common/src/lib.rs` `log_schema`) for per-process spawn/exit telemetry emitted by world-service.
    - Extend `world-service` execution paths (`/v1/execute`, `/v1/stream`) to capture process tree events for the executed command (including wrapper-invoked shells like `bash -lc`) and return them to the host.
    - Follow-on optimization: stream per-process events incrementally over `/v1/stream` (v1 batches on Exit).
    - Plumb the returned process events into `crates/trace` spans so they are persisted alongside existing world fs diffs and policy decisions.
    - Ensure redaction rules apply consistently (no secrets in argv/env); reuse shared redaction helpers.
    - Add integration tests that demonstrate parity:
      - a command that spawns children in-world records child execs,
      - wrapper-generated entrypoints (e.g. `nvm`) record the inner `bash` invocation and downstream child processes.
  - Acceptance: `trace.jsonl` contains structured per-process world execution events comparable to shim host events; debug/audit can attribute failures to specific in-world subprocesses without relying on stdout/stderr; wrappers remain deterministic and trace-friendly.

- **P0 – Rename `world_fs.require_world` / `SUBSTRATE_WORLD_REQUIRE_WORLD` to semantic “fail closed” naming (no backwards compatibility)**
  - Problem: the current name reads like “world must be enabled”, but the actual behavior is “allow host fallback when world routing fails vs fail closed”. This causes configuration mistakes and confusion during debugging.
  - Work:
    - Rename the policy/config knob from `world_fs.require_world` to a semantic fail-closed naming shape (e.g. `world_fs.fail_closed.world`) and rename the exported state env var from `SUBSTRATE_WORLD_REQUIRE_WORLD` to match (e.g. `SUBSTRATE_WORLD_FAIL_CLOSED`).
    - Add a policy-level `caged_required: true|false` that overrides workspace config `world.caged` (enforce `SUBSTRATE_CAGED=1` when required; fail closed if it cannot be enforced on the selected backend/protocol).
    - Make REPL exit behavior explicit and configurable:
      - Print a note on `exit`/`quit` (and `Ctrl+D` if treated as exit) when `world_cwd != entered_cwd`, e.g. `substrate: note: returning to host cwd: <path>`.
      - Add a config knob to control where the host returns on REPL exit:
        - `repl.exit_cwd: entered|last_world`
    - Desired policy shape (example):
      - `world_fs:`
      - `  mode: writable`
      - `  isolation: workspace`
      - `  fail_closed:`
      - `    world: true`
      - `caged_required: true`
    - **No backwards compatibility:** do not accept the old config/env names; delete/rename the fields and update schema/validation/tests/docs in lockstep.
    - Fix documentation to explicitly describe behavior:
      - what happens when the world backend is unavailable,
      - what happens when the world is disabled (`--no-world` / `SUBSTRATE_WORLD=disabled`),
      - which layer owns the knobs (policy-driven “fallback vs fail-closed” + “caged required”, not effective-config override inputs).
    - Update references across docs and standards (`docs/WORLD.md`, `docs/CONFIGURATION.md`, `docs/reference/env/contract.md`, `docs/internals/env/inventory.md`, planning pack templates/smoke scripts) and ensure error messages/warnings use the new name.
  - Acceptance: operators can understand intent from the name alone; docs explain fallback vs fail-closed semantics unambiguously; caged-required policy override is explicit; CI/tests are updated; old names are rejected (hard error) with no aliasing.

- **P1 – Make `SUBSTRATE_OVERRIDE_*` override workspace config (behavior change)**
  - Problem: operators expect `SUBSTRATE_OVERRIDE_*` to support one-off runs that override workspace config; today, when a workspace is enabled, `SUBSTRATE_OVERRIDE_*` is ignored for effective config resolution.
  - Desired contract: when a workspace is enabled, `SUBSTRATE_OVERRIDE_*` override inputs (documented subset) override the workspace config patch, but remain below CLI flags.
  - Implementation sketch:
    - Change the ordering in `crates/shell/src/execution/config_model.rs` `resolve_replace()` so override env is consulted after CLI flags and before workspace patch values.
    - Update/replace tests that currently enforce “workspace wins over override env”:
      - `crates/shell/tests/ev0_override_split.rs` (rename or invert expectations)
      - `crates/shell/tests/config_show.rs` precedence assertions
  - Risk/notes:
    - This is a behavior change; docs/ADRs/operator contract must be updated in lockstep with the code and tests.
    - Keep override env limited to the documented subset of config-shaped `SUBSTRATE_OVERRIDE_*` variables; do not introduce a generic “arbitrary env patch” surface.
    - Install/dev scripts MUST NOT export override inputs by default (they remain explicit one-off operator/test inputs).
  - Acceptance:
    - New/updated tests prove that when a workspace is enabled and `SUBSTRATE_OVERRIDE_*` is set, override env beats the workspace patch (and remains below CLI flags where flags exist).
    - `docs/project_management/adrs/implemented/ADR-0006-env-var-taxonomy-and-override-split.md`, `docs/project_management/adrs/implemented/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`, and `docs/reference/env/contract.md` reflect the updated precedence contract.

- **P1 – Warn on `config global show` when workspace config overrides**
  - Problem: `substrate policy global show` emits a clear note when a workspace policy overrides the global policy for the current directory, but `substrate config global show` does not emit an equivalent warning when `.substrate/workspace.yaml` overrides global config. This is confusing and makes it easy to misdiagnose “why does my config not match behavior?”
  - Work: when a workspace is active for the current directory, have `substrate config global show` print a note that workspace config overrides global config here and point users at the effective view (`substrate config show`).
  - Acceptance: parity with `policy global show` UX; message is shown only when a workspace override applies; docs/help updated if needed.

- **P1 – World-sync continuation (internal git v2: per-command history + compaction)**
  - Context: `docs/project_management/packs/active/world-sync/` implements host-only `workspace checkpoint`/`workspace rollback` via `.substrate/git/repo.git/` and explicitly does not implement the richer internal history described in `docs/project_management/future/INTERNAL_GIT.md`.
  - Work:
    - Record per-command internal git commits for filesystem-mutating commands and persist a mapping (trace span / command id ↔ internal git commit) for review/debug.
    - Add session/checkpoint tagging semantics and user-facing UX for undo/rollback at multiple resolutions (command / checkpoint / session).
    - Add retention + compaction (squash older command-level history into checkpoint/session commits) plus maintenance (`git gc`) so the store stays bounded.
    - Define a concurrency/locking strategy so multiple agents/sessions cannot corrupt the internal store.
  - Acceptance: recent per-command diffs are inspectable and undoable; older history remains rollbackable at checkpoint/session granularity; internal store size is bounded and operations are deterministic.

- **P1 – Structured anchor-guard events for caged `cd` (dedupe warnings; avoid heuristics)**
  - Problem: In persistent world sessions, the in-world anchor guard prints user-facing “caged root guard” warnings, but the host async REPL also performs host-side `cd` prediction/caging to keep `world_cwd` consistent when the world reports an unchanged cwd. This can produce duplicate warnings (guard + prediction) and forces brittle host-side suppression heuristics.
  - Goal: Make “anchor guard bounced `cd` back into the cage” a first-class structured signal from world-service so the host prints exactly once, consistently formatted, without stderr parsing or cwd-heuristic dedupe.
  - Proposed design (backwards-compatible; no protocol_version bump):
    - Add a per-command sentinel env var set by the in-world guard *only when it bounces* (example):
      - `__SUBSTRATE_GUARD_EVENT_B64=<base64 json>`
      - JSON payload fields:
        - `type: "caged_cd_blocked"`
        - `requested: <abs path>`
        - `anchor_root: <abs path>`
        - `returned_to: <abs path>`
        - `display_scope: "host"|"world"` (optional; controls `([Substrate Host])` vs `([Substrate World])`)
    - On persistent session completion, world-service already captures the child shell’s env; extract + decode this sentinel and:
      - include it as an optional field on `command_complete` (e.g., `guard_event` / `guard_events`),
      - strip the sentinel key(s) from the persisted env before storing (so it never leaks into later commands).
    - Host shell prints from the structured field (once) and stops printing prediction-layer warnings for world sessions (prediction remains state-only).
  - Work (concrete):
    - `crates/world/src/guard.rs`: set `__SUBSTRATE_GUARD_EVENT_B64` when bouncing a `cd` (and optionally stop printing the user-facing warning directly once the structured path is plumbed end-to-end).
    - `crates/world-service/src/pty.rs`:
      - plumb sentinel extraction from `PersistentChildEvent::Finished { env, .. }`,
      - extend `PersistentServerMessage::CommandComplete { .. }` with an optional `guard_event(s)` field (serde should ignore unknown fields for older clients),
      - strip sentinel keys from the session env before persisting.
    - `crates/shell/src/execution/repl_persistent_session.rs` and/or `crates/shell/src/execution/routing/dispatch/world_persistent_session.rs`: parse optional `guard_event(s)` and surface as a single user-facing warning line.
    - `crates/shell/src/repl/async_repl.rs`: delete any remaining prediction-layer warning emission for world sessions (keep only the cwd correction).
    - Optional: record `guard_event(s)` into `trace.jsonl` (span extra) so guard bounces are auditable without stdout/stderr inference.
  - Tests:
    - `world-service` persistent session test: run `cd ..` outside anchor and assert `command_complete.guard_event(s)` present; assert sentinel key stripped from persisted env.
    - `shell` client tests: unknown/missing `guard_event(s)` remains compatible; when present, message prints once.
    - End-to-end (Linux): `substrate` async REPL `cd ../` outside anchor prints exactly one caged warning line (no duplicates).
  - Acceptance: no duplicate caged warnings; no stderr parsing; persisted env does not retain sentinel; old/new clients interoperate via optional fields.

- **P1 – Policy-configured allowlist for host credential read paths (future enhancement)**
  - Problem: some environments will not store CLI login state in the backend’s default credential file location. Today the CLI adapter posture is “fixed backend-contract credential paths + env override,” which is safe but can be inflexible for enterprise/custom setups.
  - Work:
    - Add a strict policy surface to allowlist exact host paths (and later, tightly-scoped patterns) that are permitted for host credential reads, in addition to the backend-id allowlist gate (`agents.host_credentials.read.allowed_backends`).
    - Ensure `--explain` provenance attributes both gates (backend-id allowlist + path allowlist) and error messages are actionable without leaking secret values.
    - Document canonicalization rules, symlink handling, and “deny-by-default” posture.
  - Acceptance: operators can support non-standard credential locations without code changes while keeping least-privilege guarantees; misconfiguration cannot broaden to arbitrary host file reads.

- **P1 – Policy-driven world fs mode**
  - Problem: write permissions inside worlds currently depend on systemd hardening + overlay success, not on broker policy. Sensitive repos need a policy bit to force read-only worlds while other projects remain writable, without editing unit files manually.
  - Work: extend broker schema to accept `world.fs_mode = read_only|writable` (global + per-project), plumb into shell/world-service so PTY + non-PTY sessions honor it, and update docs/doctor to surface the active mode. Systemd units must allow `/home` writes so policy can enforce RO vs writable deterministically.
  - Acceptance: policy defaults to writable; flipping to read-only blocks writes (clear errors, trace telemetry); installers/docs explain the knob and tests cover both modes (with skip notes for hosts lacking overlay/cgroup permissions).

- **P1 – Add world OS/distro fields to doctor JSON**
  - Problem: `world doctor --json` reports host platform, but the `world` section does not identify the guest OS/distro/kernel. This is fine today (world is effectively “Linux”), but becomes important if we support multiple guest distros/images (Lima/WSL variants) and need to debug/report parity accurately.
  - Work:
    - Collect and report `world.platform` (e.g., `linux`) and a structured `world.os` block (e.g., `/etc/os-release` fields, `uname -r`, arch) from inside the world.
    - Add fields as optional/backwards-compatible (or bump schema with clear migration rules) and update any schema/fixtures/tests/docs.
  - Acceptance: `substrate world doctor --json` includes the world OS identity fields when world is enabled; missing data degrades gracefully; docs explain the semantics.

## Deferred Product Follow-Ups — Guest World Images / Linux `guest_rootfs`

- **P2 – Additional guest distro support beyond Ubuntu/Debian**
  - Problem: `guest_rootfs` is intentionally Ubuntu/Debian-first, but the long-term goal is guest-distro flexibility decoupled from the host distro.
  - Work:
    - Define how additional blessed guest images are introduced, validated, and documented.
    - Add per-image compatibility rules so unsupported images fail closed with clear remediation.
    - Extend validation to prove cross-host-distro behavior remains deterministic.
  - Acceptance: Substrate can support more than one blessed Linux guest distro without redefining the backend contract.

- **P2 – Non-APT guest-image provisioning support**
  - Problem: provisioning-time system-package support is currently manager-limited; additional guest distros will require non-APT provisioning paths.
  - Work:
    - Add manager-aware provisioning support for guest images that do not use APT (`pacman`, `dnf`, `yum`, `apk`, `zypper`, etc.).
    - Keep runtime `world deps current sync|install` probe-only for system packages across all managers.
    - Ensure manager selection is derived from in-world identity/probes rather than host PATH.
  - Acceptance: supported guest images can provision system packages with their native package manager while preserving the explicit provisioning-only posture.

- **P2 – User-imported or arbitrary guest images**
  - Problem: the first ship should bless one built-in image family, but advanced users may eventually want to bring their own guest images.
  - Work:
    - Define an allowlisted import/registration workflow for operator-supplied guest images.
    - Specify provenance, validation, and fail-closed handling for untrusted or malformed images.
    - Clarify how imported images participate in doctor output, compatibility checks, and remediation.
  - Acceptance: operators can register approved guest images without weakening image provenance or safety guarantees.

- **P2 – World image selection/pinning (Linux guest-rootfs, Lima, WSL)**
  - Problem: once we have guest-like world images across platforms, operators will want to pin the world OS/distro per workspace for reproducibility and parity (instead of relying on whatever Substrate ships by default or whatever the host happens to be).
  - Work:
    - Add a first-class “world image” identity and selection surface (workspace + global), surfaced in `substrate world doctor --json` and health output.
    - Support pinning per workspace (e.g., `.substrate/settings.yaml`) with a stable identifier and upgrade story.
    - Ensure `substrate world enable --provision-deps` and other guest-only flows validate that the active image is supported and report actionable remediation when it is not.
  - Acceptance: operators can pin the world image per workspace and see the effective image in doctor/health; behavior is reproducible across machines; unsafe/unknown images fail closed with clear guidance.

- **P3 – Make Linux `guest_rootfs` the default backend**
  - Problem: first ship should keep `host_native` as the default, but long term the stronger guest-backed model may become the preferred Linux execution posture.
  - Work:
    - Gather validation and operator evidence that `guest_rootfs` is stable enough for default use.
    - Define the exact rollout criteria and fallback/remediation expectations before changing the default.
    - Update docs, install defaults, doctor messaging, and smoke coverage if the default flips.
  - Acceptance: Linux can move to `guest_rootfs` by default without surprising existing operators or weakening fail-closed guarantees.

- **P3 – Dedicated CLI surface for guest-rootfs/image lifecycle**
  - Problem: first ship uses a script-first warm flow, but long term image lifecycle likely needs a first-class CLI contract.
  - Work:
    - Design a dedicated CLI for rootfs/image bootstrap, repair, reset, and inspection.
    - Keep the CLI aligned with existing world enable/provisioning flows rather than creating competing operator workflows.
    - Define cross-platform boundaries so Linux guest-rootfs lifecycle and future image lifecycle stay coherent with Lima/WSL.
  - Acceptance: operators can manage guest-rootfs/image lifecycle through a stable CLI surface instead of helper scripts alone.

- **P4 – Full VM-backed Linux backend (only if later justified)**
  - Problem: the current direction intentionally avoids a heavyweight Linux VM, but some future constraints may justify an optional VM-backed Linux world.
  - Work:
    - Define concrete criteria that would justify a VM-backed Linux backend instead of continuing to invest in host-kernel guest-rootfs execution.
    - Compare resource cost, isolation guarantees, and operator complexity against `guest_rootfs`.
    - If pursued, keep the backend contract explicit so VM-backed Linux does not silently replace the lighter-weight path.
  - Acceptance: any future Linux VM backend is introduced as an explicit, justified backend choice rather than accidental scope creep.

- **P1 fs_diff parity (agent HTTP + PTY)**
  - *Agent HTTP path:* Today only replay/local backends attach `fs_diff`; agent-routed non-PTY commands drop the diff. Extend `transport-api-types::ExecuteResponse` / `world-service` so `/v1/execute` returns `fs_diff: Option<FsDiff>` and update the shell to record it in completion spans. Acceptance: `fs_diff` shows up in `trace.jsonl` for agent HTTP runs.
  - *PTY sessions:* Interactive runs still lack filesystem diffs. Explore capturing post-exit diffs via overlayfs/copydiff and plumb the result through the PTY telemetry path so REPL + `substrate -i` sessions produce the same audit artifacts as non-PTY commands. Document caveats (long-running PTYs, partial diffs) and add tests to prove PTY diffs land in spans.

- **P1 – Record `world_id` for REPL/local exec spans**
  - Today `trace.jsonl` only shows `world_id` when we route a command through the world agent (non-PTY/PTY). Interactive REPL commands and other local exec paths still run in the world but log `world_id: null`, which makes auditing isolation harder.
  - Work: when the shell initializes a world session, capture the active world ID (if any) and populate it in the span metadata even for local/REPL commands. Ensure both shim and shell telemetry include it so every span reflects whether it executed in a world.
  - Considerations:
    - Plumb the ID through `ShellConfig`/`WorldRootSettings` so the logging layer can access it regardless of execution path.
    - Update tests to assert the field is set when worlds are enabled, and note the behavior change in tracing docs so operators know how to interpret the new data.

- **P1 – Capture caging/anchor in spans + replay verbose**
  - Problem: replay verbose output can’t show whether the recorded command was caged or what anchor was used because spans don’t persist that state.
  - Work: persist `caged` (bool) and `anchor_cwd` (string) in the span/replay_context, thread into replay reconstruction, and print in `--replay-verbose` with a warning when the replayed environment can’t honor the recorded caging/anchor.
  - Acceptance: trace entries include the new fields for caged sessions; replay verbose shows caging/anchor and warns on mismatch; docs/tests updated to cover the fields.

- **P1 – Policy/config lever for world exec-guard denylist (host-visible hardening)**
  - Problem: host-visible hardening needs an exec-time guard that denies host toolchain binaries (e.g., `~/.nvm/.../npm`). Today the denylist is easiest to prototype via env vars, but operators need a persistent **policy/config** lever to tune the denylist per org/workspace without per-invocation env overrides.
  - Work:
    - Add a broker-recognized policy surface (global + workspace patchable) that configures:
      - guard enabled/disabled (default: enabled when `world_fs.host_visible=true`)
      - denylist of path substrings to match against the resolved executable path
      - optional allowlist/override mechanism for exceptional cases
    - Define precedence and provenance in `--json` explain/doctor output (effective values + source).
    - Keep env vars as a diagnostics-only override (explicitly documented as weakening hardening).
  - Acceptance:
    - Operators can configure the denylist via policy/config files (no env vars required).
    - Effective config is visible in `--json` explain/doctor output with provenance.
    - Default behavior remains “deny common host toolchains” in host-visible worlds unless policy explicitly relaxes it.

- **P1 – Configurable world env forward allowlist (host-visible hardening)**
  - Problem: host-visible hardening needs a strict env-forwarding posture by default (avoid ambient host coupling), but some environments require forwarding a small set of host env vars (locale/terminal/timezone) and may need to extend that set explicitly.
  - Current direction:
    - Introduce a config lever `world.env.inherit_from_host=true|false` (default `false`) for forwarding a small built-in safe set.
    - Add a configurable extension `world.env.allow_list=[...]` to extend the forwarded set in a controlled way.
  - Work:
    - Define exact precedence (workspace over global) and provenance in `config explain` / doctor JSON.
    - Ensure forwarded keys are allowlist-only and do not include toolchain/secrets vars by default.
  - Acceptance:
    - Operators can extend the forwarded env keys without per-invocation env overrides.
    - Default remains strict (no host env forwarded unless explicitly enabled).

- **P1 - TCP bridge for agent (Cross-platform: Linux/macOS/Windows)**
  - Goal: Provide an optional loopback TCP endpoint that bridges to the agent UDS for environments/tools requiring TCP.
  - Current state:
    - **macOS** – already implemented via `vsock-proxy` (Lima forwarder exposes `127.0.0.1:17788`) with SSH-UDS fallback.
    - **Windows/WSL** – implemented by `substrate-forwarder` when launched with `--tcp-bridge` (scripts default to `127.0.0.1:17788`).
    - **Linux** – missing: needs optional systemd socket/service (e.g., `socat`) to expose `127.0.0.1:17788 → /run/substrate.sock`, plus installer/world-enable wiring and docs.
  - Security: Loopback only; document that it is a fallback, not a replacement (no TLS/external exposure).
  - Acceptance (Linux parity):
    - `curl -sSf http://127.0.0.1:17788/v1/capabilities` works whenever the bridge is enabled.
    - Shell auto-selects TCP only when UDS is unavailable and the bridge is active, matching macOS/Windows behavior.

- **P1.5 - JSON Mode Plan**
  - Plan: `docs/project_management/future/json-mode/json_mode_plan.md`

- **P2 – World deps install UX cleanup**
  - `substrate world deps current install` only works for items defined in the world-deps inventory (built-ins plus `$SUBSTRATE_HOME/deps/` and `<workspace_root>/.substrate/deps/`). Users who try to install language-level packages (pip/npm) or tools missing from the inventory get confusing errors, especially on dev installs where `$HOME` is read-only inside the world.
  - Improvements:
    - Document exactly what the inventory covers and how to install unsupported packages safely (e.g., virtualenv/pip --target, npm global installs, etc).
    - Consider adding backends for common package managers so commands like `substrate world deps current install pip:package` can proxy installations into the world deps prefix.
    - Ensure dev-install paths and env detection work inside the world so the helper can locate binaries/scripts.
  - Goal: make `world deps` a predictable way to provision world tooling or gracefully direct users to the correct workflow when an item isn’t supported yet.
  - Follow-up: expand built-in inventory coverage and improve `current show --explain` remediation for common missing/blocked cases.

- **P2 – Session listing/resume UX**
  - Pain: Each `substrate` invocation mints a fresh session ID recorded in `trace.jsonl`. Users who want to resume a previous REPL session or correlate spans currently have to parse that file manually or export `SHIM_SESSION_ID` themselves.
  - Idea:
    - `substrate sessions list` (or similar) to show recent session IDs, timestamps, and last known working directories by parsing `trace.jsonl`.
    - `substrate sessions resume <session_id>` (or a `--resume` flag) that sets `SHIM_SESSION_ID` before launching the shell so new spans append to that session. Replay remains per-span, but this improves DX for continuing a workflow.
  - Considerations:
    - The trace log can grow; listing needs to be efficient (maybe keep an index or show the last N sessions).
    - Handle missing/invalid directories gracefully when resuming and document what context is restored (only tracing metadata, not the world state).
    - Update docs to explain session IDs vs. span IDs and the new resume workflow.

- **P2 – World-disabled UX (no-world installs)**
  - Pain: When Substrate is installed with `--no-world`, commands still probe systemd/agent sockets and emit warnings (inactive socket, agent replay unavailable, cgroup/netns/overlay fallbacks) even though the world backend is intentionally disabled.
  - Improvements:
    - `substrate world doctor` should short-circuit when world is disabled and report a clear “world disabled/not provisioned” status (JSON/text) without probing systemd/socket.
    - Replay should detect disabled/missing world early and emit a single “world disabled/no agent installed; staying host-only” message instead of attempting world creation and logging cgroup/netns/overlay warnings.
    - `world deps` and related subcommands should surface that the world is disabled by install/config/env before implying a broken agent.
  - Goal: make host-only installs quiet and explicit, avoiding misleading “agent unavailable” warnings when the world was intentionally skipped.

- **P2 – macOS REPL can occasionally miss prompt return after a correctly blocked curl**
  - Symptom: On March 28, 2026, manual macOS/Lima validation of restrictive `world.net.filter=true` mostly behaved correctly: allow-all allowed traffic, restrictive allowlists fast-failed blocked `curl` requests, and `substrate -c` stayed healthy. In one unreproduced REPL run with `net_allowed=["google.com"]`, `curl spensermcconnell.com` printed the expected fast-fail `curl: (7) Failed to connect ...` error, but the `substrate>` prompt did not return afterward. Repeated `^C` / arrow-key input was echoed as raw control characters until the session was abandoned. Fresh terminals immediately afterward did not reproduce the issue.
  - Notes:
    - This does not look like a netfilter enforcement bug; the blocked connect was already rejected correctly.
    - The symptom was only seen in the interactive REPL/PTTY path, not in non-PTY `substrate -c`.
    - Most likely surface is macOS PTY-over-SSH/WebSocket session lifecycle or prompt-redraw state after a fast stderr-only failure.
  - Work:
    - Capture the relevant session trace and PTY lifecycle events the next time it reproduces.
    - Inspect whether the REPL misses a command-complete / idle transition after a rapid blocked-command exit.
    - Add stress coverage for repeated blocked `curl` failures in macOS/Lima persistent sessions.
    - Verify local terminal raw-mode cleanup and prompt reentry after blocked-command failure.
  - Acceptance: repeated macOS REPL runs under restrictive allowlists always return to `substrate>` after blocked `curl` failures, and no raw control characters are echoed into the terminal after command exit.

- **P3 – Interactive configuration commands**
  - Add shell built-ins/commands (`:config`, `:profile load`, `:world status`,
    `:shims status`, etc.) to view and adjust settings without restarting.
  - Surface doctor/shim status inside the REPL so users don’t have to exit.

- **P3 – Interactive install/setup walkthrough**
  - New users currently have to manually edit configs or run individual commands to understand world/caged/anchor/policy settings. An interactive setup wizard (e.g., `substrate setup`) could walk them through enabling/disabling worlds, picking anchor modes/paths, choosing caged vs uncaged behavior, configuring manager snippets, and reviewing policy defaults.
  - Considerations: support non-interactive modes (`--defaults`, `--profile`), validate paths, respect existing configs (idempotent), and clearly document the choices made.

- **P4 – macOS bootstrap automation**
  - Build a signed installer script (or helper binary) that validates macOS version/virtualization support, installs required Homebrew packages (`lima`, `jq`, `openssh`, `coreutils`, `gnused`, `gnu-tar`, `gettext`), provisions Lima, deploys `substrate-world-service`, and runs doctor checks automatically.
  - Acceptance: Fresh macOS host can run a single installer command and end up with the Lima VM, agent, and smoke test ready without manual steps.

- **P4 – macOS installer dependency automation**
  - Enhance `scripts/substrate/install-substrate.sh` to auto-install required macOS tools (e.g., `envsubst` via Homebrew `gettext`) when missing, falling back to clear guidance if no supported package manager is detected.
  - Acceptance: Fresh macOS host without gettext can run the installer end-to-end without manual prerequisite setup. 

- **P4 – Cleanup skips for already-removed worlds**
  - `substrate world cleanup` still enumerates every historical `wld_*` entry and prints “manual” nft/cgroup commands even when those resources no longer exist (the helper treats ENOENT as failure). Users who already deleted the leftovers get spammed with scary warnings on each run.
  - Improvements:
    - Probe nft tables, netns entries, and cgroup directories before attempting removal; skip entries that are already absent.
    - Treat ENOENT from `nft delete`/`rm -rf` as success so the CLI reports “all clear” when nothing remains.
    - Summarize results with a clean “nothing to clean” message instead of reprinting stale IDs.
  - Acceptance: running `substrate world cleanup` on a host with no lingering worlds produces a success summary with no manual commands; lingering resources still result in actionable instructions.

- **P5 – Doctor/health output UX**
  - The current `substrate world doctor`, `substrate shim doctor`, and `substrate health` outputs are verbose but not well structured: actionable issues are buried in large sections (e.g., listing every missing manager even when the host never had them), and the ordering makes it hard to see what needs attention.
  - Improvements:
    - Reorganize sections so failures/warnings surface first, followed by informational details.
    - Clearly separate “host missing manager” vs. “host has manager, world missing it”.
    - Consider a concise summary view with optional `--verbose` output for the full details.
  - Goal: make the doctor/health commands feel like a coherent report where users can quickly identify what needs fixing. 

- **P5 – Consolidate world enable toggle under `[world]`**
  - Today the on/off switch for world isolation lives under `[install]` as `install.world_enabled`. That placement dates back to the installer metadata phase, but it creates a split experience: all other runtime knobs (`anchor_mode`, `caged`, etc.) live under `[world]` while the most important one sits elsewhere. As we expand world configuration, the ergonomics would be cleaner if users could manage every world option via `world.*` keys.
  - Proposed direction: introduce `world.enable` (boolean) as the canonical key, treat `install.world_enabled` as a compatibility alias, and update docs/installers to write/read the new location. That aligns the schema semantically (all world behavior in one table) and makes `substrate config show/set` more intuitive (`world.enable=false` instead of `install.world_enabled=false`).
  - Migration considerations: loaders should prefer `world.enable` when present but still honor the old key, `substrate config set` needs to write both until we formally deprecate the installer key, and docs/tooling (install scripts, doctor output) must call out the new name plus the legacy alias so existing installs don’t break.

- **Later – Document `--uncaged` as a diagnostics-only escape when worlds are enabled**
  - Users occasionally expect `--uncaged` to grant broader filesystem access inside the world backend, but in reality it just removes the anchor guard; the process remains confined to the world’s overlay. We should update docs (`docs/CONFIGURATION.md`, `docs/USAGE.md`, `docs/WORLD.md`) and CLI help to clearly state that `--uncaged` inside a world is intended for Substrate troubleshooting (inspecting overlay internals, diff dirs, etc.) and shouldn’t be used for normal workflows.
  - Acceptance: CLI help text and documentation call out that uncaging a world session doesn’t break isolation and is mainly for debugging; examples show `--uncaged` as a dev tool rather than a standard option.

- **Later – Command hook engine**
  - Implement the hook system described in `docs/project_management/future/COMMAND_HOOKS_IMPLEMENTATION_PLAN.md`
    (hook definitions, matcher/condition/action pipeline, shim & shell integration).
  - Provide `~/.substrate/hooks.yaml` scaffolding plus CLI commands to list/enable/disable hooks.  

- **Later – Investigate scoped/named worlds (project/user/global)**
  - Idea: today every world session anchors to the launch directory and gets its own ephemeral overlay (`world.anchor_mode=project`). Consider adding explicit scopes such as “user world” or “global world” so multiple commands can share a long-lived overlay (faster warm-up, persistent tooling state) while still staying isolated from the host.
  - Considerations:
    - **Isolation & policy:** reusing a user/global world mixes state between repos. We’d need opt-in plus policy controls to avoid leaking data between unrelated projects.
    - **Lifecycle:** define commands to create/reset/destroy scoped worlds (`substrate world reset --scope=user`?) and document how cleanup works.
    - **CLI/Config UI:** extend config/flags (e.g., `world.scope = project|user|global`) and expose how this interacts with existing `anchor_mode`/`anchor_path`. For experimentation we could leverage `world.anchor_mode=custom` pointing at a shared directory to simulate the behavior.
    - **Backend changes:** the Linux backend would need to cache overlays keyed by scope instead of always generating new `wld_*` directories.
  - Not a priority now, but worth capturing so we can explore the UX/security balance later; any prototype should highlight how to emulate it today via `world.anchor_mode=custom` before adding official scopes.

- **Later – Shell env regression tests should force `SUBSTRATE_HOME`**
  - Pain: the `shell_env_*` integration tests only override `HOME`, so a user who has `SUBSTRATE_HOME` exported (e.g., after running the dev installer) leaks their real `~/.substrate` into the test harness. That causes false failures (`PATH` not shimmed, overlay snippets missing) whenever host metadata differs from the fixture expectations.
  - Fix: update `ShellEnvFixture`/`substrate_command_for_home` to set `SUBSTRATE_HOME` to the temporary home it creates (and pre-create config directories). The tests then stay hermetic even if the developer’s environment has custom prefixes.
  - Scope: treat as maintenance/reliability work—no product changes, just test harness hardening. Add a quick note in `DEVELOPMENT.md` once fixed so contributors know the tests are self-contained.

- **Later – Graph intelligence backend**
  - Finish the Kuzu-backed implementation outlined in `docs/project_management/future/PHASE_4_5_ADVANCED_FEATURES_PLAN.md`:
    ingestion pipeline, schema, query interface, and CLI.
  - Replace the current mock-only `substrate-graph` crate with a real backend.

- **Later – CLI help/flag audit**
  - Several subcommands inherit global flags that don’t make sense in context (e.g., `substrate world cleanup --help` shows `--world/--no-world` despite being world-only). Audit command help/flag surfaces, hide or override irrelevant globals, and ensure summaries/examples reflect intended usage.
  - Scope: review top-level/global flags vs per-command applicability, adjust clap definitions to suppress noise, and refresh docs/USAGE snippets with cleaned help output.
  - Acceptance: per-command help only shows relevant options; cleanup/help examples confirm noisy flags are removed or clearly explained.



## Hardening / Quality
- **P1 – Revisit and likely retire the remaining hidden `runtime_bootstrap_prompt(...)` host-orchestrator bootstrap path**
  - Context: public prompt-taking flows no longer use the hidden bootstrap prompt.
    - Public `substrate agent start` loads the caller’s `--prompt` into `plan.startup_prompt.prompt_text` in `crates/shell/src/execution/agents_cmd.rs`, then launches the owner-helper with `InitialExecPromptPlan::StartupPrompt { prompt, .. }` in `crates/shell/src/repl/async_repl.rs`.
    - Public `substrate agent turn` sends the caller’s follow-up prompt text through the private owner transport via `run_public_prompt_command()` / `request_private_prompt_stream()` / `submit_host_prompt_turn()` in `crates/shell/src/execution/agent_runtime/control.rs`.
    - The strongest in-repo proof today is `crates/shell/tests/agent_public_control_surface_v1.rs::public_start_turn_and_stop_emit_streaming_ndjson_and_authoritative_state`, which captures the fake backend stdin and asserts invocation 1 contains `"hello from start"` and invocation 2 contains `"hello from turn"`, while the startup invocation is `exec` rather than `resume`.
  - Remaining hidden path: `runtime_bootstrap_prompt(...)` is still reachable when `start_host_orchestrator_runtime_with_prepared_prompt(...)` is called with `initial_prompt == None` in `crates/shell/src/repl/async_repl.rs`.
    - This still happens in the normal async REPL host-orchestrator bootstrap path via `start_host_orchestrator_runtime_with_prepared(...)` / `start_host_orchestrator_runtime(...)`.
    - It is therefore not dead code and still injects an internal hidden prompt on non-public REPL bootstrap, even though the public agent-control paths have been de-bootstrapified.
  - Why revisit soon:
    - The codebase now has two materially different startup semantics:
      - public agent-control start/turn = operator prompt text is the real prompt,
      - legacy/internal REPL bootstrap = hidden bootstrap prompt when no explicit initial prompt is provided.
    - This split is easy for humans and future agents to misread, especially when tracing regressions around startup ordering, ownership establishment, or prompt provenance.
    - Keeping the hidden path around may be correct for now, but it should be a deliberate contract, not accidental drift.
  - Work:
    - Confirm whether the REPL still needs `runtime_bootstrap_prompt(...)` at all, or whether it should move to the same explicit-prompt model as public agent control.
    - If it must remain, document the contract clearly in `docs/USAGE.md` / `docs/WORLD.md` / agent-control docs: which entrypoints still synthesize a hidden bootstrap prompt, and why.
    - If it can be removed, delete the fallback branch in `start_host_orchestrator_runtime_with_prepared_prompt(...)` and update tests to prove there is no hidden prompt injection anywhere on host public-control paths or REPL startup.
    - Add or tighten regression coverage around prompt provenance so future changes cannot silently reintroduce bootstrap-composed prompts into public `agent start` / `agent turn`.
  - Acceptance:
    - Future readers can answer, from docs and tests, whether any shipped entrypoint still uses a hidden bootstrap prompt.
    - Either the hidden fallback is removed, or it remains with explicit documentation and tests proving its exact reachability boundary.
- Document backend divergence
  - Clarify why shell uses world-service and replay uses LinuxLocalBackend; outline future convergence plan.
- Redaction hardening for “command body” tracing (preexec + future trace surfaces)
  - Problem: some debug-grade event families (e.g., bash preexec `builtin_command`) can include raw command bodies that may contain tokens/headers/secrets. Today we omit bodies from canonical trace for safety; to safely include bodies we need hardened, shared redaction across the codebase.
  - Work:
    - Define a shared redaction module usable by shell/shim/world-service/preexec (flag consumes next arg, `--flag=value`, URL credentials, headers, exports, common token env patterns).
    - Add tests that prove representative leaks are redacted (tokens, Authorization headers, proxy URLs, `export FOO_TOKEN=...`, etc.).
    - Add an explicit opt-in mode to include redacted command bodies in canonical trace for debug profiles (and ensure it is non-triggerable by default for ADR-0029 routing).
  - Acceptance: command-body logging is safe-by-default, can be enabled explicitly for debug with strong redaction + tests, and is clearly labeled/filtered.
- Heavy isolation wiring (optional)
  - Provide a mode to invoke `LinuxIsolation::apply()` for non-PTY when capabilities allow; keep current lightweight/PTy approach as default.
  - Acceptance: gated by capability checks; clear degrade messages.
- Linux world fs last-resort fallback: copy-diff execution
  - If kernel overlayfs is unhealthy and `fuse-overlayfs` is unavailable, allow a copy-diff strategy to keep world usable for basic workflows.
  - Safety requirement: copy-diff MUST preserve the project bind-mount enforcement (private mount namespace bind of the work tree onto the host project path) to prevent absolute-path escapes.
  - Observability: record selected strategy + fallback reason in trace and surface in `substrate world doctor --json`.
- Docs consistency
  - Document non-PTY agent auto-start behavior and fallback; update REPLAY/WORLD runbooks accordingly.

## Later
- Policy/broker expansion
  - Richer allowlist/egress policy plumbing into world sessions.


## Later - Windows Transport — Cross‑Cutting Backlog
User‑Scoped Named Pipe Default

Context: We standardize on a single host pipe
`\\.\\pipe\\substrate-agent`. On shared or multi‑session Windows systems this
name can be owned by a stray forwarder, an IT‑deployed service, or another
user session. While the “single‑instance guard + friendly error” mitigates
confusion at startup, adopting a user‑scoped default eliminates cross‑user
collisions by design.

Goal: Make the default forwarder pipe name user‑scoped (for example,
`\\.\\pipe\\substrate-agent-<SID>`), while preserving backward compatibility
and explicit overrides.

Design details
- Default name schema: `\\.\\pipe\\substrate-agent-<SID>` where `<SID>` is the
  current user’s Windows SID; fallback to `<USERNAME>` if SID retrieval fails.
  Keep `\\.\\pipe\\substrate-agent` as a documented legacy alias.
- Backward compatibility: If `SUBSTRATE_AGENT_PIPE` (or equivalent) is set to
  the legacy name, honor it. Doctor/warm should detect conflicts and print
  remediation.
- Display/telemetry: Sanitize endpoints in logs/telemetry (avoid leaking SID);
  use `substrate-agent-<user>` in human‑readable fields and keep the full path
  in debug logs only.
- Security: Maintain the current SDDL (SY/BA/IU GA) and
  `reject_remote_clients(true)`. Document that user‑scoped names reduce
  accidental cross‑session access but do not replace ACLs.

Code changes required (touch points)
- `crates/forwarder/src/main.rs`
  - Default CLI `--pipe` value to the user‑scoped name computed at runtime.
  - Emit both the effective path and a redacted display endpoint in logs.
- `crates/host-proxy/src/lib.rs`
  - Change `DEFAULT_AGENT_PIPE` to resolve at runtime via a helper that
    computes the user‑scoped name when no explicit config is supplied.
  - Update serde/display to accept both legacy and user‑scoped forms.
- `crates/world-windows-wsl/src/lib.rs`
  - Ensure `build_agent_client()` resolves the same default and exposes the
    chosen endpoint for telemetry.
- `crates/transport-api-client` (tests)
  - Update unit tests that assert the hard‑coded legacy pipe to accept the
    dynamic, user‑scoped default.
- `crates/shell` (tests/telemetry)
  - Adjust Windows tests that check `transport.endpoint` to tolerate the new
    default (or inject override during tests).

Scripts and docs
- `scripts/windows/wsl-warm.ps1`, `wsl-doctor.ps1`, `wsl-stop.ps1`
  - Default to the user‑scoped pipe; support `-PipePath` override and surface
    both the effective path and display value.
- Documentation updates
  - `docs/dev/windows_host_transport_plan.md`: call out the user‑scoped default
    and rationale.
  - `docs/dev/wsl_world_setup.md`: show the new default in examples and explain
    overrides.
  - `docs/dev/wsl_world_troubleshooting.md`: update entries for conflicts and
    how to identify the owning process (handle.exe), plus how to opt back to
    the legacy name for compatibility.
  - `docs/dev/windows_transport_external_overview.md`: update architecture
    diagrams and narrative to mention user‑scoped default.

Migration and compatibility strategy
- Soft roll‑out: keep the legacy name documented; scripts accept both and print
  a deprecation‑style hint when the legacy name is detected.
- Env/config knobs: add `SUBSTRATE_AGENT_PIPE` or forwarder TOML `pipe_path` to
  force legacy behavior for environments that depend on it.
- CI/tests: set `SUBSTRATE_AGENT_PIPE=\\.\\pipe\\substrate-agent-test` in CI to
  avoid SID dependencies in headless runners.

Acceptance criteria
- Fresh Windows setup uses the user‑scoped default end‑to‑end (forwarder,
  doctor/warm, shell, host‑proxy).
- Legacy name still works when explicitly configured; friendly warnings appear
  with migration hints.
- No collisions between different signed‑in users or sessions by default.
- Updated docs are markdownlint clean; evidence log captures a successful warm
  run using the user‑scoped default.

Risks / considerations
- Third‑party automations hard‑coding the legacy name will need overrides;
  mitigate via clear warnings and a deprecation window.
- Some admin workflows may prefer a machine‑wide pipe; document how to force
  the legacy name alongside hardening advice (service account, SDDL, audit).
- Backlog – Stateful replay & session branching
  - Today replay simply re-runs a single span’s command in the current working tree; it doesn’t restore prior state or let you walk an entire session. To make replay genuinely useful for “time travel” debugging, we need:
    - The ability to list/replay a contiguous range of spans (e.g., “replay spans 3–7 of session X in order”).
    - Snapshot/rollback support so you can restore the filesystem/environment to how it looked at a given span before replaying or branching.
    - Branching semantics (fork a “what-if” timeline from a session and track it as a branch of the original session).
  - Considerations:
    - Requires capturing enough state (e.g., snapshots, artifacts, or overlays) to restore the workspace reliably.
    - Needs a UX for selecting span ranges, stepping through spans, and labeling branches.
    - Coordination with world backend: snapshots might leverage overlayfs layers or copy-diff artifacts.



## DONE -- IMPLEMENTED


- ~~**P0 –Socket-activated world-service service~~ **(Done)**
  - Current provisioning only installs `substrate-world-service.service`; the agent binds `/run/substrate.sock` itself and must stay running. Introduce a matching `.socket` unit so systemd listens on the socket, launches the agent on demand, and restarts it transparently.
  - Work items:
    - Update `world-service` to accept an inherited listener (LISTEN_FDS) in addition to binding directly; keep the existing code path for non-systemd environments.
    - Teach `ensure_world_service_ready()` to tolerate socket-activated setups (socket already present, service only starts when probed).
    - Extend Linux/Lima/WSL installers, world-enable helper, and uninstall scripts to deploy/remove both `.service` and `.socket` units.
    - Refresh docs/tests to describe the new flow; ensure guidance around sudo/user installs and permissions highlights the benefit.
  - Can run in parallel with other features—touches agent + provisioning scripts but does not block shell, tracing, or UX work.

- ~~**Replay polish – isolation + verbose scopes~~ **(Done)**
  - *Isolation follow-up:* Most of the Phase 4.5 isolation plan shipped (per-replay netns + nft scoping), but optional enhancements remain: nft cgroup matching fallback, documentation updates (`COMPLETE_FIXES_PHASE4_PRE45.md`), and diagnostic tooling for leftover netns/rules.
  - *Verbose scopes:* When running `substrate --replay --replay-verbose`, show a concise `scopes: [...]` line next to the “world strategy” output so operators can see which policy scopes were exercised per replay.
  - *Clear warnings:* Differentiate shell vs. replay world warnings—shell path messages should explicitly say “shell world-service path”, while replay warnings keep the `[replay] …` prefix.
  - *Default-to-world replay tests:* Add integration coverage for the default world-on path, `--no-world`, and env opt-out so replay regressions are caught automatically.
  - Treat all of the above as a single replay-focused bucket so backend polish and CLI visibility ship together (shared tests/docs).  


- ~~Health command manager mismatch bug~~  **(Done)**
  - `substrate health` currently reports “attention required” whenever optional manager detection hooks (direnv, asdf, conda, etc.) aren’t found on the host, even though the host never had them. We only care when the world and host detection disagree (host has a manager, world doesn’t), not when both sides are missing a manager entirely.
  - Fix: adjust health summary logic to only flag mismatches when the host reports a manager and the world fails to mirror it. Missing managers that the host doesn’t have should not trigger an “attention required” status.

- ~~Top Priority – Global configuration UX~~ **(Done)**
  - Implementation: `substrate config init` scaffolds `~/.substrate/config.yaml`, `config show` renders YAML/JSON with redaction hooks, and `config set` applies multi-key updates atomically with schema validation.
  - Docs/installer output highlight the new subcommands across Linux/macOS/Windows, and the precedence stack (flags → directory config → global config → env) remains unchanged.

- ~~Auto-start world-service on shell startup (Linux)~~ **(Done)**
  - Implementation: `run_shell()` now initializes the Linux backend, flips
    `SUBSTRATE_WORLD=enabled`, sets `SUBSTRATE_WORLD_ID`, and uses
    `ensure_session()` before handling commands (`crates/shell/src/lib.rs:1576-1620`).
  - Notes: macOS and Windows paths share the same default-on behavior; the Linux
    helper still attempts to spawn `world-service` if `/run/substrate.sock` is
    stale (`crates/shell/src/lib.rs:3680-3687`).

- ~~Non-PTY agent auto-start parity (shell)~~ **(Done)**
  - Implementation: Non-PTY routing now calls `ensure_world_service_ready()` when
    worlds are enabled (Linux HTTP path) and records transport metadata for
    macOS/Windows agent calls (`crates/shell/src/lib.rs:3680-3703`, `3560-3663`).
  - Result: The shell only falls back to host execution after a single warning
    when the agent cannot be reached; routine runs stay in-world by default.

- ~~Top Priority – Fix REPL busy-spin / async agent output~~  **(Done)**
  - Resolve the Reedline/crossterm busy loop that pegs a CPU core when idle (gate on TTY,
    introduce backoff, or adopt the async event loop from the Phase 4 concurrent output design).
  - See `docs/project_management/future/PHASE_4_CONCURRENT_OUTPUT_DESIGN.md` for context and remediation notes.
  - Implement the async agent output path so events can stream without prompt corruption.

- ~~macOS binary distribution~~ **(Done)**
  - Publish prebuilt `substrate` and `substrate-world-service` artifacts (e.g., GitHub releases or Homebrew formula) so operators do not need a Rust toolchain to onboard.
  - Acceptance: Installer/bootstrap script can download versioned binaries; manual instructions reference the published artifacts instead of local builds.

- ~~macOS backend enablement (Lima)~~ — **DONE (2025-09)**
  - Implemented `world-mac-lima` agent calls via `transport-api-client` with VSock/SSH fallbacks.
  - Shell now routes macOS commands through Lima, ensures VM/forwarding, and mirrors Linux telemetry.
  - Acceptance met: `scripts/mac/smoke.sh` validates non-PTY, PTY, and replay; docs refreshed (`docs/WORLD.md`, `docs/dev/mac_world_setup.md`, `docs/INSTALLATION.md`).
