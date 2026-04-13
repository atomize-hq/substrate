# `5a372fb` Validation Note

## Purpose

This note is the contract and checklist source of truth for what upstream commit `5a372fb` does and does not establish for this repository at the `SEAM-1` stage.
It is the verification-evidence portion of `C-01` for `THR-01`.

It is intentionally not a proof of Azure-specific behavior yet.

## Landed Tree And Identity

The repo-local baseline is the landed `gateway/` tree, built and named as `substrate-gateway`.

The exact verification commands remain rooted at that landed tree:

- build: `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
- smoke: `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`

## Current Status

- Upstream tool-call repair: present in `5a372fb` and implemented in the upstream provider path at `src/providers/openai.rs`
- Landed-tree analogue of that path: `gateway/src/providers/openai.rs`
- Verified for this seam: not yet against Azure Foundry
- Azure-specific hidden-tool behavior: unresolved at this stage
- Azure-specific reproduction and fixture work: outside this note and left for the normalization seam

## What `5a372fb` Actually Changes

The upstream commit is a narrow Kimi tool-call repair, not a gateway-wide Azure proof.

The patch in `src/providers/openai.rs`:

- adds dynamic text-block indexing so a trailing text delta does not collide with an earlier tool block
- prefers non-empty `content` and falls back to `reasoning` when Kimi emits empty content alongside reasoning text
- adds regression tests that cover Kimi tool-call ordering and empty-content fallback behavior

That means the commit improves how the provider transform preserves Kimi tool frames in the upstream code path, but it does not by itself prove the Azure Foundry behavior described in the handoffs.

## What Is Known Versus Unproven

What the upstream commit may change:

- upstream Kimi tool-call handling in the adopted codebase
- the provider transform path that downstream seams will later compare against Azure Foundry behavior

What remains unproven for Azure Foundry:

- whether Azure Kimi still hides tool intent in `reasoning_content`
- whether the hidden-tool markers from the prior handoffs still appear on Azure
- whether the upstream fix is enough for Azure without parser changes
- whether any provider boundary decision for Azure needs to change after real Azure evidence

## What This Note Must Preserve

The validation record must keep these distinctions explicit:

- upstream Kimi-related fixes are not the same as Azure Foundry hidden-tool proof
- native Moonshot behavior is not sufficient evidence for Azure Foundry behavior
- baseline contract freezing is not the same as normalization implementation
- the upstream repair in `src/providers/openai.rs` must stay separate from the Azure Foundry hidden-tool evidence in the handoff docs

## Required Checklist

The note must remain usable as a later implementation checklist and must preserve these exact verification commands:

- build: `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
- smoke: `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`

It must also record the evidence boundary for later Azure validation work:

- confirm whether `5a372fb` materially improves the Kimi tool-call path in the upstream provider transform
- confirm whether Azure Foundry still reproduces hidden-tool intent in `reasoning_content`
- confirm whether the observed Azure marker patterns require parser work beyond the upstream fix

Pass/fail criteria for later `S3` validation:

- pass if an Azure Foundry probe reproduces the hidden-tool gap described in the handoffs and the gateway records that behavior explicitly
- pass if the later fixture/probe evidence shows whether explicit `tool_calls` and hidden `reasoning_content` markers normalize to the same internal representation
- pass if the evidence set includes the exact request, response, and normalized-output artifacts needed to explain what `5a372fb` does and does not cover
- fail if the later note collapses native Moonshot behavior and Azure Foundry behavior into one claim
- fail if the later note says `5a372fb` is sufficient for Azure without an Azure-specific reproduction
- fail if the later note changes the provider boundary before the Azure hidden-tool gap is actually reproduced

Evidence expectations for later Azure validation work:

- the Azure model and endpoint used in the probe
- the request shape that exposed the hidden-tool path
- the raw `reasoning_content` or equivalent marker evidence
- the normalized internal event outcome
- a clear statement of whether the upstream commit changed anything observable for Azure

## Explicitly Unresolved Items

The following remain open:

- Azure Foundry hidden-tool reproduction
- Azure-specific regression fixtures
- provider parsing of hidden Kimi markers
- any conclusion that would change the provider boundary defined in `claude-code-mux-extension-boundary.md`
- the operational proof that `5a372fb` closes the Azure gap rather than only the upstream Kimi case

## Downstream Stale Triggers

Any of the following should force downstream revalidation instead of silent reuse:

- `SEAM-2` if the Azure evidence later proves the upstream repair is insufficient or if hidden-tool parsing needs a different contract than the one described here
- `SEAM-3` if this note is used to justify Anthropic surface behavior, raw stream handling, or any claim that the upstream repair settles client-facing semantics
- `SEAM-4` if this note is used to justify planner/executor policy, model-role selection, or any other internal routing decision
- `SEAM-5` if this note is used to justify multiple public backend identities, loopback-only deployment assumptions, or raw provider-stream downstream contracts

## Checklist Status

- Exact repo-root build command recorded: yes
- Exact repo-root smoke command recorded: yes
- Upstream tool-call repair separated from Azure evidence: yes
- Azure-specific validation resolved: no
- Later Azure evidence path defined: yes
