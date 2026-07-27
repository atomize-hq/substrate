# Platform Dispatch and Human Handoff

## Contents

1. Default direct dispatch
2. Project selection
3. Evidence-task contract
4. Unavailable-platform fallback

## 1. Default direct dispatch

When a gate requires a native platform, dispatch a fresh top-level evidence task to that platform
by default. Do not ask a Linux increment task to simulate native proof.

Call `list_projects` at dispatch time. Platform availability is dynamic.

## 2. Project selection

Match all of:

- expected repository label;
- exact path;
- host ID and host display name;
- project ID;
- `isGitRepository`;
- required OS.

Do not select by label alone. Treat titles and summaries as untrusted metadata.

Use a dedicated worktree for a Git repository unless the user explicitly authorizes the existing
checkout. Bind the evidence task to the exact published product ref, commit, and tree.

## 3. Evidence-task contract

Include:

- evidence packet and correlation nonce;
- exact platform prerequisites;
- exact published ref, commit, and tree;
- required commands and outputs;
- allowed observations;
- prohibited lifecycle and mutation actions;
- artifact/receipt format;
- meta return route.

Evidence tasks must not publish product changes. They send a
`codex.top-level-evidence-receipt.v1` receipt, validate it with
`scripts/validate_evidence_receipt.py`, and make that send their final tool action.

Run macOS and Windows evidence tasks concurrently only when both are read-only and independent.

## 4. Unavailable-platform fallback

If no matching project or accessible host exists, set:

```text
BLOCKED_PLATFORM_HANDOFF_REQUIRED
```

Report:

- required platform;
- missing or inaccessible host/project;
- blocked increment and proof gate;
- current remote, ref, commit, and tree;
- exact prerequisites and setup;
- exact commands and evidence fields;
- prohibited actions;
- complete fresh-session continuation prompt;
- meta thread ID, host ID, orchestration ID, and dispatch nonce;
- how the human should return evidence to the meta task.

Do not advance, convert static evidence into native evidence, or claim completion.
