# ADR Intake Sheet

## 1. Codename + Created date/time + Status

- Codename: `stashing_ferret`
- Created: 2026-02-20T19:09:29Z
- Status: brainstorming
- Dependencies: [`detecting_badger`]
- Related intakes (coordination only): `provisioning_otter`

## 2. Working Title (tentative)

Persist detected Linux distro/pkg-manager in `$SUBSTRATE_HOME/install_state.json` for later diagnostics + guidance

## 3. Problem / Motivation (3–6 bullets)

- We already do best-effort package manager detection during install (`scripts/substrate/install-substrate.sh`) to install host prerequisites, but that detection is not persisted in a way that Substrate can reuse later.
- Users can experience confusing “wrong package manager” guidance later (example: a Manjaro host seeing apt-flavored guidance or attempts elsewhere), and there’s no stable “what did we detect at install time?” record to debug against.
- `~/.substrate/install_state.json` already exists as the canonical installer metadata store (schema `version=1`) used for group/linger cleanup on uninstall; it’s a natural place to store host platform detection.
- Today `install_state.json` is only written when host-state “events” occurred (group/linger changes). On many successful installs, that means the file may not exist at all — which makes it unreliable for later diagnostics.
- We need a small, additive, backwards-compatible way to persist host distro detection + chosen package manager so other tooling (installer retries, `substrate * doctor`, future provisioning flows) can present correct, actionable guidance.

## 4. Proposed Outcome (1–3 bullets)

- Linux installs record `distro_id`, `distro_like`, and chosen `pkg_manager` in `$SUBSTRATE_HOME/install_state.json`.
- Metadata is recorded even when no group/linger changes occurred (install_state becomes reliably present after install).
- Later UX surfaces can read this metadata to present correct “manual install” guidance (and to reduce ambiguity in support/debug).

## 5. Non-Goals (explicit)

- Changing world-deps provisioning behavior or adding new world package manager support (pacman/apk/etc) in this ADR.
- Changing `install_state.json` schema version (must remain `schema_version=1` for uninstall compatibility).
- Implementing a full “distro support matrix” policy; this is just persistence + diagnostics.

## 6. Constraints / Invariants (security, UX, compatibility, performance)

- **Compatibility:** must not break existing uninstall flows that require `schema_version == 1`.
- **Additive:** new fields must be optional and safe to ignore by older code/scripts.
- **UX:** if metadata is missing or unreadable, behavior must fall back to best-effort runtime detection (never hard-fail).
- **Privacy:** avoid recording sensitive info (no full environment dump, no hostnames); stick to `/etc/os-release` fields + chosen manager string.
- **Performance:** writes are small; no network calls.

## 7. Interfaces / Contracts (CLI/config/API/files/events) — list concrete changes

Installer metadata file (Linux only):
- File: `<prefix>/install_state.json` (default `~/.substrate/install_state.json`)
- Existing: `schema_version=1`, `host_state.group`, `host_state.linger`, timestamps
- Add fields (names tentative; must be stable if adopted):
  - `host_state.platform.os_release.id` (string; from `/etc/os-release` `ID`)
  - `host_state.platform.os_release.id_like` (string; from `ID_LIKE`, raw)
  - `host_state.platform.pkg_manager.selected` (string; e.g. `pacman`, `apt-get`)
  - `host_state.platform.pkg_manager.source` (string enum; e.g. `flag|env|os_release|path_probe`)

Write semantics:
- Ensure `install_state.json` is written at least once per successful install, even if there were no group/linger events.
- Existing “event” records remain; new platform fields update idempotently on subsequent runs.

Read semantics (future consumers; this ADR may only define the contract):
- Prefer `install_state.json` metadata for user-facing guidance strings, but always fall back to runtime detection when missing.

## 8. Options (at least 2)

### Option 1 — Extend `install_state.json` (schema_version=1) with platform metadata (recommended)

**Description (1 paragraph)**
Add optional platform metadata keys under `host_state.platform.*` while keeping `schema_version=1`. Update installer
write logic so the file is created/updated even when no group/linger changes occurred. Uninstallers keep reading only
the existing keys and ignore the new ones.

**Pros**
- Reuses the canonical metadata file already documented and shipped.
- Backwards-compatible (no schema bump).
- Single source of truth for “what did install detect?”.

**Cons**
- Requires modifying the “only write when events exist” behavior, which is currently an intentional optimization.

**Risk notes**
- Must be careful not to accidentally record metadata on non-Linux platforms (keep Linux-only contract).

### Option 2 — Write a new `host_platform.json` file under `$SUBSTRATE_HOME` (separate from install_state)

**Description (1 paragraph)**
Create a dedicated file for distro/pkg-manager detection output and keep `install_state.json` strictly for
group/linger/uninstall cleanup.

**Pros**
- Avoids touching the uninstall-coupled file.

**Cons**
- Adds another state file to document, migrate, and keep in sync.
- Consumers must know which file to read.

**Risk notes**
- State fragmentation increases support burden.

### Option 3 — Store detection in `config.yaml` (avoid)

**Description (1 paragraph)**
Persist distro/pkg-manager detection into `$SUBSTRATE_HOME/config.yaml` as a normal config key.

**Pros**
- Easy to read from the Rust CLI.

**Cons**
- Conflates “detected host facts” with “user intent configuration”.
- Harder to reason about “should the user edit this?”.

**Risk notes**
- Misconfiguration risk; would need strong validation.

## 9. Recommendation (tentative) + “Choose Option X when…”

Tentative: **Option 1**.

Choose Option 1 when we want one canonical, documented metadata store and can keep it additive.
Choose Option 2 only if we decide `install_state.json` must remain “event-only”.

## 10. Slice Decomposition (required)

- ADR Candidate A (this one): persist platform detection in `install_state.json`.
  - Slice 1: Parse `/etc/os-release` (best-effort) and record `id`/`id_like` + selected pkg manager + source.
  - Slice 2: Ensure install_state is written even when no group/linger events occurred (idempotent, safe).
  - Slice 3: Add installer smoke assertions (update `tests/installers/install_state_smoke.sh`) to verify new keys exist on Linux.
- Candidate B (follow-up): use persisted metadata to improve “manual install guidance” messages in relevant surfaces (installer retries, doctor output, world-deps blockers).
- Candidate C (follow-up): if/when we add non-apt world provisioning, use the metadata contract (or a new “world OS detection” contract) to select a system package manager safely.

## 11. Acceptance Criteria Draft (<= 8 items, observable outcomes)

- After a Linux install, `<prefix>/install_state.json` exists even if no group/linger changes occurred.
- The JSON includes `host_state.platform.os_release.id` and `id_like` when `/etc/os-release` is present.
- The JSON includes `host_state.platform.pkg_manager.selected` and a `source` value.
- If `/etc/os-release` is missing/unreadable, install proceeds and still records `pkg_manager.selected` with `source=path_probe` (or similar).
- Existing uninstall flows continue to work unchanged (schema_version remains `1`).

## 12. Open Questions / Unknowns (with priority)

- P0: Should installs with `--no-world` also write `install_state.json` platform metadata (proposal: yes; it’s host metadata, not world provisioning)?
- P0: Exact field naming + nesting under `host_state` (prefer `host_state.platform.*` as above).
- P1: Should we record `VERSION_TAG` / installer version in `install_state.json` for debugging, or keep that elsewhere?

## 13. “Ready to Draft ADR?” checklist (yes/no with reasons)

- [ ] Option is locked (Option 1 vs 2).
- [ ] Field names are locked (stable JSON keys).
- [ ] Decision on `--no-world` write semantics is locked.
- [ ] Acceptance criteria match desired support/debug UX.
