# Sanitized R7 Delegated Acceptance Matrix

`matrix.json` is the analyzer-owned R7-5.1 acceptance corpus. It contains ten synthetic cases that
map one-to-one to the R7 spec matrix. Each case names sanitized session trajectories, a compact row
recipe, typed delegation-link observations, and the exact analyzer delegation result expected for
every included session.

The row recipes deliberately preserve only the behavioral shape needed to prove ownership:

- `parent_clean` and `parent_wait_loop` contain bounded orchestration-only activity;
- `child_minimal`, `child_advances`, and `child_stalls` exercise child-local analysis;
- `single_agent_clean` pins the ordinary non-delegated control; and
- link states cover verified, parent-only, child-only, conflicting-parent, and deeper-residue input.

The acceptance test builds a v0.2 bundle through the normal test bundle writer, reloads it through
the production analyzer input path, and then asserts link state, graph admission, topology,
visibility, confidence, progress ownership, and scorer ownership. Non-verified links must never
enter the semantic graph. The two behavioral cross-trajectory cases additionally pin
parent-visible progress and child-local `dead_end_thrash` attribution.

These are synthetic fixtures, not copied rollouts. They contain no real prompts, private paths,
credentials, raw tool output, real session or call ids, user names, or agent nicknames.
