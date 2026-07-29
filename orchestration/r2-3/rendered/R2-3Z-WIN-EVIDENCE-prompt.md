ROLE

You are a fresh read-only top-level evidence task for `R2-3Z/WIN-EVIDENCE` on native Windows.
You collect only the declared native mapping evidence. You do not implement, edit tracked files,
commit, publish, provision, start a forwarder, stop or unregister WSL, kill a timeout process,
write or delete a PID, mutate a pipe/service, install/uninstall, clean, or perform any teardown,
retry, replacement, convergence, or deletion lifecycle action.

This task and any Codex work in it use GPT-5.4 with Extra High reasoning and Standard/default
speed, never Fast. Do not dispatch subagents; the evidence collection is bounded and read-only.
This prompt is self-contained and does not depend on a skill installed on this platform.

DISPATCH IDENTITY

- orchestration_id: `substrate-r2-3`
- dispatch_nonce: `2234989708fd55aa0b795ad5f64ab760ed79fd6450d11c630ce9ca9053e235b1`
- meta_thread_id: `019fa3f7-c447-7132-9126-82e2cf38bd9d`
- meta_host_id: `remote-ssh-discovered:spenser-linux-codex`
- evidence_id: `R2-3Z/WIN-EVIDENCE`

EVIDENCE-TASK IDENTITY BARRIER

Do not run tools or evidence commands until the meta orchestrator sends a follow-up binding your
own real evidence-task thread ID and host ID to this exact nonce. Echo those exact IDs in the
receipt. Identity binding and execution authority arrive together.

SOURCE BINDING

- remote: `origin`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- commit: `58462482126609ad87e59b896f5d22437461d144`
- tree: `131cb88fcd22cf739eac5c29a2d6e7d3dded43f0`
- saved project path: `C:/Users/spmcc/Documents/__Project_Code/substrate-r2-3`
- project ID: `e5022d35-561c-4fa5-a5db-2cca9aef4e63`
- project host ID: `remote-control:env_e_6a3d9cd24b5483238ba55b699a98be35`

Work only in the task-assigned dedicated Codex worktree. Never mutate
`C:\Users\spmcc\Documents\__Project_Code\substrate-r2-3`. The assigned worktree path is expected
to differ from the saved project path and must be reported separately in the evidence artifact.
A detached task worktree is normal. After identity binding, you may fetch the exact target ref
and, only if the assigned worktree is clean, detach that disposable worktree at the exact source
commit. Do not reset, clean, merge, rebase, or modify the saved checkout. Before collecting
evidence, verify the live remote, HEAD, tree, required ancestor
`0f1e147fb735791b44a65099a65167cbdc1803af`, clean status, and zero product diff.

NATIVE PREREQUISITES

- The host must be native Windows with supported PowerShell 7.
- `pwsh`, `wsl.exe`, `git`, `cargo`, and `rustc` must be available.
- A pre-existing registered WSL2 distribution must already be `Running`.
- Use the explicitly declared registered distribution name. Default to `substrate-wsl` only when
  that is the exact registered running name.
- If no suitable distribution is already running, stop with
  `BLOCKED_PLATFORM_HANDOFF_REQUIRED`. Never install, import, create, start, terminate, stop, or
  unregister one for this evidence task.

EVIDENCE CONTRACT

This is the native existing-WSL, mapping-only assignment for `R2-MAP-WIN-01` and its mapping
portion of `R2-DIAG-01`.

1. Record exact native environment identity and versions:
   - Windows edition/build/architecture and PowerShell version;
   - `wsl.exe --version`, `wsl.exe --status`, `git --version`, `cargo --version`, and
     `rustc --version`;
   - saved project ID/path/host ID plus the actual assigned worktree path.
2. Resolve the current Windows account and SID from the current process token and the
   LocalApplicationData Known Folder with `SHGetKnownFolderPath` for that token. Do not use
   inherited `USERPROFILE` or `LOCALAPPDATA` as authority.
3. Enumerate exact registered distro names with `wsl.exe -l -q`, confirm the selected exact name
   is already Running with `wsl.exe -l -v`, then make only read-only observations through that
   already-running distro: stable `/etc/machine-id` twice, `id -un`, `id -u`, and matching
   `getent passwd` name/UID rows yielding the guest account-database home.
4. Choose and record one explicit non-root selected Windows prefix `A` without creating it. Choose
   a distinct conflicting ambient root `B`, also without creating it. Select one canonical pipe
   (normally `\\.\pipe\substrate-agent`) and prove its exact normalization.
5. Create any helper or harness only outside the tracked checkout. Use a temporary Rust harness
   with the published `transport-api-types` crate, plus read-only PowerShell function extraction
   where useful, to construct/validate/encode/decode one IH and one WSL PM from the real native
   observations. Record the exact non-secret decoded fields:
   - A and conflicting B;
   - account+SID and token Known Folder;
   - IH commitment;
   - exact registered distro name and guest machine ID/account/UID/home;
   - normalized pipe;
   - canonical SID+distro+machine-ID+pipe scope digest and shared PID root;
   - A-scoped forwarder config and log paths;
   - guest `/run/substrate.sock`; and
   - the authenticated PM-derived WSL argv/environment projection.
   Do not place the raw carrier or raw encoded PM in general logs.
6. Under conflicting inherited `LOCALAPPDATA`, `USERPROFILE`, `SUBSTRATE_HOME`,
   `SUBSTRATE_ROOT`, `SUBSTRATE_FORWARDER_PIPE`, `SUBSTRATE_FORWARDER_TARGET*`, and `WSLENV`,
   run only non-spawning focused proof that the authenticated projection uses PM-derived distro,
   target, commitment, config/log roots, and complete WSL environment replacement or fails before
   spawn. Use the applicable published Rust tests, including the forwarder spawn-spec/internal
   preflight tests, and the PowerShell `scripts/windows/prefix-mapping-r2-3.Tests.ps1 -W2Only`
   suite as supplemental proof. Set `CARGO_TARGET_DIR` outside the checkout.
7. Do not invoke `scripts/windows/start-forwarder.ps1`, `wsl-warm.ps1`, installers,
   uninstallers, pipe clients, smoke tests, or any command that creates/opens/mutates the product
   pipe or starts a product child. Do not execute production code beyond read-only mapping,
   identity, registry, and guest account-database observation.
8. Prove no prohibited command ran. At minimum record that there was no WSL install/import/start,
   `--terminate`, `--shutdown`, `--unregister`, provisioning, forwarder launch, timeout kill,
   service mutation, PID write/delete, artifact cleanup, or installer/uninstaller action.
9. Finish with the same HEAD/tree, empty tracked and untracked status, no staged diff, no commit,
   no push, and the live remote still equal to the bound source commit.

Static/cross-target evidence may supplement but never replace the real Windows token/Known Folder
and already-running WSL observations. Do not claim provisioning success, product pipe transport,
forwarder activation, or native lifecycle evidence.

ARTIFACT AND RECEIPT

Write one canonical JSON evidence artifact outside the tracked checkout. It must include every
required observation, command with exit status, supplemental check result, A/B non-authority
proof, prohibited-action audit, initial/final repository identity, and explicit statement that
provisioning/forwarder/lifecycle work remains unclaimed. SHA-256 the exact artifact.

Create and validate a `codex.top-level-evidence-receipt.v1` receipt with:

- exact bound evidence task thread/host identity;
- `platform: "windows"`;
- exact source commit/tree/ref/live remote;
- environment host/project ID/saved project path/OS/tool versions;
- absolute artifact path and `sha256:<digest>`;
- sorted unique gates `["R2-DIAG-01", "R2-MAP-WIN-01"]`;
- `prohibited_actions_confirmed: true`; and
- `checkout_unchanged: true`.

Validate it with
`orchestration/r2-3/skill/orchestrate-top-level-tasks/scripts/validate_evidence_receipt.py`.
If any native prerequisite or proof is missing, send a blocked evidence receipt instead; never
substitute static evidence.

Send the receipt to meta thread `019fa3f7-c447-7132-9126-82e2cf38bd9d` on host
`remote-ssh-discovered:spenser-linux-codex` with `send_message_to_thread`. That send must be your
final tool action. After it succeeds, perform no tool call or external action and return only the
human-readable report.
