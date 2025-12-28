# I7-spec: Manual Testing Playbook Alignment (I0–I5)

## Scope
- Reconcile `manual_testing_playbook.md` with:
  - I0–I5 specs (`I0-spec.md` … `I5-spec.md`)
  - Current policy schema documented in `docs/CONFIGURATION.md`
  - The role/task expectations described in the I0–I5 kickoff prompts
- Fix concrete playbook drift:
  - Ensure every `.substrate-profile` example includes the required top-level `id` and `name`.
  - Ensure the playbook does not claim behaviors that are not specified by I0–I5 specs.
  - Ensure the playbook’s expected outputs are accurate and actionable.

### Exit codes in the playbook
- I0–I5 specs do **not** currently define exit-code mappings for these scenarios.
- The playbook must therefore:
  - Avoid asserting specific numeric exit codes unless the relevant spec explicitly defines them, or
  - If it references exit codes, it must quote the canonical taxonomy and explain that the current CLI
    may differ until an explicit exit-code triad lands.

### Guardrails
- This triad is documentation-first: it should not introduce new runtime behavior requirements.
- Any “new contract” changes (e.g., changing CLI exit codes) belong in a separate triad/spec.

## Acceptance
- `docs/project_management/next/p0-agent-hub-isolation-hardening/manual_testing_playbook.md` no longer
  contains invalid `.substrate-profile` examples (i.e., missing `id`/`name`).
- Each section (I0–I5) matches the relevant spec’s scope and acceptance criteria.
- The playbook remains runnable on Linux and includes correct macOS/Windows notes where applicable.
- Automated coverage (test-only) prevents obvious playbook drift (at minimum: missing `id`/`name` in
  the embedded `.substrate-profile` snippets).

## Out of Scope
- Changing Substrate runtime behavior (including exit code mappings).
- Adding new verifier commands (`substrate world verify`) — handled in I6.
