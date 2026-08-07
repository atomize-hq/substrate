R3 MAC PAIRING-PROTOCOL CONTINUATION AUTHORITY

Resume the same top-level task and preserved worktree under a new authority/risk epoch:

- task: `019fd9fe-670c-7253-bf60-86f78cd373b5` on `local`
- worktree: `/Users/spensermcconnell/.codex/worktrees/60cc/substrate`
- continuation nonce: `e347bf1023f0193e5cfe204e86df65f0f9cb688cd554fb4d46f28479e49dc9a2`
- original dispatch nonce: `d7c49c63813094f35052883eb0a2964d28ce2df44d2c422664ea69c7c5f7072c`
- original base/tree: `d8a65fc8890dd37584aaeac2984c906188e5f06e` / `d8f25cc993f264f0c67fe3e00180eb393e41b2e3`
- expected live target remains the original base on `origin` `refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap`
- model/reasoning for the orchestrator and every subagent: `gpt-5.6-terra` / Extra High

Before any further mutation, verify the exact preserved subject: 13 tracked changed paths, no staged
paths, tracked diff SHA-256 `cd80c2d89b8e870fb302ccb7cb8fb0d7ed8684b418d4e32c9c44f9e77846603d`,
untracked Bash regression SHA-256 `353c3ef2d9d1f9ca1db663a401eb9133554c2368b9c9c59ce9033af71da79ba1`,
and untracked compile regression SHA-256 `0402cb717d1fa6b38e65e2d9fcd70e43feea2d296dded9576edb7fd5e07e149c`.
Do not clean, reset, discard, reconstruct, or archive this worktree.

The P1 is valid, and the authority it requested is now granted. The earlier source-only/non-R3
restriction is superseded exactly for the repository-defined R3 MAC pairing protocol. Implement the
complete safe protocol; do not replace the stub narrowly or fabricate authority. Preserve and
integrate the three already-passing corrections. This authority also permits uniquely nonce-scoped,
disposable native Keychain/XPC regression state with mandatory pre-state capture and exact cleanup;
it does not authorize the official evidence run or canonical host installation.

Run a new review epoch because authority and risk changed. Only publish when all four original
blockers and the pairing P1 are clean, native disposable state is restored, the full subject passes,
and live remote remains the original base. Publish exactly one commit and return the structured
source-correction receipt. Do not dispatch installation or evidence.

The complete binding amendment follows and is authoritative:

```json
{
  "protocol": "codex.orchestration-authority-amendment.v1",
  "orchestration_id": "substrate-a1-1d-5r3-mac-20260806-41f97e1c570d",
  "amendment_id": "0007-r3-mac-pairing-protocol-correction",
  "authorized_at_utc": "2026-08-07T02:31:46Z",
  "authority_source": "meta corridor adjudication under prior user authority to fix the verified macOS source blockers",
  "continuation_nonce": "e347bf1023f0193e5cfe204e86df65f0f9cb688cd554fb4d46f28479e49dc9a2",
  "task": {
    "thread_id": "019fd9fe-670c-7253-bf60-86f78cd373b5",
    "host_id": "local",
    "worktree": "/Users/spensermcconnell/.codex/worktrees/60cc/substrate"
  },
  "preserved_subject": {
    "base_commit": "d8a65fc8890dd37584aaeac2984c906188e5f06e",
    "base_tree": "d8f25cc993f264f0c67fe3e00180eb393e41b2e3",
    "tracked_diff_sha256": "sha256:cd80c2d89b8e870fb302ccb7cb8fb0d7ed8684b418d4e32c9c44f9e77846603d",
    "tracked_changed_path_count": 13,
    "untracked_paths": {
      "tests/installers/dev_install_bash32_fd_regression.sh": "sha256:353c3ef2d9d1f9ca1db663a401eb9133554c2368b9c9c59ce9033af71da79ba1",
      "tests/mac/dev_install_compile_surface_r3.sh": "sha256:0402cb717d1fa6b38e65e2d9fcd70e43feea2d296dded9576edb7fd5e07e149c"
    }
  },
  "supersedes": "The source-only/non-R3 restriction is superseded only as necessary to implement the repository-defined R3 MAC signed guest-pairing protocol and its native disposable regression proof. The four-blocker corrective objective, exact base, target ref, single-commit publication rule, and unrelated-architecture freezes remain unchanged.",
  "authorized_surfaces": [
    "common typed guest-ticket payload, signature, validation, and host-record contracts directly required by the macOS protocol, with backward-compatible Linux behavior and no Windows implementation",
    "macOS System Keychain non-exportable P-256 key creation/opening, key-bound protected state, SPKI export, canonical signing, and low-S normalization",
    "macOS XPC accepted-connection audit-token and designated-requirement binding for issue and consume operations",
    "trusted PM, guest-machine, staged-artifact, component, challenge/reservation, expiry, generation, and request bindings",
    "generation-CAS protected host pairing-record transitions for Reserved, TicketIssued, Consumed, retry, and unused revocation",
    "macOS issuer, consumer, client/control, guest bootstrap, and caller-supplied-ticket bypass removal",
    "directly necessary unit, integration, fixture, documentation, and review-control changes"
  ],
  "native_regression_authority": "The task may create uniquely nonce-scoped disposable macOS Keychain/XPC/test-service state solely for implementation regression proof, must record exact pre-state, must not reuse production publisher identities or official evidence artifacts, and must retire and prove exact post-test restoration before publication. This is implementation validation, not EVIDENCE:R3-MAC-IMP-01.",
  "boundaries": [
    "do not install Substrate onto the real canonical host prefix",
    "do not mutate Lima production state or run official native evidence",
    "do not change Linux behavior except backward-compatible shared-contract adaptation and regression coverage",
    "do not implement or alter Windows pairing behavior",
    "do not add alternate selectors, transports, principals, or authority bypasses",
    "stop if a required change genuinely escapes the R3 MAC pairing corridor"
  ],
  "review_epoch": "Authority and risk changed. Treat the blocked discovery record as mandatory design input, then run a fresh full-subject discovery review and a different fresh closure review. Remediate valid P1/P2 under the existing bounded causal rules.",
  "publication": "After all four original blockers and this P1 are clean, create exactly one commit from the original base and normal fast-forward push to the target ref.",
  "successor": "AUX-R3-MAC-HOST-PREP-INSTALL-RETRY",
  "successor_dispatch_authorized": false
}

```
