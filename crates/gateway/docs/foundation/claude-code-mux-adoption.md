# Claude Code Mux Adoption

## Purpose

This note freezes the repo-local adoption topology for `claude-code-mux` as the foundation contract for `SEAM-1`.
It is the adoption-topology portion of `C-01` for `THR-01`.

It is intentionally contract-level only:

- it does not import the upstream codebase yet
- it does not define Azure normalization behavior
- it does not define Anthropic surface behavior beyond the baseline contract

## Archived-Source Rationale

The source repository is archived, so this seam treats the upstream codebase as a fixed starting point rather than a living dependency.

That matters for two reasons:

- `SEAM-1` must freeze one real baseline and one real verification path before any downstream seam makes assumptions about the runtime shape.
- The seam/review docs require the plan to stabilize the adopted baseline first, then perform identity renames, then begin feature work. Any move that skips that ordering is a stale trigger for this contract.

## Upstream Evidence

- Imported snapshot: `/tmp/claude-code-mux-upstream`
- Upstream commit: `2d6133c164e2db97575809d8f1d765d6285443d1`
- Verified pre-rename build path: `cargo build --manifest-path gateway/Cargo.toml`
- Verified pre-rename help path: `cargo run --manifest-path gateway/Cargo.toml -- --help`
- Renamed package/binary identity: `substrate-gateway`
- Renamed runtime/config directory: `~/.substrate-gateway`

## Frozen Adoption Topology

The adopted foundation is defined as:

- the archived `claude-code-mux` codebase becomes the repo's primary starting codebase
- the adopted source lands under `gateway/`
- the Rust crate identity is renamed in `gateway/Cargo.toml` to `package.name = "substrate-gateway"`
- repo-local naming changes follow baseline stabilization, not precede it

## Identity Pass Surfaces

The repo-local identity pass must cover all naming surfaces that are implied by the upstream clone evidence and by the seam/review docs:

- crate/package identity: `gateway/Cargo.toml` -> `substrate-gateway`
- binary and CLI command identity: upstream `ccm` and `Claude Code Mux` labels become repo-local gateway naming
- config/runtime directory identity: upstream `~/.claude-code-mux/` and its files become repo-local gateway runtime/config naming
- runtime files under that directory, including config, tracing, routing, and pid/state files
- repo-local labels and documentation strings that still refer to the old project name
- user-facing service/help text and status output that would otherwise keep the upstream identity visible

## Observed Deviations

- No blocking baseline deviations were found during the import, build, or help proof.
- The only intentional compatibility deviation retained in code is the legacy config migration bridge for `config/default.toml` and `~/.claude-code-mux/config.toml`; this keeps pre-existing local state readable while the renamed runtime now prefers `~/.substrate-gateway/config.toml`.
- Any remaining `claude-code-mux` / `ccm` / `.claude-code-mux` strings after the doc sweep are either upstream provenance/history, compatibility migration paths, or internal references that do not present the current repo-local interface.
- No Azure/provider work was started in this slice.

The upstream evidence for these surfaces appears in the `src/main.rs`, `src/cli/mod.rs`, `src/server/mod.rs`, and `src/router/mod.rs` file families, which embed the command name, config path, runtime directory, and banner/service labels.

## Required Order

The contract order is:

1. Establish the adopted baseline under `gateway/`.
2. Stabilize the baseline close to upstream behavior.
3. Rename repo-local identity surfaces, including crate, binary, config, and documentation labels.
4. Begin provider normalization and gateway feature work only after the baseline and rename pass are proven stable.

## Contract Boundaries

This foundation note deliberately does not assume:

- an upstream sync path, because the source repository is archived
- loopback HTTP as a permanent architecture
- direct host credential access in the core request path
- Anthropic-only data structures at the core boundary

Those constraints are handled by the extension-boundary note and later seam work, not by this adoption note.

## Repo-Root Verification Checklist

The baseline verification surface for later implementation is fixed to the repo root and must remain exactly this shape:

- build: `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
- smoke: `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`

Implementation evidence from this slice:

- the pre-rename baseline built and showed the upstream `ccm` help surface successfully
- the repo-local identity pass then renamed the package, binary, help banner, config directory, pid path, token-store path, and statusline path to `substrate-gateway`
- the rename preserved the baseline command shape while changing only identity labels and the minimum compatibility paths needed to keep the renamed tree usable
- post-rename verification succeeded with `cargo build --manifest-path gateway/Cargo.toml -p substrate-gateway`
- post-rename help succeeded with `cargo run --manifest-path gateway/Cargo.toml -p substrate-gateway -- --help`
- tests were practical and passed with `cargo test --manifest-path gateway/Cargo.toml -p substrate-gateway`
- the only follow-up needed for testability was updating example/test imports from `claude_code_mux` to the new library crate name `substrate_gateway`

Pass/fail conditions:

- pass if the build command succeeds from the repo root with the adopted crate identity resolved through `gateway/Cargo.toml`
- pass if the smoke command succeeds from the repo root and reaches the gateway help path without needing `cd gateway`
- fail if either command requires an alternate manifest path, a different crate name, or a loopback-only bootstrap that leaks into the core request path
- fail if the baseline depends on direct host credential access, always-on host process state, or any other host-only assumption that `IMPORTANT_SUBSTRATE_ALIGNMENT.md` forbids
- fail if the identity pass cannot make the repo-local runtime/config directory, CLI command, and docs labels consistent with the new project name

## Checklist Status

- Adoption topology defined: yes
- Baseline-first sequencing defined: yes
- Identity rename targets defined: yes
- Runtime adoption performed: yes
- Upstream import performed: yes
