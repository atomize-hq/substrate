# Decision Register — Warn on `config global show` when workspace config overrides

This register contains the architectural decisions for this planning pack.

## DR-0001 — When to emit the workspace-override note

Context:
- `substrate config global show` prints the global patch, but the effective config in a workspace may differ.

### Option A — Always warn when inside an enabled workspace
- Emit the note whenever a workspace root is detected, regardless of workspace patch contents.

Tradeoffs:
- ✅ No need to read/parse the workspace config file.
- ❌ Noisy: warns even when the workspace patch is empty (`{}`), which is a common state.
- ❌ The note would not mean “an override is active”, only “a workspace exists”.

### Option B — Warn only when the workspace override is active
Definition (active):
- Workspace root is detected AND
- workspace patch is non-empty, OR parsing fails (treated as active without failing the command).

Tradeoffs:
- ✅ Matches the intent of the backlog item (“only when an override applies”).
- ✅ Avoids noise in empty-workspace-patch cases.
- ✅ Still robust in invalid-YAML cases (no new failure modes).
- ❌ Requires reading/parsing the workspace config file (with parse errors handled gracefully).

Decision:
- **Choose Option B**.

Links:
- Spec: `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md`
- Contract: `docs/project_management/next/warn-config-global-show-workspace-overrides/contract.md`

## DR-0002 — What the note should contain (path + guidance)

Context:
- The note must direct operators to the effective config view and be stable for log scanning.

### Option A — Include the workspace config path and point to `substrate config show --explain`
- Message includes `<workspace_root>/.substrate/workspace.yaml`.
- Message guidance: `run 'substrate config show --explain' ...`.

Tradeoffs:
- ✅ Operator can immediately see *which* workspace file is in play (important for nested dirs).
- ✅ `--explain` shows provenance, aligning with the “why does this differ?” question.
- ❌ Message is slightly longer.

### Option B — Omit the path and point to `substrate config show` (no `--explain`)
- Message is shorter and purely directional.

Tradeoffs:
- ✅ Shorter.
- ❌ Less actionable when multiple workspaces exist or when the operator needs provenance.

Decision:
- **Choose Option A**.

Links:
- Spec: `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md#note-text-exact-template`

## DR-0003 — Interaction with the existing “global patch is empty” note

Context:
- Today, `config global show` emits a note when the global patch is empty.
- With the new workspace-override note, it is possible to emit two notes in one invocation
  (global patch empty + workspace override active).

### Option A — Emit both notes when both conditions apply
Tradeoffs:
- ✅ Preserves all prior messaging.
- ❌ Produces noisy, repetitive stderr output.

### Option B — Suppress the global-empty note when the workspace-override note is emitted
Tradeoffs:
- ✅ Keeps stderr minimal and high-signal.
- ✅ The workspace note already points to the effective view.
- ❌ Slight behavior change in the “global empty + workspace override active” scenario
  (only affects stderr notes, not stdout).

Decision:
- **Choose Option B**.

Links:
- Spec: `docs/project_management/next/warn-config-global-show-workspace-overrides/C0-spec.md#interaction-with-existing-notes`
