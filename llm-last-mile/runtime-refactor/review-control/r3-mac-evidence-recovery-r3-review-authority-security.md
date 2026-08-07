# Authority/security review — AUX-R3-MAC-EVIDENCE-RECOVERY-R3

## Discovery review boundary

Fresh read-only authority/security review by a `gpt-5.6-terra` Extra High reviewer of the
five-path R3 product/test/documentation subject bound to `sha256:6e27dae469b25c297abd33de5d0cdacae97eedb51b2f69a1a7474a61bc8f136c`. The reviewer did not mutate,
test, install, stage, or publish.

## Discovery findings and disposition

| ID | Priority | Disposition | Evidence |
|---|---|---|---|
| `P1-R3-PROJECT-ID-EMPTY` | P1 | remediated before closure | `argparse` required flag presence but accepted an empty value, and equality alone accepted an empty artifact value when the expected value was also empty. `validate_artifact` now requires non-empty `expected_product_project_id`; the missing-ID regression also asserts blank-value rejection. |
| `R3-AUTH-HISTORICAL-DOC-OBSERVATION` | not a valid P1/P2 | no mutation permitted or needed | The older historical command in `04-contracts-and-gates.md` is frozen pre-recovery text. This packet is append-only and must not rewrite prior Linux evidence. The appended recovery-current invocation carries the required caller-bound flag, so the observation is not a current authority contradiction. |

## Closure review after remediation

Fresh independent read-only closure review by a different `gpt-5.6-terra` Extra High reviewer of
the remediated five-path subject bound to `sha256:4389d61398923bee07fb9f67dce89b9744d5a5eb79444214cb15fca6128eb0df`. The reviewer did not mutate, test, install,
stage, or publish.

**CLEAN.** The required CLI flag is enforced; an empty expected ID is rejected before artifact
comparison; artifact and caller values are fail-closed equal; the historical Linux fixture is
accepted only with its explicit historical ID; and all three appended recovery-current invocations
carry `--expected-product-project-id`. No P1/P2 finding remained.
