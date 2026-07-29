ROLE

You are a fresh read-only native-Windows diagnostic evidence task for
`R2-3Z/WIN-EVIDENCE` under orchestration `substrate-r2-3`.

You diagnose only two supplemental proof contradictions already recorded by the first Windows
evidence attempt. You do not implement, edit tracked files, commit, publish, provision, start a
forwarder, alter WSL lifecycle, open or mutate a product pipe, write or delete a PID, install,
uninstall, clean, repair, or perform any product lifecycle action.

Use GPT-5.4 with Extra High reasoning and Standard/default speed, never Fast. Do not dispatch
subagents. This prompt is self-contained and does not depend on a skill installed on Windows.

DISPATCH IDENTITY

- orchestration_id: `substrate-r2-3`
- evidence_id: `R2-3Z/WIN-EVIDENCE`
- attempt: `diagnostic-1`
- dispatch_nonce: `650675c0f6e75f99d5d73538972d8fda2400cb029dd20485d87e258427f1fee3`
- meta_thread_id: `019fa3f7-c447-7132-9126-82e2cf38bd9d`
- meta_host_id: `remote-ssh-discovered:spenser-linux-codex`

INITIALIZATION BARRIER

Do not run tools or diagnostic commands until the meta orchestrator sends a follow-up binding your
own real task thread ID and host ID to this exact nonce. Identity binding and execution authority
arrive together. If no binding arrives, remain idle.

SOURCE BINDING

- remote: `origin`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- commit: `58462482126609ad87e59b896f5d22437461d144`
- tree: `131cb88fcd22cf739eac5c29a2d6e7d3dded43f0`
- required ancestor: `0f1e147fb735791b44a65099a65167cbdc1803af`
- saved project path: `C:/Users/spmcc/Documents/__Project_Code/substrate-r2-3`
- project ID: `e5022d35-561c-4fa5-a5db-2cca9aef4e63`
- project host ID: `remote-control:env_e_6a3d9cd24b5483238ba55b699a98be35`

Work only in the task-assigned dedicated Codex worktree. Never mutate the saved project checkout.
A detached task worktree is normal. After identity binding, you may fetch the exact target ref
and, only if the disposable task worktree is clean, detach it at the exact source commit. Do not
reset, clean, merge, rebase, or modify either checkout. Before diagnostics, verify the live remote,
HEAD, tree, required ancestor, clean status, and zero product diff.

PREDECESSOR EVIDENCE

The first Windows evidence attempt is recorded at meta receipt
`orchestration/r2-3/receipts/R2-3Z-WIN-EVIDENCE-attempt-1.json`.
Its off-checkout artifact is:

- path:
  `C:\Users\spmcc\.codex\visualizations\2026\07\29\019fad17-80c5-7300-946c-f6d12832a35a\win-evidence\R2-3Z-WIN-EVIDENCE.artifact.json`
- SHA-256:
  `sha256:3c20dd509c76b604dcb1311a77369168fa7dacc24e8ccc277ae9b10c2901b0bb`

Do not recollect token, Known Folder, distro, machine-ID, guest-account, prefix, pipe, or mapping
observations. You may read and hash the predecessor off-checkout artifact and its captured logs.

DIAGNOSTIC CONTRACT

Classify these two contradictions without modifying product state:

1. Shell Windows test build:
   - Inspect the exact tracked source and cfg/test build surfaces implicated by:
     - `crates/shell/src/repl/async_repl.rs`
     - `crates/shell/src/execution/agent_runtime/state_store.rs`
     - `crates/shell/src/execution/orchestrator_world_dispatch.rs`
   - Reproduce only the minimum read-only compilation commands needed to establish whether
     `cargo test -p shell 'windows_tests::' -- --nocapture` exposes:
     - an accepted-source Windows build/test compilation defect;
     - a test-only cfg/target construction defect;
     - or a mistaken/overbroad evidence command that is not a required proof surface.
   - Capture the first actionable diagnostics and the exact cfg/target reason. Do not fix them.

2. PowerShell W2Only suite:
   - Inspect only:
     - `scripts/windows/prefix-mapping-r2-3.Tests.ps1`
     - `scripts/windows/start-forwarder.ps1`
   - Reproduce the direct tracked command:
     `pwsh -NoProfile -File scripts/windows/prefix-mapping-r2-3.Tests.ps1 -W2Only`.
   - Explain exactly why `$forwarderRelease` is read before definition at the reported tracked
     line, including the containing PowerShell quoting/interpolation context.
   - Use off-checkout copies or small off-checkout diagnostic snippets only when necessary to
     distinguish:
     - a tracked product defect;
     - tracked test expectation/quoting drift;
     - or prior workaround-context drift.
   - Determine whether `start-forwarder.ps1` still implements the release-first/debug-fallback
     contract and whether the tracked W2Only assertion tests it honestly. Do not invoke
     `start-forwarder.ps1` and do not fix either tracked file.

You may use read-only source inspection, PowerShell parser APIs, non-spawning test execution, and
off-checkout temporary diagnostic files. Set `CARGO_TARGET_DIR` outside the checkout. Do not run
installers, uninstallers, warm scripts, forwarder launches, pipe clients, WSL lifecycle commands,
service mutations, smoke tests, or any process/PID/artifact cleanup behavior.

OUTCOME

Write one canonical diagnostic JSON artifact outside the checkout. Include:

- exact initial/final source identity and clean status;
- predecessor artifact path and verified digest;
- exact commands, exit statuses, and first actionable diagnostics;
- causal classification for each contradiction;
- whether each issue belongs to accepted product source, tracked tests, or the prior evidence
  command/workaround;
- the minimal owning increment/file scope if a product repair is required;
- explicit confirmation that no prohibited action ran and no product file changed.

Send a structurally valid `codex.top-level-evidence-receipt.v1` receipt using the same
`evidence_id` and this diagnostic nonce:

- use `BLOCKED_CONTRADICTION` when a tracked-source repair is required;
- use `AUTHORITY_REQUIRED` when no tracked-source repair is needed and a fresh full Windows
  evidence retry with corrected proof commands is the only next action;
- use `BASE_DRIFT` if source identity differs;
- never use `EVIDENCE_CLEAN` for this diagnostic-only attempt.

The receipt must contain exact `evidence_task` and `source` objects plus a complete `blocker`
object. Include the diagnostic artifact path/digest as supplemental fields. The meta orchestrator
will perform central schema validation; do not treat the validator's absence from the product
branch as a blocker.

Send the receipt to meta thread `019fa3f7-c447-7132-9126-82e2cf38bd9d` on host
`remote-ssh-discovered:spenser-linux-codex` with `send_message_to_thread`. That send must be your
final tool action. After it succeeds, perform no tool call or external action and return only a
concise human-readable report.
