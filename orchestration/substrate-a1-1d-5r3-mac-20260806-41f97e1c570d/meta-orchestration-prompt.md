Use $orchestrate-top-level-tasks.

You are the persistent top-level meta orchestrator for:

substrate-a1-1d-5r3-mac-20260806-41f97e1c570d

INITIALIZATION BARRIER

Do not create or message an increment task until a follow-up binds your own real thread ID,
host ID, state path, target ref, and explicit start authority.

After identity binding:

1. Read the orchestration state at /Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/state.json.
2. Read the reusable protocol from /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.agents/skills/orchestrate-top-level-tasks/SKILL.md.
3. Verify the exact product target:
   - publication mode: read from durable state and the increment contract
   - remote: origin
   - ref: refs/heads/feat/internal-host-orchestrator-world-dispatch-bootstrap
   - expected commit: 270f6e55e1a94b7e2f9b2667e605980d2e50579c
   - expected tree: acef6844c15d17aba8cfd18d2fc7ef88d9c3420b
4. Preserve these protected checkouts:
- /Users/spensermcconnell/__Active_Code/atomize-hq/substrate
5. Follow this exact sequence:
1. A1.1d-5R3-MAC
2. EVIDENCE:R3-MAC-IMP-01
3. A1.1d-5R3-MAC-CLOSEOUT
4. Stop at AUTHORITY_REQUIRED:A1.1d-5R3-WIN
6. Create one fresh top-level increment orchestrator at a time.
7. Become idle after dispatch.
8. Advance only after independently validating the publication-mode-matching receipt:
   `LANDED_CLEAN` for remote publication or `CLOSED_CLEAN` for local two-commit closeout.
9. Validate the receipt successor against the authoritative index; use `EVIDENCE:<increment>` for
   a declared native gate before that increment.
10. Dispatch native evidence directly to matching platform projects by default.
11. Bind each evidence task to its required evidence ID, platform, project ID/path, host/task
    identities, source checkpoint, and nonce; require its task to become terminal before
    acceptance.
12. Require verified evidence to exactly satisfy durable `required_evidence` before the gated
    increment or overall completion.
13. If a required platform is unavailable, stop as BLOCKED_PLATFORM_HANDOFF_REQUIRED and emit the
    complete human handoff package.

Render each increment prompt from:

- template: /Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/increment-orchestrator-prompt-template.md
- increment contracts: /Users/spensermcconnell/.codex/worktrees/eb49/substrate/orchestration/substrate-a1-1d-5r3-mac-20260806-41f97e1c570d/increments
- renderer: /Users/spensermcconnell/__Active_Code/atomize-hq/substrate/.agents/skills/orchestrate-top-level-tasks/scripts/render_increment_prompt.py

Persist state and verified receipts only on the isolated orchestration branch. Never merge those
artifacts into the product target.

For local closeouts, persist the pre-dispatch remote observation in state, independently run
`verify_local_closeout.py --state <state.json>`, and require no merge, push, or remote mutation.
Do not infer new product authority. Do not merge, rebase, reset, clean, or force-push a product
checkout. Do not run implementation yourself.
