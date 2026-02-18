# Session log — WDH0-integ

- Date (UTC): 2026-02-16
- Worktree: `/home/spenser/__Active_code/substrate/wt/world-deps-host-visible-hardening-wdh0-integ`
- Spec: `docs/project_management/_archived/next/world-deps-host-visible-hardening/WDH0-spec.md`

This file records smoke + targeted integration validation results for WDH0 without modifying planning docs.


## Run 2026-02-16 05:14:58 UTC

- Git HEAD: 6291ff0de28efb054b8f679dfefe26724285083e
- Git branch: world-deps-host-visible-hardening-wdh0-integ
- Platform: Linux spenser-linux 6.16.8-1-MANJARO #1 SMP PREEMPT_DYNAMIC Fri, 19 Sep 2025 16:09:36 +0000 x86_64 GNU/Linux
- Artifacts: artifacts/wdh0-integ-20260216-051458

### Platform smoke: scripts/check-container-prereqs.sh


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/check-container-prereqs.txt


### Build: cargo build -p substrate --bins


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/cargo-build-substrate.txt


### CLI smoke: target/debug/substrate --version


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/substrate-version.txt


### Platform smoke: substrate health --json


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/substrate-health.json


### CLI reference: substrate --help
- Output: artifacts/wdh0-integ-20260216-051458/substrate-help.txt


### CLI reference: substrate policy --help
- Output: artifacts/wdh0-integ-20260216-051458/substrate-policy-help.txt


### Tests: cargo test -p shell --tests


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/cargo-test-shell.txt


### Tests: cargo test -p world-agent --tests


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/cargo-test-world-agent.txt


### Tests: cargo test -p substrate-shim


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/cargo-test-substrate-shim.txt


### WDH0 acceptance smoke (host_visible=true policy, synthetic host env)

- Temp workspace: /tmp/substrate-wdh0-0eFfKw
- Effective policy (json): artifacts/wdh0-integ-20260216-051458/wdh0-policy-effective.json
- Non-PTY stdout: artifacts/wdh0-integ-20260216-051458/wdh0-env-nonpty.txt
- Non-PTY stderr: artifacts/wdh0-integ-20260216-051458/wdh0-env-nonpty.stderr
- PTY stdout: artifacts/wdh0-integ-20260216-051458/wdh0-env-pty.txt
- PTY stderr: artifacts/wdh0-integ-20260216-051458/wdh0-env-pty.stderr
- Checks: artifacts/wdh0-integ-20260216-051458/wdh0-env-checks.txt

- Note: Overwrote artifacts/wdh0-integ-20260216-051458/wdh0-env-checks.txt with corrected parsing (no rg -n line numbers).

#### WDH0 verdict (from artifacts/wdh0-integ-20260216-051458/wdh0-env-checks.txt)

- FAIL: `PATH` does not begin with `/var/lib/substrate/world-deps/bin:` (extra prefix segments present).
- FAIL: `PATH` contains host toolchain segments (e.g. `/.cargo/bin`, `/.local/bin`).
- FAIL: `HOME` is forwarded (expected `/root` by spec).
- FAIL: `XDG_CONFIG_HOME` is forwarded (expected `/root/.config` by spec).
- FAIL: `TERM` is forwarded (expected `xterm-256color` by spec).


### WDH0 config lever probe: world.env.inherit_from_host
- Temp workspace: /tmp/substrate-wdh0-0eFfKw
- Command: substrate config show --json --explain
- Result: exit_code=2
- Stdout: artifacts/wdh0-integ-20260216-051458/wdh0-config-show.json
- Stderr: artifacts/wdh0-integ-20260216-051458/wdh0-config-show.stderr
- Exit: artifacts/wdh0-integ-20260216-051458/wdh0-config-show.exit
  substrate: note: showing effective merged config; use --explain to view per-key sources
  invalid YAML in /tmp/substrate-wdh0-0eFfKw/substrate_home/config.yaml: unknown field `env`, expected one of `enabled`, `anchor_mode`, `anchor_path`, `caged`, `deps`

### Platform smoke: substrate world doctor --json


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/substrate-world-doctor.json


### Platform smoke: substrate shim doctor --json


- Result: exit_code=0
- Output: artifacts/wdh0-integ-20260216-051458/substrate-shim-doctor.json


## Run 2026-02-16 12:53:00 UTC (WDH0 merge + env determinism follow-up)

- Goal: Diagnose why WDH0 PATH/HOME/XDG/TERM contract failed in the earlier run and make behavior operational under the default world-agent execution mode.
- Summary:
  - The WDH0 request builder now sends a deterministic env map (validated by `cargo test -p shell --test world_env_path_sanitization_wdh0`).
  - The earlier smoke failure was caused by the **installed** world-agent’s default `always_isolate` execution path running the workload via a login shell (`sh -lc`), which sources system profile scripts and mutates `PATH` (e.g. prepending `/root/.local/bin` and appending host PATH segments).
  - As a proof of diagnosis: forcing `SUBSTRATE_WORLD_REQUEST_PROFILE=world-deps-provision` (which opts out of `always_isolate`) yields the expected baseline `PATH` inside `--world` runs.
  - Implemented fix (code changes):
    - World backend: run workloads under non-login shells for overlay/caged execution, clear inherited env, and apply the deterministic env contract defensively.
    - World-agent: honor `SUBSTRATE_WORLD_SOCKET` env var for direct-bind dev runs (useful for local testing; systemd still binds `/run/substrate.sock` by default).
- Status:
  - Full end-to-end validation of the fixed `always_isolate` path requires rebuilding/reprovisioning the system world-agent service (root/systemd) and rerunning `docs/project_management/_archived/next/world-deps-host-visible-hardening/smoke/linux-smoke.sh` with `SUBSTRATE_SMOKE_SLICE_ID=WDH0`.


## Run 2026-02-16 14:32:49 UTC (WDH1-integ kickoff)

- Git HEAD: 599c3ca7868fc8ea3802c3600506852420303623
- Git branch: world-deps-host-visible-hardening-wdh1-integ-core
- Platform: Linux spenser-linux 6.16.8-1-MANJARO #1 SMP PREEMPT_DYNAMIC Fri, 19 Sep 2025 16:09:36 +0000 x86_64 GNU/Linux
- Worktree: `/home/spenser/__Active_code/substrate/wt/world-deps-host-visible-hardening-wdh1-integ-core`
- Spec: `docs/project_management/_archived/next/world-deps-host-visible-hardening/WDH1-spec.md`
- Artifacts: artifacts/wdh1-integ-20260216-143249

### Platform smoke: scripts/check-container-prereqs.sh

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/check-container-prereqs.txt


### Build: cargo build -p substrate --bins

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/cargo-build-substrate.txt


### Tests: cargo test -p shell --tests

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/cargo-test-shell.txt


### CLI smoke: target/debug/substrate --version

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/substrate-version.txt


### Platform smoke: substrate health --json

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/substrate-health.json


### Platform smoke: substrate world doctor --json

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/substrate-world-doctor.json


### Platform smoke: substrate shim doctor --json

- Result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/substrate-shim-doctor.json


### WDH smoke: linux-smoke.sh (slice=WDH1)

- Default request profile result: exit_code=1
- Output: artifacts/wdh1-integ-20260216-143249/linux-smoke-wdh1.abs.txt
- Evidence (Case A PATH): artifacts/wdh1-integ-20260216-143249/linux-smoke-caseA-path.txt
- Workaround (`SUBSTRATE_WORLD_REQUEST_PROFILE=world-deps-provision`) result: exit_code=0
- Output: artifacts/wdh1-integ-20260216-143249/linux-smoke-wdh1.abs.world-deps-provision.txt


### WDH smoke: macos-smoke.sh (slice=WDH1)

- Result: exit_code=4 (not supported on this Linux host)
- Output: artifacts/wdh1-integ-20260216-143249/macos-smoke-wdh1.abs.txt


### WDH smoke: windows-smoke.ps1 (slice=WDH1)

- Result: exit_code=4 (`pwsh` not installed on this Linux host)
- Output: artifacts/wdh1-integ-20260216-143249/windows-smoke-wdh1.abs.txt


### WDH1 acceptance probe: apt npm wrappers

- Setup:
  - Policy set: world_fs.host_visible=true
  - Cleaned wrappers: `rm -f /var/lib/substrate/world-deps/bin/npm /var/lib/substrate/world-deps/bin/npx`
- Baseline: `substrate --world -c 'command -v npm >/dev/null'` exits 1
  - Output: artifacts/wdh1-integ-20260216-143249/wdh1-accept-npm-not-enabled.txt
- Attempted apply: `substrate world deps current sync` exits 4
  - Output: artifacts/wdh1-integ-20260216-143249/wdh1-accept-current-sync.txt
  - Reason: `apt-get` not found in this world (host is Manjaro/Arch), so `install.method=apt` cannot be applied end-to-end on this machine.
