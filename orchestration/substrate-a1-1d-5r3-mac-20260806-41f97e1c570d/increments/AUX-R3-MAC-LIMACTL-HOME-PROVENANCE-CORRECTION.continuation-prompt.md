META CONTINUATION BINDING AND START AUTHORITY — AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION

Continue the existing clean source-correction task. Bind exactly:

- task/thread: `019fdfa4-ba96-70e0-b68d-1d0a4cf32a83` / `local`
- worktree: `/Users/spensermcconnell/.codex/worktrees/2f78/substrate`
- orchestration: `substrate-a1-1d-5r3-mac-20260806-41f97e1c570d`
- packet: `AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION`
- dispatch_nonce: `f2776d7aa9dd669dd9ff2a885ee50536ac9c01da991257d806850cb8be82a19b`
- expected base/tree: `1126b907df6e38043e9071da222f6c6a377b341c` / `3a29fa323a8b9099d894c11d98cc085ecf4807c5`
- target: `origin refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- contract: `increments/AUX-R3-MAC-LIMACTL-HOME-PROVENANCE-CORRECTION.contract.md`
- contract_sha256: `sha256:44201afa00d934254835f42af6fe0c378633d1bfbcbb1ec77fe1302e83073f77`
- amendment: `authority-amendments/0042-r3-mac-limactl-home-provenance-source-correction.json`
- amendment_sha256: `sha256:815ed0977d5affb77560fb63de0e5474e2f0f7e2db7aeeefd3ca34bb9d164fbe`
- skill suite: `25 skills / 54 files / sha256:e3de93f358594033ed59696a9624d2d0ac2fd0c2aa7ddf4a532810a05b262282`
- model/reasoning: `gpt-5.6-terra` / Extra High
- start_authorized: `true`

Read the complete contract and amendment before editing. Load `using-agent-skills` first and invoke
all named repository-local skills. This is a narrow MAC-only source correction, not an evidence run.

Fix the canonical installer's privileged fixed-limactl `--version` probe so its `env -i` environment
has an explicit root-controlled non-user HOME while retaining the exact absolute no-follow Lima
image, fixed privileged PATH, scrubbed environment, and fail-closed provenance joins. Prefer and
validate `/var/empty`; do not forward the caller's HOME. Add a deterministic base-RED/GREEN
regression that would have caught Lima 2.1.1's HOME panic and proves no ambient/user HOME leaks.

Stay inside the exact path/symbol fence. Do not run the real installer or mutate prefix, /Library,
Keychain, launchd, Lima, sudoers, known_hosts, or official evidence. Complete the full causal
cascading review with fresh independent reviewers and publish only after terminal CLEAN. Send the
single structured task receipt to meta as your final tool action. Do not dispatch evidence.
