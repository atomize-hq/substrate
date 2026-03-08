# Decision Register — best-effort-distro-package-manager

Standard:

- `docs/project_management/system/standards/planning/PLANNING_RESEARCH_AND_ALIGNMENT_STANDARD.md` (Decision Register Standard)

Scope:

- This decision register supports `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`.
- Each decision is recorded as exactly two viable options (A/B) with explicit tradeoffs and a single selection.

---

### DR-0001 — `/etc/os-release` parser posture for production detection

**Decision owner(s):** Installer + security maintainers  
**Date:** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

**Problem / Context**

- ADR-0031 makes `/etc/os-release` parsing a production installer behavior. The parser must produce deterministic `ID` and `ID_LIKE` values without copying the test-only `source /etc/os-release` pattern from container smoke scripts.

**Option A — Source the file in shell and read `ID` / `ID_LIKE`**

- **Pros:**
  - Smallest implementation diff inside a shell script.
  - Mirrors common shell snippets used in ad hoc distro detection.
- **Cons:**
  - Executes file contents in the installer shell.
  - Conflicts with the ADR requirement for safe parsing.
- **Cascading implications:**
  - `contract.md` would need to permit shell execution from the os-release input path.
  - Hermetic tests would inherit a more dangerous production pattern.
- **Risks:**
  - Arbitrary shell execution from a malformed or hostile alternate os-release file.
  - Drift between production parsing and the safety posture promised to operators.
- **Unlocks:**
  - Faster initial implementation only.
- **Quick wins / low-hanging fruit:**
  - None that justify the safety regression.

**Option B — Parse only `ID` and `ID_LIKE` with line-oriented key extraction**

- **Pros:**
  - Satisfies the ADR safety posture: no shell execution.
  - Keeps the alternate os-release hook deterministic for hermetic tests.
  - Produces one explicit parser contract that downstream docs can inherit.
- **Cons:**
  - Requires explicit rules for comments, quotes, duplicates, and normalization.
- **Cascading implications:**
  - `contract.md` must define exact line parsing, quote stripping, duplicate-key behavior, and `<unknown>` semantics.
  - Validation must cover readable, unreadable, and invalid alternate-input cases.
- **Risks:**
  - Slightly more implementation work than sourcing the file.
- **Unlocks:**
  - Safe production detection and a stable downstream persistence contract.
- **Quick wins / low-hanging fruit:**
  - Reuse the same parser rules for `/etc/os-release` and `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.

**Recommendation**

- **Selected:** Option B — Parse only `ID` and `ID_LIKE` with line-oriented key extraction.
- **Rationale (crisp):** It removes code-execution risk while producing a deterministic parser contract that tests and downstream docs can reuse.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - os-release read + parsing contract
  - emitted `<unknown>` sentinel behavior
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - inherited `os_release.id` / `os_release.id_like` semantics

**Downstream docs constrained by the selection**

- `docs/INSTALLATION.md`
- `tests/installers/pkg_manager_detection_smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`

**Follow-up tasks (explicit)**

- Define the exact parser rules and `<unknown>` semantics in `contract.md`.
- Make `tests/installers/pkg_manager_detection_smoke.sh` assert the parser on unreadable and alternate-input branches.

---

### DR-0002 — Multi-manager `PATH` ambiguity posture and fixed fallback order

**Decision owner(s):** Installer maintainers  
**Date:** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

**Problem / Context**

- ADR-0031 requires deterministic fallback when os-release mapping does not select a manager. Mixed images and custom `PATH`s can expose more than one supported manager, so the installer needs one exact policy for selection and one exact ordered probe list.

**Option A — Warn and select the first detected manager in one fixed order**

- **Pros:**
  - Preserves best-effort installation behavior when several managers are available.
  - Gives operators a visible warning and a deterministic override path.
  - Aligns with the existing fallback order already encoded in `install-substrate.sh`.
- **Cons:**
  - A warning line becomes part of the contract and must remain stable.
- **Cascading implications:**
  - `contract.md` must define the exact warning template and the exact probe order.
  - Tests and docs must assert the same detected-manager list ordering.
- **Risks:**
  - The selected manager is not always the operator’s preferred manager; the explicit override remains the correction path.
- **Unlocks:**
  - Deterministic behavior without converting multi-manager hosts into hard failures.
- **Quick wins / low-hanging fruit:**
  - Lock the current order: `apt-get -> dnf -> yum -> pacman -> zypper`.

**Option B — Fail when more than one supported manager is present**

- **Pros:**
  - Removes ambiguity by refusing to choose among several managers.
  - Avoids the need for a warning-line contract.
- **Cons:**
  - Breaks best-effort installation on mixed images and dev containers.
  - Increases operator friction for environments that previously worked.
- **Cascading implications:**
  - `contract.md` would need a new failure branch and new remediation text for multi-manager `PATH` cases.
  - The repo smoke harness would need a hard-failure assertion instead of a warning assertion.
- **Risks:**
  - Higher support burden for benign environments that expose more than one package manager.
- **Unlocks:**
  - Stricter explicitness only.
- **Quick wins / low-hanging fruit:**
  - None compatible with the ADR’s best-effort posture.

**Recommendation**

- **Selected:** Option A — Warn and select the first detected manager in one fixed order.
- **Rationale (crisp):** It keeps the installer best-effort while making the fallback behavior deterministic, reviewable, and overrideable.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `PATH` probe contract
  - multi-manager warning line
  - decision-line source vocabulary
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - inherited `pkg_manager.selected` and `pkg_manager.source` values

**Downstream docs constrained by the selection**

- `docs/INSTALLATION.md`
- `docs/reference/env/contract.md`
- `tests/installers/pkg_manager_detection_smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

**Follow-up tasks (explicit)**

- Pin the warning template and fixed order in `contract.md`.
- Make the hermetic installer smoke assert both the selected manager and the warning line when more than one manager is present.

---

### DR-0003 — Alternate os-release input contract for hermetic tests

**Decision owner(s):** Installer + planning maintainers  
**Date:** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

**Problem / Context**

- The pack requires a hermetic repo test that can drive os-release inputs without mutating the host. The feature needs one exact rule for whether a production-visible hook exists or whether tests must depend on private harness mechanics.

**Option A — No production-visible hook; tests inject os-release content through harness-only plumbing**

- **Pros:**
  - Keeps the production env-var surface smaller.
  - Avoids documenting an installer-local test hook in operator docs.
- **Cons:**
  - Conflicts with downstream pack assumptions that already reference `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
  - Forces test-only plumbing that other docs cannot inherit.
- **Cascading implications:**
  - `persist-detected-linux-distro-pkg-manager` would need to unwind its existing dependency on the hook name.
  - The feature-local smoke wrapper and repo test would depend on internal-only wiring.
- **Risks:**
  - Drift between hidden harness behavior and published contract docs.
- **Unlocks:**
  - One less documented env var only.
- **Quick wins / low-hanging fruit:**
  - None that outweigh the downstream drift.

**Option B — Publish one Linux-only installer env var: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`**

- **Pros:**
  - Gives the pack and its downstream persistence pack one shared hook name.
  - Keeps hermetic tests deterministic without host mutation.
  - Produces one explicit absence and invalid-path contract.
- **Cons:**
  - Adds one documented installer-local env var.
- **Cascading implications:**
  - `contract.md` must define absolute-path validation, absence semantics, and the “no fallback to `/etc/os-release` when set” rule.
  - `docs/reference/env/contract.md`, the repo smoke harness, and the feature-local smoke wrapper must all use this exact hook name.
- **Risks:**
  - An accidental hook setting still needs deterministic invalid-path behavior.
- **Unlocks:**
  - One stable hook for hermetic tests and downstream pack alignment.
- **Quick wins / low-hanging fruit:**
  - Reuse the same hook in `tests/installers/pkg_manager_detection_smoke.sh` and the feature-local smoke wrapper.

**Recommendation**

- **Selected:** Option B — Publish one Linux-only installer env var: `SUBSTRATE_INSTALL_OS_RELEASE_PATH`.
- **Rationale (crisp):** It resolves existing downstream assumptions with one explicit contract instead of leaving hermetic input behavior hidden or unstable.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - `SUBSTRATE_INSTALL_OS_RELEASE_PATH` env-var contract
  - os-release read-path precedence and absence semantics
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
  - selected env-var ownership mapping
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`
  - inherited alternate-input contract

**Downstream docs constrained by the selection**

- `docs/reference/env/contract.md`
- `tests/installers/pkg_manager_detection_smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/persist-detected-linux-distro-pkg-manager/contract.md`

**Follow-up tasks (explicit)**

- Define the env-var contract in `contract.md` with exact path-validation and invalid-path semantics.
- Make `pre-planning/spec_manifest.md` pin the hook as selected instead of leaving it as an unresolved alternative.

---

### DR-0004 — Wrapper exit-status behavior for `scripts/substrate/install.sh`

**Decision owner(s):** Installer maintainers  
**Date:** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`

**Problem / Context**

- The quick-install path enters through `scripts/substrate/install.sh`, but the new package-manager contract introduces explicit exit codes `2`, `3`, and `4`. The wrapper needs one exact rule for whether it preserves those codes or collapses them.

**Option A — Keep wrapper failures collapsed to exit `1`**

- **Pros:**
  - No wrapper-code change required.
  - Preserves current generic failure behavior.
- **Cons:**
  - Hides the feature’s explicit exit-code taxonomy from the operator-facing entrypoint.
  - Makes the wrapper path contradict the direct installer contract.
- **Cascading implications:**
  - Docs and smoke coverage would need to scope exit-code guarantees to direct installer runs only.
  - Manual playbooks would need separate expectations for wrapper and direct paths.
- **Risks:**
  - Operators using the documented one-liner would not receive the advertised failure classes.
- **Unlocks:**
  - None beyond avoiding a small wrapper change.
- **Quick wins / low-hanging fruit:**
  - None compatible with a stable operator contract.

**Option B — Preserve direct-installer exit status for feature codes `0`, `2`, `3`, and `4`**

- **Pros:**
  - Makes the wrapper and direct installer share one operator-visible contract.
  - Keeps manual testing and smoke assertions aligned.
  - Preserves the selected error classes for the most common install entrypoint.
- **Cons:**
  - Requires a wrapper behavior change.
- **Cascading implications:**
  - `contract.md` must define wrapper pass-through as part of the public contract.
  - The smoke harness must assert both direct and wrapper paths.
- **Risks:**
  - None beyond the wrapper implementation change itself.
- **Unlocks:**
  - Stable one-liner behavior for explicit override and fallback failures.
- **Quick wins / low-hanging fruit:**
  - Reuse the same exit-code assertions for direct and wrapper invocations.

**Recommendation**

- **Selected:** Option B — Preserve direct-installer exit status for feature codes `0`, `2`, `3`, and `4`.
- **Rationale (crisp):** The documented install entrypoint must expose the same explicit failure classes as the underlying installer or the contract is not real for operators.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - wrapper interaction rules
  - exit-code table
- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
  - wrapper exit-status ownership mapping

**Downstream docs constrained by the selection**

- `docs/INSTALLATION.md`
- `tests/installers/pkg_manager_detection_smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`

**Follow-up tasks (explicit)**

- Pin wrapper pass-through in `contract.md`.
- Make slice and smoke specs assert wrapper parity for exit codes `0`, `2`, `3`, and `4`.

---

### DR-0005 — Validation topology for repo smoke and feature-local smoke

**Decision owner(s):** Planning + validation maintainers  
**Date:** 2026-03-08  
**Status:** Accepted  
**Related docs:** `docs/project_management/adrs/draft/ADR-0031-detecting-badger.md`, `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/impact_map.md`

**Problem / Context**

- The pack requires both a repo-level hermetic test path and a feature-local smoke artifact. The planning docs need one exact rule for whether the feature-local smoke script is authoritative on behavior or whether it is only a wrapper over the repo test.

**Option A — Keep `smoke/linux-smoke.sh` as a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`**

- **Pros:**
  - Produces one authoritative assertion set for the feature.
  - Keeps planning evidence capture without duplicating behavior logic.
  - Aligns with the impact map’s selected path.
- **Cons:**
  - The feature-local smoke script cannot diverge or add second-source behavior assertions.
- **Cascading implications:**
  - `BEDPM3` acceptance criteria must treat the repo test as authoritative.
  - The manual playbook must reference the wrapper as an execution convenience, not a second contract.
- **Risks:**
  - None if the wrapper remains thin.
- **Unlocks:**
  - Reusable validation for automation and human reruns with one assertion source.
- **Quick wins / low-hanging fruit:**
  - Call the repo test directly from the feature-local wrapper.

**Option B — Give `smoke/linux-smoke.sh` an independent assertion set**

- **Pros:**
  - Allows the feature-local smoke script to evolve separately from the repo test.
- **Cons:**
  - Creates two behavior authorities for the same feature.
  - Increases drift risk across planning docs, tests, and smoke output.
- **Cascading implications:**
  - `BEDPM3` would need to specify and maintain two assertion suites.
  - Manual and automated evidence can diverge without a single canonical test path.
- **Risks:**
  - Duplicate assertions drifting over time.
- **Unlocks:**
  - None required by this feature.
- **Quick wins / low-hanging fruit:**
  - None compatible with single-source validation.

**Recommendation**

- **Selected:** Option A — Keep `smoke/linux-smoke.sh` as a thin wrapper over `tests/installers/pkg_manager_detection_smoke.sh`.
- **Rationale (crisp):** One authoritative repo test plus one thin wrapper gives the pack runnable evidence without creating a second contract.

**Impacted contract surfaces**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/pre-planning/spec_manifest.md`
  - exact repo test path ownership mapping
  - feature-local smoke topology mapping
- `docs/project_management/packs/draft/best-effort-distro-package-manager/contract.md`
  - downstream no-drift reuse rule for smoke and docs

**Downstream docs constrained by the selection**

- `docs/project_management/packs/draft/best-effort-distro-package-manager/manual_testing_playbook.md`
- `docs/project_management/packs/draft/best-effort-distro-package-manager/smoke/linux-smoke.sh`
- `tests/installers/pkg_manager_detection_smoke.sh`

**Follow-up tasks (explicit)**

- Keep `tests/installers/pkg_manager_detection_smoke.sh` as the authoritative assertion suite.
- Make `smoke/linux-smoke.sh` invoke that repo test without adding a conflicting contract.
