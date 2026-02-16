# Session log — WDH0-integ

- Date (UTC): 2026-02-16
- Worktree: `/home/spenser/__Active_code/substrate/wt/world-deps-host-visible-hardening-wdh0-integ`
- Spec: `docs/project_management/next/world-deps-host-visible-hardening/WDH0-spec.md`

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
