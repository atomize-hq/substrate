**Kind:** slice/task
**Stable ID:** `A1.1d-5R2-2-renewed-closeout-family`
**Canonical for:** A1.1d-5R2-2 renewed closeout phases and slice/task record
**Status:** canonical
**Authority scope:** exact extracted family-local source bodies only
**Source span:** [`03-phase-slice-map.md#a11d-5r2-2-b1-broad-wall-provenance-phase`](../03-phase-slice-map.md#a11d-5r2-2-b1-broad-wall-provenance-phase) lines 2120–2227
**Supersedes:** canonical ownership of the extracted source body; the source headings remain compatibility anchors
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md)

# A1.1d-5R2-2 renewed closeout phases and slice/task record

## A1.1d-5R2-2-B1 broad-wall provenance phase

This six-file documentation-only phase follows exact publication-authority commit
`928f94e7b4c498273b40385f7bffea9e4f949700` and precedes the renewed integration wall. Every live
broad shell wall, parallel/serial wall, canonical or differential baseline wall, final
change-detection wall, renewed closeout wall, and F/Harness wall in this phase map uses the
[canonical shell-library broad-wall invocation contract](../04-contracts-and-gates.md#canonical-shell-library-broad-wall-invocation-contract).

| B1 phase field | Frozen requirement |
|---|---|
| Defect | The renewed-closeout prompt omitted the private `TMPDIR`/`XDG_RUNTIME_DIR` provenance used by the canonical F wall. Rust `TempDir` roots fell beneath normal mode-`1777` `/tmp`, and trusted-root validation correctly rejected that ancestor. Serial and isolated controls failed identically; exact private-root parity restored the canonical wall. |
| Classification | `BaselineCommandMismatchConfirmed`; runtime source/tests unchanged; not environment drift, test isolation, or production regression. |
| Root/command authority | Create one fresh validated private `R` per wall; bind exact `R/tmp` and `R/xdg-runtime` before Cargo starts; preserve root/command/toolchain/result/hash provenance; clean only the revalidated disposable root after every child exits and evidence is retained. Ambient temporary-root selection is ineligible. |
| Canonical evidence | `1309/1264/45/0`; failure names `b23bb59ad12833d2c1d37c19c54933cd6bcb1c75e0dab8a70179b9881372be70`; normalized signatures `33c686a6ec9f3a0a4f51e1fca976445e6804da12fbbff50312a03f0042cdfac3`. |
| Ineligible evidence | `1309/1205/104/0`; failure names `7d6564bbcfaaed1fdb784eb4610683a02e4464b362215761d728be3e7361d8e5`; normalized signatures `ae67f87fbdc155575e13467506405afbd4568fc08b9058cb4159daa539549579`; 59 additions excluded from baseline authority. |
| Allowed files | Exactly the six canonical `llm-last-mile/runtime-refactor/00`–`05` Markdown files. |
| Forbidden work | Production, tests, fixtures, scripts, schemas, dependencies, generated files, trusted-root weakening, baseline replacement, renewed wall execution, source push, replay/rebase/rewrite/cherry-pick/merge/force push, integration-clean branch, R2-3/R2-4/R3. |
| Preservation | Commit directly on parent `928f94e7b4c498273b40385f7bffea9e4f949700`; preserve only through `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722`; leave the source remote at `2f6f1f69b3519dafff01ef543e7d260da2c37700`. |
| Historical B1 exit/next | Six-file validation and three fresh isolated read-only reviews CLEAN; local source 19 ahead and clean; then stop. The exact next task at that checkpoint was **Resume A1.1d-5R2-2 renewed production-fix-free integration closeout using provenance-validated private roots**; the remediation insertion below supersedes only that next-task disposition. |

The historical B1-only DAG was:

```text
publication-authority commit 928f94e7
  ↓
B1 broad-wall invocation docs correction
  ↓
renewed production-fix-free integration wall using validated private roots
  ↓
fresh independent reviews
  ↓
final six-file integration-closeout docs
  ↓
one ordinary fast-forward source push
```

R2-2 remained incomplete throughout B1. No integration wall ran in that phase. The current
controlling DAG is the remediation/publication map below.

## A1.1d-5R2-2 renewed closeout publication phases

The selected publication model is **Docs-on-top → one fast-forward publication**. Execute the
renewed closeout in exactly these phases:

| Phase | Allowed work | Completion evidence | Stop boundary |
|---|---|---|---|
| 1. Publication-authority correction | The six canonical control-pack files are committed as `928f94e7b4c498273b40385f7bffea9e4f949700` on exact runtime HEAD `e5fbd2d4441d248e137d52c44e493fb0abe158f8`; only dedicated preservation branch `feat/preserve-a1-1d-5r2-2-publication-authority-20260722` carries it remotely | Source remote still `2f6f1f69b3519dafff01ef543e7d260da2c37700`; local/remote preservation parity; original 17 identities unchanged | Complete; does not authorize an ambient-root integration wall |
| 1B. Broad-wall invocation correction | Commit exactly the six canonical files on parent `928f94e7b4c498273b40385f7bffea9e4f949700`; preserve only at `feat/preserve-a1-1d-5r2-2-broad-wall-invocation-20260722` | Clean source worktree; source remote still `2f6f1f69b3519dafff01ef543e7d260da2c37700`; local source 19 ahead; runtime and publication-authority identities unchanged; private-root contract frozen | Do not run the renewed integration wall or push the source in B1 |
| 1C. Remediation planning | Commit/review exactly the six control-pack files on `f7ded83ef147b748678ba6b028eea959870a04fe`; preserve only the planning branch | Planning ref parity; source remote unchanged; exact R1/P1 allowlists and sequence frozen | No implementation, wall, integration-clean ref, source push, history rewrite, or stale-wall eligibility |
| 1D. R1 implementation | One bounded two-file R1 commit, focused proof, and fresh authority/security review | Exact non-disclosure/argv/diagnostic/xtrace contract and review CLEAN | No P1 work, broader caller/helper change, weak validation, wall, or source push |
| 1E. P1 implementation | One bounded two-file P1 commit, adversarial self-tests, and fresh provenance/security review | Authenticated runner, containment, continuous-authority deletion, and review CLEAN | No product semantic/dependency/privileged change, fallback harness, wall eligibility claim, or source push |
| 2. Fresh baseline and integration wall | First run three parallel and one serial canonical wall through P1 with distinct roots; only after that baseline is CLEAN, run the complete renewed production-fix-free closeout | Every required baseline/integration artifact has complete P1 provenance and is proof-clean; closeout makes zero implementation/test change | Any invalid root/containment/deletion proof makes a wall ineligible; any failure/stop blocks source push, is classified, and requires docs-first future remediation |
| 3. Independent reviews | Four fresh read-only integration reviews of the exact proof state: authority/security, lifecycle/product, inventory/source closure, and baseline/platform/publication | Historical pre-disposition publication gate: all four reviewers CLEAN; a purely documentary finding is remediated docs-only and re-reviewed by a fresh replacement | Any finding that invalidates proof or topology stops publication and requires the affected proof/review to rerun from a corrected preserved state; no source push on any unresolved finding |
| 4. Final documentation | Draft the final integration-closeout update across the same six canonical files after the certified range/proof; validate/review to CLEAN; commit the exact reviewed bytes; rerun final documentation validation/reviews | Final committed docs prove the wall, keep R2-2 status truthful, and match the pre-commit reviewed bytes; any authorized remediation successor is append-only and six-file-only | Fix pre-commit documentary findings in the worktree; a post-commit documentary finding permits only an append-only remediation-doc successor plus renewed docs validation/review; a proof/topology-invalidating finding reruns affected proof/review; no rewrite or implementation/product/seam change |
| 5. Publication | Freshly read the source remote and require exact old OID `2f6f1f69b3519dafff01ef543e7d260da2c37700`; bind an explicit expected-old-OID CAS/lease to that exact OID; independently prove the update is an ordinary fast-forward | Source contains the original 17 runtime commits, authority docs, B1 docs, remediation-planning docs, bounded R1 and P1 commits, final-closeout docs, and only review-required append-only remediation-doc successors; local HEAD, upstream, and remote are identical at zero ahead/zero behind | The CAS/lease cannot authorize a forced/non-fast-forward update; any old-OID change, non-fast-forward, second/staged push, or identity mismatch is a hard stop |

F's completed replay and historical Route D docs-first replay remain scoped to their own packets.
The renewed closeout certifies the already assembled range, so it must not replay that range.
The docs-on-top -> one ordinary fast-forward publication model and phase ordering above remain
controlling, but phase 3's pre-disposition four-review-CLEAN gate is historical only. RP3 has
since completed with the unchanged canonical `1309/1264/45/0` baseline and zero pairwise
differentials. RP4 product proof on exact integration commit/tree
`8c46135c861a468dea316cf9fd7d6c6bb15bddac` / `5358497a8baec6f36e15aaef58415a759e64977d` is
accepted clean by human disposition; the raw persistence review remains `REQUEST_CHANGES` and is
preserved as three cache-only orchestration/attestation findings carried as non-blocking
process-audit debt outside RP4 product-proof scope. This exact six-file change was the bounded RP5
closeout docs packet. Source publication had not yet occurred in that run and was the exact next
step after the reviewed/committed RP5 docs; it later completed before the accepted R2-3 suffix and
refreshed native evidence closed R2-3, with R2-4 and R3 still later.

## R2-2 remediation insertion before renewed closeout (historical RP0-RP4 plan)

The failed renewed closeout inserts two docs-first remediation increments without changing the
existing commit identities or the final docs-on-top/one-fast-forward publication model. The table
below preserves the historical RP0-RP5 sequencing model that governed RP1-RP4; the controlling
current RP5 state is recorded above.

| Phase | Entry state | Authorized work | Exit gate | Forbidden |
|---|---|---|---|---|
| RP0 — planning authority (historical checkpoint) | Clean `f7ded83ef147b748678ba6b028eea959870a04fe`; source still at `2f6f1f69b3519dafff01ef543e7d260da2c37700`; two prior blockers | Source-close R1/P1; edit/review/commit exactly six control-pack Markdown files; preserve only the planning ref | Six-file scope and cross-document checks CLEAN; three fresh read-only docs reviewers CLEAN; planning preservation parity; source remote unchanged | Implementation/tests/scripts, wall rerun, integration-clean ref, source push, history rewrite, R2-3/R2-4/R3 |
| RP1 — R1 carrier non-disclosure | Reviewed RP0 authority | Add the exact structured redaction helper, change only `deploy_shims`, and extend only `prefix_propagation_r2_2.sh` | Focused adversarial proof CLEAN; byte-exact execution; carrier absent from every display/evidence surface; fresh authority/security review CLEAN | Carrier removal, weaker validation, generic regex redaction, `eval`, other production/test files, P1 work |
| RP2 — P1 tracked provenance runner | R1 implementation/review CLEAN | Add the exact two Linux-only test-harness files and no dependency; bind the reviewed P1 commit trailers to its bootstrap and three argv-template constants, then record separately the substituted actual argv hashes; implement only the `04`-frozen authenticated host → Stage A → Stage B topology, sealed runner projections, fixed private peer-credential socket, Stage-B `--as-pid-1` worker, explicit Stage-A output pipe, immutable seeds plus confined writable Cargo runtime, and host-finalized teardown | All 95 runner self-tests CLEAN; exact combined-output EOF/backpressure/bounds/hash proof; worker reap-to-`ECHILD`; Stage-A private-tree absence; host proof of Stage-A namespace/backing absence; containment/deletion/security review CLEAN; no product-flow effect | Product runtime/test semantics, service or privileged state, mutable external invocation artifact, ambient/unnamed supervisor, parent-only waiting, assumed arbitrary control/evidence FD inheritance, Stage-A self-finalization, pathname-only cleanup, or wall eligibility claim before every host gate |
| RP3 — canonical baseline | R1/P1 CLEAN | Run three parallel and one serial wall, each through P1 with a fresh validated private root | Every wall independently provenance-valid; canonical counts, hashes, and zero differential; no masked route | Reuse of old roots/results, fallback harness, implementation remediation |
| RP4 — renewed production-fix-free closeout | Fresh RP3 baseline CLEAN | Re-run the complete focused/integration/review wall without product/test changes | All gates and four fresh integration reviewers CLEAN | Any implementation/test remediation; classification of stale walls as eligible |
| RP5 — historical final docs + publication plan | RP4 CLEAN | Add final six-file closeout docs, validate/review, then one ordinary source fast-forward | Local/upstream/remote parity; final docs are the only commit added after the RP4-certified R1/P1 source state; R2-2 complete | Replay/rebase/merge/cherry-pick/squash/force/staged source publication |

The immutable order is:

```text
f7ded83
  -> RP0 / reviewed remediation-planning docs
  -> RP1 / R1
  -> RP2 / P1
  -> RP3 / fresh canonical baseline
  -> RP4 / renewed production-fix-free closeout
  -> RP5 / final six-file closeout docs
  -> one ordinary fast-forward source publication
  -> A1.1d-5R2-3
```

Historical closeout results stay historical: the four earlier matching walls remain ineligible and
the original NOT CLEAN/BLOCKED reviews remain findings, not passes. The table above is preserved as
historical sequencing only. RP3, RP4, RP5, and the ordinary fast-forward source publication later
completed at `0f1e147fb735791b44a65099a65167cbdc1803af`; the process-only bounded-review
calibration also completed. At that checkpoint, the published tip
`43e528af8c71c4f42a7b1238f729078f20ee3760` authorized only the
[docs-first R2-3D subdivision](../a1.1d-5r2-3/slice-and-task.md#a11d-5r2-3-bounded-docs-first-subdivision).
R2-3A product code remained a separately authorized future control-pack node.

**Source provenance:**
- extracted from [`03-phase-slice-map.md#a11d-5r2-2-b1-broad-wall-provenance-phase`](../03-phase-slice-map.md#a11d-5r2-2-b1-broad-wall-provenance-phase), lines 2120–2227; baseline span SHA-256 `ccb2a4db82ee7c5ad2d29f86d1291bbd58ea9ff184e19ddb9741945f57794e1a`
