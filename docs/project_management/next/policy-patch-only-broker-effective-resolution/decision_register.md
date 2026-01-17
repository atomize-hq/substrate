# Decision Register — Policy Patch-only + Broker Canonical Effective Resolution (ADR-0013)

This decision register records the architectural decisions for `ADR-0013`.

Rules:
- Each decision is recorded as exactly two options (A/B), with explicit tradeoffs and one selection.
- No backwards compatibility or migration behavior is included unless explicitly stated.

---

## DR-0001 — Where the policy patch resolver lives (broker-owned vs shared crate)

**Decision owner(s):** Shell/Broker maintainers  
**Date:** 2026-01-17  
**Status:** Accepted  
**Related docs:** `docs/project_management/next/ADR-0013-policy-patch-only-broker-canonical-effective-resolution.md`, `docs/project_management/next/ADR-0008-workspace-config-policy-scope-and-dot-substrate-unification.md`

**Problem / Context**
- ADR-0008 establishes patch-only policy files with layered precedence, but the broker’s runtime loader still expects a strict full document and does “first match wins”.
- Multiple execution paths depend on `substrate_broker::detect_profile`, so policy resolution must be canonicalized in one place to avoid shell/shim/world-agent drift.
- This work explicitly rejects “auto-detect legacy formats” and “support both formats” as non-goals.
- Config also uses sparse patch YAML + provenance semantics (ADR-0008/ADR-0012), but this decision is specifically about policy resolution placement; shared patch mechanics can be refactored later without splitting policy authority.

### Option A (selected): Broker owns patch schema + merge + validation
- **Decision:** Implement policy patch parsing/merging/validation in `crates/broker`, and have other subsystems consume the broker-resolved effective policy.
- **Pros:**
  - Single canonical implementation for runtime policy resolution (prevents split-brain between shell vs broker).
  - Eliminates silent fallback risks because all execution paths share one resolver and one error policy.
  - Keeps the policy enforcement engine and its resolution semantics co-located.
- **Cons:**
  - Requires moving/refactoring existing shell-side merge plumbing or re-implementing it in the broker.
  - Requires careful API design to keep the shell’s CLI UX (`current show --explain`) intact while removing duplication.
- **Cascading implications:**
  - Legacy broker structs and YAML serializers that encode full-policy documents are removed/rewritten as patch-only.
  - Workspace discovery used by the broker must match shell semantics (including `.substrate/workspace.disabled`).
- **Risks:**
  - Temporary duplication risk during refactor if shell + broker both implement merge logic in parallel.
- **Unlocks:**
  - Makes `detect_profile` safe for sparse patch files across shell/shim/world-agent.
- **Quick wins / low-hanging fruit:**
  - Fixes the “minimal patch breaks interactive execution” repro without requiring any CLI workflow changes.

### Option B: New shared crate provides patch resolution (broker calls it)
- **Pros:**
  - Avoids duplicating merge logic across broker and shell while still making broker the canonical runtime caller.
  - Creates a natural home for shared workspace discovery + patch parsing utilities.
- **Cons:**
  - Adds a new crate and dependency surface that must be curated to avoid cycles (`crates/shell` must not be depended on).
  - Risks re-introducing split-brain if shell starts calling the shared crate directly instead of consuming broker-resolved effective policy.
  - Increases “where do I change policy semantics?” ambiguity unless strictly governed.
- **Cascading implications:**
  - Requires a stable, well-defined crate boundary and a policy on what can depend on it.
  - Still requires removing full-policy format support from broker, docs, and tests.
- **Risks:**
  - The shared crate becomes a de-facto second policy semantics authority if adoption is inconsistent.
- **Unlocks:**
  - Potential reuse for future config/policy convergence work beyond this ADR, if governance stays strict.
- **Quick wins / low-hanging fruit:**
  - Faster initial refactor if existing merge logic can be lifted with minimal edits.

**Recommendation**
- **Selected:** Option A — Broker owns patch schema + merge + validation
- **Rationale (crisp):** Policy resolution must be canonicalized where policy is enforced; co-locating patch resolution in the broker eliminates shell/broker drift with the smallest governance surface.
