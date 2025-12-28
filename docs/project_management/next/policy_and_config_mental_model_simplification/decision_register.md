# Decision Register — Policy + Config Mental Model Simplification (ADR-0003)

This decision register exists because ADR-0003 contains multiple non-trivial architectural decisions. Each entry records exactly two options (A/B), their tradeoffs, and one selection.

## DR-0001 — Workspace root marker

- Option A: Workspace root is the nearest ancestor containing `.substrate/workspace.yaml`.
  - Pros: explicit, unambiguous, supports “workspace exists” gating; enables parent-walk discovery without relying on `.git`.
  - Cons: requires an explicit init step; workspaces without the file are “not a workspace”.
- Option B: Workspace root is inferred from `.substrate/` directory or `.git/`.
  - Pros: less explicit setup for some repos.
  - Cons: ambiguous (multiple `.substrate/` uses), breaks non-git workspaces, complicates discovery and gating semantics.
- Selection: Option A.

## DR-0002 — Canonical file inventory

- Option A: Global defaults in `$SUBSTRATE_HOME/` + workspace overrides in `<workspace_root>/.substrate/` with a strict, finite set of file names.
  - Pros: predictable, testable, single mental model; avoids “policy vs profile vs settings” collisions.
  - Cons: breaking change; requires updates to existing docs/sprints.
- Option B: Keep multiple legacy file names and discovery fallbacks.
  - Pros: fewer immediate breakages.
  - Cons: preserves ambiguity and drift; increases future integration complexity.
- Selection: Option A.

## DR-0003 — Backwards compatibility policy

- Option A: No backwards compatibility; remove all legacy keys/files/flags/env vars.
  - Pros: simplest mental model; reduces ongoing maintenance; aligns with `sequencing.json` greenfield breaking policy.
  - Cons: immediate breaking changes for existing installs.
- Option B: Compatibility layer with aliases and migrations.
  - Pros: smoother upgrade.
  - Cons: preserves ambiguity and slows simplification; increases test and support burden.
- Selection: Option A.

## DR-0004 — Anchor mode naming

- Option A: `world.anchor_mode=workspace|follow-cwd|custom` (rename “project” to “workspace”).
  - Pros: matches the workspace model and avoids conflating “project” with multiple repo layouts.
  - Cons: breaking rename.
- Option B: Keep `project` terminology.
  - Pros: no rename.
  - Cons: inconsistent with “workspace” as the primary scoping concept in ADR-0003.
- Selection: Option A.

## DR-0005 — Environment script responsibilities

- Option A: Split `$SUBSTRATE_HOME/env.sh` (stable exports) from `$SUBSTRATE_HOME/manager_env.sh` (runtime glue that sources `env.sh`).
  - Pros: eliminates drift; makes cached state stable; runtime can rewrite manager glue safely.
  - Cons: introduces a new file and clear ownership rules.
- Option B: Store stable exports in `manager_env.sh` and allow runtime to rewrite it.
  - Pros: fewer files.
  - Cons: inherently drifts (exports clobbered by runtime rewrites); inconsistent behavior across code paths.
- Selection: Option A.

## DR-0006 — World enable “home” semantics

- Option A: `substrate world enable --home <PATH>` sets `$SUBSTRATE_HOME` for the operation and all state writes live under that home.
  - Pros: unambiguous; one root for logs/config/env scripts; consistent helper invocation.
  - Cons: breaking CLI change.
- Option B: Keep `--prefix` as “install root” while metadata writes still use `$SUBSTRATE_HOME`.
  - Pros: avoids changing the helper contract.
  - Cons: split-brain; confusing; easy to provision one install and update another home.
- Selection: Option A.

## DR-0007 — Policy mode taxonomy

- Option A: `policy.mode=disabled|observe|enforce`.
  - Pros: distinguishes “no evaluation” vs “evaluate but do not block”; supports audit and agent planning.
  - Cons: requires plumbing and tests.
- Option B: Only `observe|enforce`.
  - Pros: simpler.
  - Cons: cannot represent “logging/world only” without policy evaluation.
- Selection: Option A.

## DR-0008 — Command pattern semantics

- Option A: `*` => glob; otherwise substring match.
  - Pros: simple, predictable, minimal foot-guns.
  - Cons: limited expressiveness vs regex.
- Option B: Regex-based patterns.
  - Pros: expressive.
  - Cons: harder to reason about; higher risk of accidental broad matches.
- Selection: Option A.

## DR-0009 — Protected sync excludes

- Option A: Always-on protected excludes (`.git/**`, `.substrate/**`, `.substrate-git/**`) that cannot be removed.
  - Pros: prevents corrupting workspace/system state; safe by default.
  - Cons: less configurable.
- Option B: Allow full user control of excludes.
  - Pros: maximum flexibility.
  - Cons: easy to break repositories and Substrate internal state.
- Selection: Option A.

## DR-0010 — Nested workspace behavior

- Option A: Refuse nested workspaces (exit `2`, no writes) if a parent workspace exists.
  - Pros: avoids ambiguity about workspace root; simplifies discovery and gating.
  - Cons: restricts nested repo setups.
- Option B: Allow nested workspaces.
  - Pros: supports monorepos with nested roots.
  - Cons: introduces ambiguity in discovery and policy/config layering.
- Selection: Option A.

