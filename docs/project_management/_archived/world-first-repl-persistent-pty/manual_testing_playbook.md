# Manual Testing Playbook — World-First REPL With Persistent World PTY

This playbook contains runnable commands and expected results.

Exit code taxonomy: `docs/project_management/standards/EXIT_CODE_TAXONOMY.md` (no overrides declared for this feature).

## Behavioral smoke scripts (authoritative for behavior platforms)

Behavior platforms required: `linux`, `macos`.

- Linux: `bash smoke/linux-smoke.sh` (expected exit: `0`)
- macOS: `bash smoke/macos-smoke.sh` (expected exit: `0`)
- Windows (CI parity-only; no behavioral assertions for this feature): `pwsh -NoProfile -File smoke/windows-smoke.ps1` (expected exit: `0`)

Slice-scoped smoke:
- Use `SUBSTRATE_SMOKE_SLICE_ID` to run only the expected checks for a slice (`C0`, `C1`, `C2`, `C3`, `C4`, `C5`).
- Example: `SUBSTRATE_SMOKE_SLICE_ID=C3 bash smoke/linux-smoke.sh`

## CI parity (compile/test)

CI parity platforms: `linux,macos,windows`

Required gates (integration tasks dispatch these):
- `make ci-compile-parity CI_WORKFLOW_REF="feat/world-first-repl-persistent-pty" CI_REMOTE=origin CI_CLEANUP=1`
- `scripts/ci/dispatch_ci_testing.sh --workflow-ref "feat/world-first-repl-persistent-pty" --remote origin --cleanup`

### CI audit + evidence ledger (recommended first)

Goal:
- Reduce redundant CI dispatch by checking whether the last green run already covered the required OS set, and whether changes since then are docs/planning-only.

Ledger (per slice; gitignored):
- `docs/project_management/_archived/world-first-repl-persistent-pty/logs/<slice>/ci-audit/ledger.jsonl`

Before dispatching **CI parity** (ci-testing / compile-parity / CI Testing):
- `scripts/ci-audit/ci_audit.sh --kind ci-testing --orch-branch "feat/world-first-repl-persistent-pty" --ledger-path "docs/project_management/_archived/world-first-repl-persistent-pty/logs/<slice>/ci-audit/ledger.jsonl"`

After a dispatch completes (record run id + tested sha from dispatcher stdout):
- `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/_archived/world-first-repl-persistent-pty/logs/<slice>/ci-audit/ledger.jsonl" --kind ci-testing --mode compile-parity --orch-branch "feat/world-first-repl-persistent-pty" --run-id "<RUN_ID>" --tested-sha "<HEAD>"`

Docs/planning-only changes:
- Per policy, ci-audit recommends **SKIP all CI** when the diff is entirely under `docs/`.

## Linux manual validation (local)

Preconditions:
- World backend is provisioned and reachable:
  - `substrate world doctor --json` exits `0` and `.ok==true`.

### 1) Interactive REPL world-first semantics (world enabled)
Run:
- `substrate`

In the REPL, run:
1) Create a world-only directory:
   - `mkdir -p .wf_repl_dir`
2) Verify the directory is not present on the host filesystem (in a separate host terminal):
   - `test ! -d .wf_repl_dir && echo OK: host does not see world-only dir`
3) Verify in-world cwd persistence:
   - `cd .wf_repl_dir`
   - `pwd` (expected: ends with `/.wf_repl_dir` and reflects physical cwd semantics)
4) Verify exported env persistence:
   - `export WF_FOO=bar`
   - `echo "$WF_FOO"` (expected: `bar`)
   - `unset WF_FOO`
   - `echo "$WF_FOO"` (expected: empty line)

Expected:
- The REPL prints normal command output and continues prompting.
- `cd` succeeds even though the directory does not exist on the host filesystem (world-first semantics).

### 2) `:pty` shares persistent session state (world enabled)
In the same REPL session:
- `cd /tmp`
- `:pty pwd`

Expected:
- Output is `/tmp` (the PTY passthrough command runs inside the persistent session).

### 3) `:host` gating (disabled by default)
In the REPL started without `--repl-host-escape`:
- `:host pwd`

Expected:
- The REPL rejects the directive and does not execute on host or world.

Now exit and re-run with host escape enabled:
- `substrate --repl-host-escape`

In the REPL:
- `:host pwd` (expected: host pwd output)
- `pwd` (expected: still world-first; `:host` does not change world session state)

### 4) Protocol failure mode is fail-closed (no host fallback)
Precondition:
- Stop the world backend (e.g., stop world-agent socket/service).

Run:
- `substrate` (with world enabled)

Expected:
- Startup fails with a high-signal error.
- Exit code: `3` (dependency unavailable) per taxonomy.

### 5) Out-of-band Session PTY output while idle (unattributed; does not corrupt Reedline)
Purpose:
- Validate the v1 contract that PTY bytes may arrive while idle and MUST be rendered without breaking the line editor buffer.
- This is not “supported job control”; it is a robustness test for the output/rendering path.

In a world-enabled REPL session, run:
- Start a detached writer that will print to the controlling TTY after the submission completes:
  - `bash -lc 'nohup sh -c "sleep 1; echo OOB_FROM_WORLD_PTY" >/dev/tty 2>/dev/tty </dev/null &'`

Expected:
- After ~1s, `OOB_FROM_WORLD_PTY` appears on the terminal even though the REPL is idle.
- The current input buffer is preserved (your partially typed input is not destroyed/corrupted).
- No attempt is made to attribute this output to a specific `cmd_id` in v1.

### 6) Structured host output buffering during PTY passthrough (no injection into PTY bytes)
Purpose:
- Validate the locked routing invariant that Substrate-managed structured output MUST NOT be injected into the Session PTY byte stream.
- During PTY passthrough, structured output SHOULD be buffered and flushed after the foreground PTY command completes.

In a world-enabled REPL session, run:
1) Start a structured-output producer:
   - `:demo-agent`
2) Immediately start a PTY passthrough command that runs long enough to overlap:
   - `:pty bash -lc 'sleep 3'`

Expected:
- While the `:pty` command is running, you do NOT see `:demo-agent` structured output interleaved into the PTY output.
- After the `:pty` command completes, buffered `:demo-agent` output is rendered (and the REPL remains usable).

## macOS manual validation (local)

Preconditions:
- Lima is provisioned and world backend is reachable:
  - `substrate world doctor --json` exits `0` and `.ok==true`.

Run:
- `bash smoke/macos-smoke.sh`

Expected:
- Exit `0`.
- Output contains `OK:` lines for each validated slice.

## Windows manual validation (local)

Run:
- `pwsh -NoProfile -File smoke/windows-smoke.ps1`

Expected:
- Exit `0`.
- Output contains `OK: Windows smoke is a no-op for this feature`.

## CI behavioral smoke dispatch (required for Linux + macOS behavior validation)

Run from the integration worktree `HEAD`:
- `make feature-smoke FEATURE_DIR="docs/project_management/_archived/world-first-repl-persistent-pty" PLATFORM=behavior SMOKE_SLICE_ID="<slice>" RUNNER_KIND=self-hosted WORKFLOW_REF="feat/world-first-repl-persistent-pty" REMOTE=origin CLEANUP=1 RUN_INTEG_CHECKS=1`

Recommended before dispatch:
- `scripts/ci-audit/ci_audit.sh --kind feature-smoke --orch-branch "feat/world-first-repl-persistent-pty" --feature-dir "docs/project_management/_archived/world-first-repl-persistent-pty" --ledger-path "docs/project_management/_archived/world-first-repl-persistent-pty/logs/<slice>/ci-audit/ledger.jsonl"`

After a dispatch completes (record run id + tested sha from dispatcher stdout):
- `scripts/ci-audit/ci_audit_record.sh --ledger-path "docs/project_management/_archived/world-first-repl-persistent-pty/logs/<slice>/ci-audit/ledger.jsonl" --kind feature-smoke --mode behavior --orch-branch "feat/world-first-repl-persistent-pty" --run-id "<RUN_ID>" --tested-sha "<HEAD>" --feature-dir "docs/project_management/_archived/world-first-repl-persistent-pty"`

Expected:
- Dispatcher prints `DISPATCH_OK=1` and a `RUN_URL=...`.
- Workflow concludes success for Linux and macOS.

Slice values:
- `SMOKE_SLICE_ID` is one of: `C0`, `C1`, `C2`, `C3`, `C4`, `C5`.
