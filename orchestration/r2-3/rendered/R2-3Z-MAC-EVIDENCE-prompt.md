ROLE

You are a fresh read-only top-level evidence task for `R2-3Z/MAC-EVIDENCE` on native macOS.
You collect only the declared native mapping evidence. You do not implement, edit tracked files,
commit, publish, provision, clean, start or stop a VM, start a forwarder, or perform any socket,
process, PID, guest-service, unit, teardown, retry, or deletion lifecycle action.

This task and any Codex work in it use GPT-5.4 with Extra High reasoning and Standard/default
speed, never Fast. Do not dispatch subagents; the evidence collection is bounded and read-only.
This prompt is self-contained and does not depend on a skill installed on this platform.

DISPATCH IDENTITY

- orchestration_id: `substrate-r2-3`
- dispatch_nonce: `eba960c4ad90f9ad4eb6d1ceea2c278f438d35526f534aa5bbd6ad7cddf4f918`
- meta_thread_id: `019fa3f7-c447-7132-9126-82e2cf38bd9d`
- meta_host_id: `remote-ssh-discovered:spenser-linux-codex`
- evidence_id: `R2-3Z/MAC-EVIDENCE`

EVIDENCE-TASK IDENTITY BARRIER

Do not run tools or evidence commands until the meta orchestrator sends a follow-up binding your
own real evidence-task thread ID and host ID to this exact nonce. Echo those exact IDs in the
receipt. Identity binding and execution authority arrive together.

SOURCE BINDING

- remote: `origin`
- target ref: `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- commit: `c583c5f293644fab75d8d42bd3bcad63f114d4fe`
- tree: `a070f5f5787c27180f13dece9a1c3c3241728fda`
- saved project path: `/Users/spensermcconnell/__Active_Code/atomize-hq/substrate`
- project ID: `local-b2016f8a311fef93149e42eaec4d704d`
- project host ID: `local`

Work only in the task-assigned dedicated Codex worktree. Never mutate the saved project checkout.
The assigned worktree path is expected to differ from the saved project path and must be reported
separately in the evidence artifact. A detached task worktree is normal. After identity binding,
you may fetch the exact target ref and, only if the assigned worktree is clean, detach that
disposable worktree at the exact source commit. Do not reset, clean, merge, rebase, or modify the
saved checkout. Before collecting evidence, verify the live remote, HEAD, tree, required ancestor
`0f1e147fb735791b44a65099a65167cbdc1803af`, clean status, and zero product diff.

NATIVE PREREQUISITES

- The host must be native Darwin/macOS with Virtualization.framework available.
- `limactl`, `jq`, `python3`, `bash`, `git`, `cargo`, and `rustc` must be available.
- A pre-existing declared Lima instance must already exist and already be `Running`.
- Use the explicitly declared instance name. Default to `substrate` only when that is the actual
  registered running instance; otherwise bind the exact existing declared name.
- If no suitable instance is already running, stop with `BLOCKED_PLATFORM_HANDOFF_REQUIRED`.
  Never create or start one for this evidence task.
- A previous read-only attempt observed the declared `substrate` instance as `Stopped`. The user
  reports that `limactl start substrate` is now running outside the evidence task. Verify the
  current registered state independently; this report is not evidence by itself.

EVIDENCE CONTRACT

This is the native pre-existing-Lima, no-forwarder mapping-only assignment for
`R2-MAP-MAC-01` and its mapping portion of `R2-DIAG-01`.

1. Record exact native environment identity and versions:
   - `sw_vers`, `uname -a`, Virtualization.framework availability;
   - `limactl --version`, `jq --version`, `python3 --version`, `bash --version`,
     `git --version`, `cargo --version`, and `rustc --version`;
   - saved project ID/path/host ID plus the actual assigned worktree path.
2. Resolve the current macOS account, UID, and account home from the host account database, not
   from inherited `HOME`. Prove the Lima control root is exactly `<account-database-home>/.lima`.
3. Choose and record one explicit non-root selected host prefix `A` without creating it. Choose a
   distinct conflicting ambient root `B`, also without creating it. Run the published
   `scripts/mac/lima-warm.sh --check-only --install-prefix A` against the already-running declared
   instance while inherited `HOME`, `LIMA_HOME`, `SUBSTRATE_HOME`, `SUBSTRATE_ROOT`,
   `SUBSTRATE_FORWARDER_PORT`, and `SUBSTRATE_WORLD_SOCKET` conflict toward B.
4. The native check-only run must:
   - resolve the host account and UID from the account database;
   - overwrite child `HOME` and `LIMA_HOME` with the account-database home/control root;
   - inspect only the exact declared running instance;
   - observe a stable guest machine ID twice;
   - round-trip guest account, UID, and home through the guest account database;
   - construct and validate one canonical `PlatformBootstrapMappingV1`;
   - bind the exact IH commitment;
   - project future host socket `A/sock/agent.sock` to guest `/run/substrate.sock`;
   - report ambient VSock/TCP/forwarder availability only as non-authority; and
   - report the explicit R3 forwarding-activation prerequisite.
5. Create any helper or harness only outside the tracked checkout. Use a temporary Rust harness
   with the published `transport-api-types` crate, or an equivalently exact read-only decoder, to
   independently construct/validate/encode/decode the IH and PM from the real native observations.
   Record the exact non-secret decoded fields: A, conflicting B, IH commitment, account-database
   home, `~/.lima` control root, VM name, guest machine ID, guest account/UID/home, realized guest
   substrate home, `A/sock/agent.sock`, `/run/substrate.sock`, and the explicit R3 prerequisite.
   Do not place the raw carrier or raw encoded PM in general logs.
6. From those same real observations, invoke the published
   `world-mac-lima` `mac_backend_smoke` example directly with explicit
   `--install-bootstrap-context-v1`, `--platform-bootstrap-mapping-v1`, and `--project-dir`
   arguments. It must validate typed construction and exit successfully before lifecycle,
   forwarding, readiness, endpoint, session, or exec behavior. Run it under conflicting ambient
   values and prove the explicit authenticated inputs win. Set `CARGO_TARGET_DIR` outside the
   checkout. Do not run the enclosing `scripts/mac/orchestration-smoke.sh`, because that script
   intentionally includes warm/product lifecycle legs outside this evidence assignment.
7. Run `bash tests/mac/prefix_mapping_r2_3.sh` only as supplemental regression proof. Static or
   fixture proof does not replace the native observations above. Set `CARGO_TARGET_DIR` outside
   the checkout for any Cargo command.
8. Prove no prohibited command ran. At minimum inspect the exact commands you invoked and record
   that no `limactl start`, `create`, `stop`, `delete`, `copy`, `factory-reset`, forwarder command,
   socket unlink/removal, guest unit/service mutation, or provisioning command occurred.
9. Finish with the same HEAD/tree, empty tracked and untracked status, no staged diff, no commit,
   no push, and the live remote still equal to the bound source commit.

Do not run `lima-warm.sh` without `--check-only`. Do not run installer, uninstaller, enable,
provisioning, smoke, forwarder, or cleanup commands. Read-only `limactl list --json` and
`limactl shell` observations are allowed only after confirming the declared instance is already
Running. Do not treat socket/process availability as authority or as product transport proof.

ARTIFACT AND RECEIPT

Write one canonical JSON evidence artifact outside the tracked checkout. It must include every
required observation, command with exit status, supplemental check result, A/B non-authority
proof, prohibited-action audit, initial/final repository identity, and explicit statement that
macOS product transport remains pending R3. SHA-256 the exact artifact.

Create and validate a `codex.top-level-evidence-receipt.v1` receipt with:

- exact bound evidence task thread/host identity;
- `platform: "macos"`;
- exact source commit/tree/ref/live remote;
- environment host/project ID/saved project path/OS/tool versions;
- absolute artifact path and `sha256:<digest>`;
- sorted unique gates `["R2-DIAG-01", "R2-MAP-MAC-01"]`;
- `prohibited_actions_confirmed: true`; and
- `checkout_unchanged: true`.

Recover the exact receipt validator and its `protocol_json.py` dependency read-only from meta
commit `21252983b077d48b4a4bc27971221b9092c91971` using `git show`, writing both only to a
temporary directory outside the checkout. Validate the receipt with that temporary validator.
Do not treat the meta ref as product source and do not write either validator into the checkout.
If any native prerequisite or proof is missing, send a blocked evidence receipt instead; never
substitute static evidence.

Send the receipt to meta thread `019fa3f7-c447-7132-9126-82e2cf38bd9d` on host
`remote-ssh-discovered:spenser-linux-codex` with `send_message_to_thread`. That send must be your
final tool action. After it succeeds, perform no tool call or external action and return only the
human-readable report.
