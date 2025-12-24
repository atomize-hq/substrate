# World Deps Selection Layer — Manual Testing Playbook

This is the human-run validation checklist for the selection-driven world-deps model and ADR-0002 install-class + provisioning routing.

Authoritative specs:
- `docs/project_management/next/world_deps_selection_layer/S0-spec-selection-config-and-ux.md`
- `docs/project_management/next/world_deps_selection_layer/S1-spec-install-classes.md`
- `docs/project_management/next/world_deps_selection_layer/S2-spec-system-packages-provisioning.md`

Goal:
- Validate end-to-end behavior across Linux, macOS (Lima), and Windows (WSL) where technically possible.
- Where a journey is not supported, validate explicit, actionable failure messages and exit codes.

---

## 0) Preconditions (all platforms)

1) Verify the CLI:
```bash
substrate --version
which substrate
```

2) Capture world readiness:
```bash
substrate world doctor --json | jq .
```

3) Capture baseline health:
```bash
substrate health --json | jq .
```

4) Create a clean test workspace:
```bash
mkdir -p /tmp/substrate-wdl-smoke
cd /tmp/substrate-wdl-smoke
```

5) Initialize the workspace (C0):
```bash
substrate init
echo "exit=$?"
```

Expected:
- Exit `0`.

---

## 1) Selection gating (required)

### 1.1 Unconfigured selection is a no-op (all subcommands)

1) Ensure selection files are absent:
```bash
rm -f .substrate/world-deps.selection.yaml
rm -f ~/.substrate/world-deps.selection.yaml
```

2) Run:
```bash
substrate world deps status
echo "exit=$?"
substrate world deps sync
echo "exit=$?"
substrate world deps install nvm
echo "exit=$?"
substrate world deps provision
echo "exit=$?"
```

Expected:
- Each command prints one prominent “not configured (selection file missing)” line plus next steps.
- Each command exits `0`.
- No guest installs occur and no provisioning attempts occur.

### 1.2 Configure selection (workspace) and verify precedence output

1) Create a minimal workspace selection:
```bash
mkdir -p .substrate
cat > .substrate/world-deps.selection.yaml <<'YAML'
version: 1
selected:
  - nvm
  - pyenv
  - bun
YAML
```

2) Run:
```bash
substrate world deps status --json | jq '.selection'
```

Expected:
- `configured=true`
- `active_scope="workspace"`
- `active_path=".substrate/world-deps.selection.yaml"`

### 1.3 `--all` ignores selection (discovery + debugging)

1) Run:
```bash
substrate world deps status --all --json | jq '.selection.ignored_due_to_all'
```

Expected:
- `ignored_due_to_all=true`
- Tool scope expands to inventory (tools list includes entries not in `selected`).

---

## 2) Install class enforcement (required)

### 2.1 `user_space` installs succeed (prefix writable)

Precondition:
- Pick at least one selected tool that is `user_space` (expected: `bun`).

1) Run:
```bash
substrate world deps sync
echo "exit=$?"
substrate world deps status --json | jq '.tools[] | select(.name=="bun")'
```

Expected:
- Exit `0` if all in-scope tools are satisfied (per S1).
- `bun` transitions to guest `present`.
- No runtime OS package installation is attempted.

### 2.2 `system_packages` never installs at runtime (must route to provisioning)

Precondition:
- Pick at least one selected tool that is `system_packages` (expected: `pyenv`).

1) Run:
```bash
substrate world deps sync
echo "exit=$?"
substrate world deps status --json | jq '.tools[] | select(.name=="pyenv")'
```

Expected:
- `sync` does not run any OS package manager.
- `pyenv` remains blocked and output references:
  - `substrate world deps provision`
- Exit code reflects “unmet prerequisites” (`4`) if any selected `system_packages` tool is not satisfied.

### 2.3 `manual` tools are never installed

Precondition:
- Add a manual tool via the user manifest overlay and select it.

1) Create a manual tool entry in the user overlay (`~/.substrate/manager_hooks.local.yaml`):
```bash
cat > ~/.substrate/manager_hooks.local.yaml <<'YAML'
version: 2
managers:
  - name: wdl-manual-demo
    guest_detect:
      command: "command -v wdl-manual-demo >/dev/null 2>&1"
    guest_install:
      class: manual
      manual_instructions: |
        Create a shim inside the world-deps prefix:
          /var/lib/substrate/world-deps/bin/wdl-manual-demo
        Example (run inside the world):
          printf '#!/bin/sh\necho wdl-manual-demo\n' > /var/lib/substrate/world-deps/bin/wdl-manual-demo
          chmod +x /var/lib/substrate/world-deps/bin/wdl-manual-demo
YAML
```

2) Update the workspace selection to include the manual tool:
```bash
cat > .substrate/world-deps.selection.yaml <<'YAML'
version: 1
selected:
  - nvm
  - pyenv
  - bun
  - wdl-manual-demo
YAML
```

3) Run:
```bash
substrate world deps status --json | jq '.tools[] | select(.name=="wdl-manual-demo")'
```

Expected:
- `selected=true`
- `install_class="manual"`

Expected:
- If the manual tool is missing in the guest, `sync` prints manual instructions, does not install, and exits `4`.
- `install wdl-manual-demo` exits `4` and prints the manual instructions.

---

## 3) Provisioning system packages (`world deps provision`)

### 3.1 macOS (Lima) and Windows (WSL): provisioning succeeds

Precondition:
- The active selection includes at least one tool whose `install_class` is `system_packages`.

1) Run:
```bash
substrate world deps provision
echo "exit=$?"
```

Expected:
- Exit `0`.
- Output lists computed apt packages and confirms success.

2) Re-run (idempotency):
```bash
substrate world deps provision
echo "exit=$?"
```

Expected:
- Exit `0` again (repair/upgrade is “re-run provision”).

3) Concrete “becomes present” assertion (system_packages)

This is the required proof that `system_packages` tools become satisfied after provisioning:
- The tool’s `guest_detect.command` must succeed (per `decision_register.md` DR-0014), and `status` must show `guest.status="present"`.

Run:
```bash
substrate world deps status --json | jq -e '
  .tools[]
  | select(.selected==true)
  | select(.install_class=="system_packages")
  | select(.guest.status=="present")
' >/dev/null
echo "exit=$?"
```

Expected:
- Exit `0`.

4) Follow-up:
```bash
substrate world deps sync
echo "exit=$?"
```

Expected:
- Tools that were blocked on `system_packages` can now proceed (depending on their class/routing rules).

### 3.2 Linux host backend: provisioning is explicitly unsupported

1) Run:
```bash
substrate world deps provision
echo "exit=$?"
```

Expected:
- Exit `4`.
- Message: “unsupported on Linux host backend (would mutate host system packages)”.
- Output includes the required package list and manual install guidance.

---

## 4) Full-cage compatibility spot check (Linux only)

Preconditions:
- Linux host with I2/I3 implemented and enabled.
- A world backend is available (`substrate world doctor --json` reports the backend as available).

1) Request full cage via a per-workspace policy file (`.substrate-profile`):
```bash
cat > .substrate-profile <<'YAML'
world_fs:
  require_world: true
  mode: writable
  cage: full
  read_allowlist:
    - "**"
  write_allowlist:
    - "**"
YAML
```

2) Run:
```bash
substrate world deps sync
echo "exit=$?"
```

Expected:
- If the full cage is created successfully and `/var/lib/substrate/world-deps` is writable inside the cage, user-space installs succeed and exit code is `0`.
- If full cage cannot be created, `sync` exits non-zero and prints an actionable error.

---

## 5) Evidence to capture in PRs (copy/paste checklist)

- `substrate world doctor --json` (platform-specific proof of backend availability)
- `substrate world deps status --json | jq '.selection'`
- `substrate world deps status --json | jq '.tools'`
- Logs/outputs for:
  - unconfigured selection no-op (`status`, `sync`, `install`, `provision`)
  - `--all` ignoring selection
  - `system_packages` runtime block + provisioning route
  - provisioning success (Lima/WSL) or explicit unsupported error (Linux)
  - full-cage spot check (if applicable)
