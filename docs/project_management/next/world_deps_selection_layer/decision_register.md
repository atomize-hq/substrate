# World Deps Selection Layer — Decision Register

This document is the single source of truth for architectural decisions made while implementing ADR-0002.

## Rules (non-negotiable)
- Every decision is captured with **exactly two** viable options (Option A / Option B).
- The final decision is **explicit** (no TBDs, no “optional”, no open questions).
- Every option includes: pros, cons, cascading implications, risks, what it unlocks, and low-hanging fruit/quick wins.
- Every decision ends with a single **recommended** option and a crisp rationale.
- Persist all research findings here (and/or in additional docs under this triad), with enough detail that a teammate can implement without interpretation.

## Template

### DR-0001 — <Decision Title>

**Decision owner(s):** <names/role>  
**Date:** <YYYY-MM-DD>  
**Status:** Proposed | Accepted | Superseded  
**Related docs:** `docs/project_management/next/ADR-0002-world-deps-install-classes-and-world-provisioning.md`, <other links>

**Problem / Context**
- <What is being decided and why now?>

**Option A — <Name>**
- **Pros:**
  - <…>
- **Cons:**
  - <…>
- **Cascading implications:**
  - <…>
- **Risks:**
  - <…>
- **Unlocks:**
  - <…>
- **Quick wins / low-hanging fruit:**
  - <…>

**Option B — <Name>**
- **Pros:**
  - <…>
- **Cons:**
  - <…>
- **Cascading implications:**
  - <…>
- **Risks:**
  - <…>
- **Unlocks:**
  - <…>
- **Quick wins / low-hanging fruit:**
  - <…>

**Recommendation**
- **Selected:** Option <A|B> — <Name>
- **Rationale (crisp):** <Why this is the best trade-off given Y0/I0–I5/C0–C9 constraints>

**Follow-up tasks (explicit)**
- <List concrete tasks/spec updates that implement the decision>

