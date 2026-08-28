**Kind:** architecture
**Stable ID:** `shared-review-question`
**Canonical for:** shared architecture review question only
**Status:** canonical shared-architecture record
**Authority scope:** exact extracted shared review-question source body only; no implementation authority
**Source span:** [`../01-target-architecture.md#review-question`](../01-target-architecture.md#review-question) lines 487–493
**Supersedes:** canonical ownership of the extracted source body; source heading remains a compatibility anchor
**Superseded by:** none
**Projection consumers:** [`README.md`](README.md), [`../01-target-architecture.md`](../01-target-architecture.md), [`../a1.1d-5r2-2f/README.md`](../a1.1d-5r2-2f/README.md)

# Shared architecture review question

## Review question

Every refactor PR must be able to answer:

> Can Substrate prove exact session identity, exact applicable binding, exact applicable policy snapshot, exact applicable credential handoff, exact applicable work receipt, and durable lifecycle/obligation truth for this action regardless of ingress surface?

If the answer depends on a helper still running, a socket being reachable, a terminal tool call returning, or an env variable being trusted, the target architecture has not landed.
