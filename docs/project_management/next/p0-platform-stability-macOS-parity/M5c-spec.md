# M5c Spec – First-run UX wiring (world deps)

## Goal
Ensure fresh macOS installs/provisioning produce a coherent “feels like host” experience by wiring world deps syncing into the right first-run paths, with aligned messaging across health/doctor/install flows.

This triad is about **workflow + UX coherence**, not reworking manifests (M4/M5a) or host detection semantics (M5b).

## Scope
### Required behavior
1. **Installer/provision integration**
   - `--sync-deps` (and/or equivalent first-run flow) invokes the correct world deps sync logic after provisioning completes successfully.
   - Output should be concise and clearly state what was synced vs skipped and why.
2. **Coherent recommendations**
   - `substrate health` and `substrate shim doctor` recommendations should point users to the correct follow-up (`world deps sync`, `shim repair`, etc.) without contradictions.
3. **Safe defaults**
   - First-run sync should not attempt guest installs for tools that are not host-present (per M5b semantics), unless explicitly requested (e.g., `--all`).

### Out of scope
- Expanding tool lists or recipes beyond what the manifests already define.
- Changing core world backend provisioning (covered by M1–M3).

## Acceptance criteria
- Fresh macOS install + provision + `--sync-deps` yields a world session where common host capabilities are available inside the guest with minimal manual steps.
- Health/doctor outputs and installer logs do not disagree about what is “missing” or what action the user should take.
- Sync behavior is explicit about skipped tools and includes actionable next steps.

