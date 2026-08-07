R5 CONTINUATION 0030 — BIND INSTALL PROVENANCE, PRE-PM CAPSULE, AND CLOSED LIMA EFFECT

Continue in the same task/worktree:
- thread/host: 019fddab-1dd2-7570-9bd3-7f3841ce0283 / local
- worktree: /Users/spensermcconnell/.codex/worktrees/686b/substrate
- original dispatch nonce: e480367f5a8308d8ee73f97eb7734c80bd49e37a31c3a70a6e663668d622e70e
- prior continuation nonce: bb14577c654e5fbdb30bb981d4f0d712acd04c960219939f9b26b789c0f6420a
- continuation nonce: e8b183d657cdfebb9890749939737d814461ff1b801d796d95c0987d5b479b5d
- base/tree: 05d655fa1458a276f179d12057cbda772cc51eb6 / 845694f733fccbfcd3703957bf4976da0ed59b40
- tracked diff: sha256:26b115f6edf314cb8b2860a71854f8576fb7ca39efafff361d6322aea68fbef4
- combined subject: sha256:66934fd54096eb209e7e29e214ef93f983fe7edf68205c925e4513329f41757c
- status entries: 17; staged: 0
- review record: VALID discovery-1 bounded_stop at sha256:6eea8804d5c027ed5272f139afdef20ee752a32e86e132fdc117e5816738fb85

Reverify those bindings and preserve the subject. This inline prompt and nonce are complete authority; `authority-amendments/0030-r5-install-provenance-stage1-capsule-effect.json` is meta-only and is not expected in your worktree. No reset, clean, rebase, archive, donor bulk-copy, native action, publication, or R6 work.

The blocker is valid. R5 already requires the protected executor to prove absence, create/start the fixed Lima instance, observe/reobserve guest identity, finalize PM and durably publish the successor. Supply the missing producers and closed effect as follows; do not add a generic helper or caller selector.

## 1. Global install provenance contains install-time facts only

Replace/narrow partial `MacPublisherBootstrapProvenanceV1` as `MacPublisherInstallProvenanceV1`. The fixed root-owned record `/Library/Application Support/Substrate/lifecycle/bootstrap-provenance.v1.json` is canonical root:wheel 0444, no-follow and absent-or-exact. Remove per-scope manifest generation/digest.

The MAC branch of `dev-install-substrate.sh` creates it only after the prefix control binary, fixed privileged executor `/Library/PrivilegedHelperTools/com.substrate.lifecycle.publisher.v1`, existing LaunchDaemon plist and managed-copy records are durable. Record source commit/tree/ref, R5 review digest, IH/prefix, measured control/executor identities, plist digest, measured absolute `limactl` path/identity/digest/code identity/version, and embedded profile-template algorithm/version/digest. Prefix-local executor copies are non-authorizing. Install/retry/tamper tests remain in the existing installer fixture.

## 2. Direct bootstrap creates the per-scope pre-PM capsule

The hidden command accepts a new closed `MacPublisherBootstrapRequestV1` containing only the exact IH carrier. Reject every legacy manifest/scope/source/build/profile/action/Stage-1/PM/path field. After retained-terminal confirmation, the measured control reads install provenance, allocates secure UUIDv7 scope/attempt/nonces and bounded times, and constructs the complete generation-one pre-PM manifest internally.

Carry the full canonical pre-PM manifest in the retained FD3 authorization (or one exact nested capsule field); its digest must equal the authorization. The privileged executor creates/resumes the key and counter-zero anchor, then constructs and signs a one-use `LimaStageOneAuthorizationV1` with that same System-Keychain P-256 key. Persist `MacLimaStageOneCapsuleV1` in a fixed per-scope System-Keychain account, exact-joining install provenance, bootstrap authorization, complete pre-PM bytes/digest, initial anchor/counter, signed Stage-1 authorization/template/profile and revision. States are `Issued`, `Prepared`, `EffectStarted`, `InstanceObserved`, `Completed`, or `PreservingBlocked`. Return the signed Stage-1 authorization in the bounded direct-bootstrap response; its carrier bytes are public but replay authority comes only from signature plus the exact capsule/anchor state. Do not change `LifecyclePublisherProtectedStateV1`.

## 3. Embedded canonical profile and attempt marker

Use a versioned profile template embedded in `substrate-lifecycle-macos.rs`; no caller/profile path, `LIMA_PROFILE_PATH`, `PROJECT_PATH`, environment, repo lookup, or `scripts/mac/lima/substrate.yaml`. Render only signed attempt ID, capsule digest and fixed constants. The minimal profile may write one root-owned fixed guest Stage-1 marker containing attempt ID/capsule digest. It must not install packages, alter DNS, stage project files, create publisher/world components, or perform post-PM guest projection.

The signed authorization carries canonical rendered profile bytes (base64url) and SHA-256. Re-render and require byte equality. Install provenance binds template algorithm/version/digest.

## 4. One closed executor-owned Lima effect

Only the StageOne branch of `execute_mac_managed_action_v1` may:
1. open and remeasure the provenance-recorded absolute `limactl` executable no-follow;
2. write the re-rendered profile to a root-owned attempt-scoped file below the fixed lifecycle state root, fsync it and record its identity in the capsule;
3. clear inherited selectors; set exact IH/account-derived `HOME`/`LIMA_HOME`; build PATH only from the recorded tool parent plus fixed system dirs;
4. run only fixed commands: `limactl list <instance> --json`, `limactl start --tty=false --name <instance> <internal-profile>`, and `limactl shell <instance> -- <fixed literal observation command>`;
5. strictly parse and reobserve twice the instance/status, fixed attempt marker, machine ID, guest account, positive UID and account-database home.

The post-effect input is canonical `LimaStageOneObservationV1`, not machine ID alone. Derive PM from that exact tuple plus signed IH/template and run the specialized N→N+1 manifest/receipt/index/head/anchor/protected-state/admission completion from 0029. The wrapper supplies no profile/config/tool/path. Generic `lima-action`, raw helper relay and arbitrary shell remain forbidden.

Persist `EffectStarted` before spawn. If a retry sees absence, the same attempt may retry. If present, resume only when marker and every reobserved identity exact-join the capsule. Timeout, unknown state, partial/mismatched marker or identity preserves the instance/capsule and returns the closed blocked result; never delete/recreate/adopt.

## 5. Finish R5

Surgically revise the current partial code; retain valid provenance/FD3/admission/template/receipt work. Fix the ordinary wrapper/decoder closed-shape issues. The exact allowed fifteen implementation/test plus four review paths are unchanged. Do not edit `scripts/mac/lima/substrate.yaml`.

Add deterministic tests for installer publication, minimal bootstrap request, canonical pre-PM capsule, same-key Stage-1 signing, profile render/marker, fixed limactl image/env/argv/config, absence and pre-existing cases, double observation, PM derivation, every kill/retry boundary, and all earlier seven findings. Run MAC-native/x86_64-apple-darwin checks, installer fixture, formatting, path/symbol fence, MAC-only containment, secret scan and diff check. No Linux/Windows target commands and no native action.

`discovery-1` already exists. After remediation freeze the subject and run `closure-1` with a different fresh read-only gpt-5.6-terra Extra High reviewer. Only causally unmasked P1/P2 may use at most two supplemental cycles; record P3/P4 only; CLEAN stops. Publish exactly one fast-forward R5 commit only after terminal CLEAN, return `next_increment: AUX-R3-MAC-EVIDENCE-RECOVERY-R6`, and do not dispatch R6.
