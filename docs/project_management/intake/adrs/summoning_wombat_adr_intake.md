# ADR Intake Sheet

## 1. Codename + Created + Status

- Codename: `summoning_wombat`
- Created: 2026-02-20T00:48:09Z
- Status: brainstorming

## 2. Working Title (tentative)

Make `substrate world enable` work after `dev-install-substrate.sh --no-world`.

## 3. Problem / Motivation

- Dev installs done with `scripts/substrate/dev-install-substrate.sh --no-world` write `~/.substrate/config.yaml` with `world.enabled: false`, so `substrate world doctor` reports “world isolation disabled…”.
- `substrate health` (and `substrate shim doctor`) still attempt to compute “world deps applied” status even when the world is disabled via config, which leads to confusing failures like:
  - `world backend unavailable: world-agent binary not found`, or
  - `world backend unavailable: world-agent readiness probe failed`.
- On Linux dev installs specifically, `--no-world` currently also skips building/staging the `world-agent` binary into the “version dir” layout that `substrate world enable`’s helper expects (e.g., `<repo>/target/bin/linux/world-agent`).
- The main installer path (`scripts/substrate/install-substrate.sh`) copies an entire release bundle into `~/.substrate/versions/<version>/` even when `--no-world` is used; release bundles include `world-agent` under `bin/world-agent` (Linux) or `bin/linux/world-agent` (macOS/Windows guest agent). So the *missing `world-agent` artifact* problem appears dev-install-specific; the *world-disabled UX/noisy probing* problem is shared.
- Result: “install with `--no-world` then later run `substrate world enable`” is not reliably execution-ready for dev installs without manual extra steps (build `world-agent`, then provision), and the diagnostics read like breakage rather than “intentionally disabled/not provisioned”.

## 4. Proposed Outcome

- After a dev install with `--no-world`, a developer can run `substrate world enable` and have it provision successfully (or fail with a single, correct, actionable remediation that unblocks them).

## 5. Non-Goals

- Redesigning world isolation, overlay/copy-diff behavior, or the world-agent API.
- Expanding world-deps inventory coverage or changing dependency semantics.
- Making Windows world enable supported (explicitly not supported today).
- Reworking systemd unit hardening/capabilities beyond what’s needed for this slice.

## 6. Constraints / Invariants

- Security: provisioning remains an explicit privileged operation (sudo/systemd); do not introduce silent privilege escalation.
- UX: `--dry-run` must remain no-change; failure modes should be concise and actionable.
- Compatibility: production installs (release bundles) must keep working; avoid requiring `cargo` outside dev/source checkouts.
- Platform: Linux first; macOS Lima path may require separate treatment (follow-up ADR if needed).

## 7. Interfaces / Contracts (concrete changes)

Candidate A scope (this intake):

- `substrate world enable` behavior when invoked from a dev-install “version dir” (e.g., `…/target/`) that lacks `bin/linux/world-agent`.
- Potential changes to `scripts/substrate/dev-install-substrate.sh` and/or the `world-enable.sh` helper expectations for where `world-agent` is found.
- Error/reporting contract: when `world-agent` is missing, emit a deterministic remediation (“build this artifact” / “rerun dev-install with …”) rather than surfacing downstream socket/readiness failures.

Out-of-scope for Candidate A (capture for follow-ups):

- `substrate health` / `shim doctor` should skip world-deps probing when `world.enabled=false` in config (see `docs/BACKLOG.md` “World-disabled UX”).
- Doctor output wording (“disabled by effective config (--no-world)”) should ideally reflect the actual source (config/env/flag).
- Production installer verification: ensure no regression for `install-substrate.sh --no-world` → `substrate world enable` (should already work because the bundle includes `world-agent`).

## 8. Options

### Option 1 — Build/stage `world-agent` during `substrate world enable` when missing

**Description**
When the enable flow detects it is running from a dev-install version dir (typically `target/`) and cannot find `bin/world-agent` or `bin/linux/world-agent`, it performs a targeted build (`cargo build -p world-agent --release` or the selected profile) and stages/symlinks the resulting binary into the expected location before provisioning systemd.

**Pros**
- One-command “enable” experience for dev installs.
- Keeps dev-install `--no-world` fast.

**Cons**
- Requires `cargo` + a usable workspace context at enable time (not always true for production installs).
- More moving parts in the enable path; harder to keep deterministic across platforms.

**Risk notes**
- Must avoid running arbitrary builds when invoked from non-repo installs; needs careful detection and clear messaging.

### Option 2 — Always build/stage `world-agent` during dev-install even with `--no-world` (but skip provisioning)

**Description**
Change `scripts/substrate/dev-install-substrate.sh --no-world` so it still builds `world-agent` (Linux host) and stages it into `target/bin/linux/world-agent` (via the existing `ensure_release_bin_bridge`), but continues to skip provisioning/systemd and writes `world.enabled: false`. Later, `substrate world enable` can provision using the already-staged artifact.

**Pros**
- Keeps `substrate world enable` simple and aligned with production assumptions (“binary already present in version dir”).
- Minimal change surface: dev-install script + documentation/expectations.

**Cons**
- `--no-world` dev installs get slower and consume more disk (building `world-agent` even when you might never enable it).

**Risk notes**
- Need to ensure the built artifact matches the enable default profile (`--profile release`) or define how mixed debug/release enable is handled.

### Option 3 — Keep behavior; improve failure messaging + docs (explicit 2-step enable)

**Description**
Do not change build/provision behavior. Instead, make the enable path (or preflight checks) detect missing `world-agent` in the version dir and exit early with explicit remediation:
`cargo build -p world-agent --release` (or rerun dev-install without `--no-world`), then rerun `substrate world enable`.

**Pros**
- Smallest implementation risk; no new implicit build step.
- Clearer mental model (enable requires an artifact + provisioning).

**Cons**
- Still multi-step; “enable later” remains less convenient for dev installs.

**Risk notes**
- Needs careful wording so users don’t confuse “agent binary for provisioning” with “socket/service is running”.

## 9. Recommendation (tentative)

Tentative: **Option 2** for Linux dev installs, because it preserves the existing “enable provisions from version dir artifacts” contract and keeps `substrate world enable` aligned with production installs.

Choose **Option 1** when we strongly prefer “enable builds what it needs” and can reliably detect a source checkout + cargo environment.

Choose **Option 3** when we want the smallest vertical slice with minimal behavioral risk and are okay with a 2-step dev workflow.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): Make “dev-install `--no-world` → `substrate world enable`” an execution-ready flow on Linux (one behavior delta).
  - Slice 1: Add preflight detection for missing `world-agent` in the version dir and convert current downstream failures into a single actionable error (even if we later implement Option 1/2).
  - Slice 2: Implement one of Option 1 or Option 2 to ensure `world-agent` is present for provisioning.
- ADR Candidate B (follow-up): World-disabled UX cleanup (`health`/`shim doctor`/`world deps`/`world doctor` should avoid probing world backend when `world.enabled=false`).
- Candidate C (maintenance / tack-on): Documentation cleanup for the “world enabled” toggle and metadata format (docs/examples still reference `install.world_enabled` / `config.json`, but current code + installers use `world.enabled` in `config.yaml`). Consider folding into Candidate A or B unless we decide to change runtime compatibility behavior.
- ADR Candidate D (follow-up): Dev-install layout parity with production installs (populate `~/.substrate/versions/dev/` with `bin/` + `scripts/` so helper discovery + artifact staging don’t depend on `<repo>/target/` conventions).
- ADR Candidate E (follow-up): Doctor/health messaging should report *why* world is disabled (CLI flag vs config vs env), instead of always implying `--no-world`.

## 11. Acceptance Criteria Draft (observable outcomes)

- After `scripts/substrate/dev-install-substrate.sh --no-world`, running `substrate world enable` no longer fails with low-level “world-agent binary not found/readiness probe failed” without remediation.
- The enable flow either provisions successfully or prints a single actionable step that unblocks provisioning (build/stage guidance).
- If provisioning succeeds, `~/.substrate/config.yaml` ends with `world.enabled: true`.
- `substrate world doctor --json` shows `world_enabled=true` and `ok=true` after successful provisioning (Linux).
- `substrate health` no longer reports a world-deps error that is solely caused by the missing `world-agent` binary after the chosen fix is applied.
- No regression: installs from `install-substrate.sh --no-world` continue to be enable-able later via `substrate world enable` (bundle contains `world-agent`).

## 12. Open Questions / Unknowns (with priority)

- P0: What is the desired dev meaning of `--no-world`? “Skip provisioning only” (build ok) vs. “Skip everything world-related” (skip build too).
- P0: Should `substrate world enable` be allowed to invoke `cargo build` automatically (Option 1), or must it remain provisioning-only (Option 2/3)?
- P1: Which build profile should be authoritative for staging `world-agent` on dev installs (debug vs release), given `substrate world enable --profile` defaults to `release`?
- P1: Should we gate any new behavior behind an explicit flag/env (e.g., `SUBSTRATE_WORLD_ENABLE_BUILD=1`) to avoid surprises?
- P2: Do we want a dedicated “world prerequisites” check that can be run without any socket probing when world is disabled?

## 13. “Ready to Draft ADR?” checklist

- [ ] One behavior delta is locked (Option 1 vs 2 vs 3).
- [ ] Scope boundaries agreed (Linux-only vs cross-platform).
- [ ] Acceptance criteria match intended UX.
- [ ] Open questions P0 answered.
