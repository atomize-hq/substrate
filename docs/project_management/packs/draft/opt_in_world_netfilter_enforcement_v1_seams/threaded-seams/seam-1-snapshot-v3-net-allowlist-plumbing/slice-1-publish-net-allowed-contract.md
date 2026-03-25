---
slice_id: S1
seam_id: SEAM-1
slice_kind: delivery
execution_horizon: active
status: decomposed
plan_version: v1
basis:
  currentness: current
  basis_ref: seam.md#basis
  stale_triggers:
    - "Any change to net_allowed canonicalization/validation rules"
gates:
  pre_exec:
    review: inherited
    contract: inherited
    revalidation: inherited
  post_exec:
    landing: pending
    closeout: pending
threads:
  - THR-01
contracts_produced:
  - C-01
contracts_consumed: []
open_remediations: []
candidate_subslices: []
---
### S1 - Publish Snapshot V3 `net_allowed` contract (C-01)

- **User/system value**: make `net_allowed` an explicit cross-boundary input so the world does not rely on hidden in-guest broker state for allowlists.
- **Scope (in/out)**:
  - In: Snapshot V3 schema field + canonicalization/validation helpers + unit tests.
  - Out: host/world-agent plumbing (covered in `S2`/`S3`).
- **Acceptance criteria**:
  - `PolicySnapshotV3.net_allowed: Vec<String>` exists with `#[serde(default)]`.
  - Canonicalization rules are explicit and test-locked, including:
    - trim + drop empty
    - dedupe
    - collapse any `"*"` presence to exactly `["*"]`
    - reject non-`"*"` wildcard forms (e.g., `"*.example.com"`) when enforcement is requested
  - Hostname normalization posture is explicitly decided (casefolding + IDNA) and resolves `REM-001`.
- **Dependencies**:
  - `../../threading.md` (`C-01`, `THR-01`)
- **Verification**:
  - `cargo test -p agent-api-types` (unit tests for canonicalization/validation)
- **Rollout/safety**:
  - Additive-only: missing `net_allowed` in older snapshots defaults via serde to `[]` (or an explicitly decided default), with no behavior change unless downstream opt-in gating requests isolation.
- **Review surface refs**:
  - `../../review_surfaces.md` (R2/R3)
  - `review.md` (mismatch hotspots: normalization drift, wildcard semantics)

#### S1.T1 - Decide and document hostname normalization rules

- **Outcome**: explicit rules for casefolding + IDNA posture that every consumer uses.
- **Inputs/outputs**:
  - In: `REM-001`, `scope_brief.md` assumptions, current broker semantics (evidence).
  - Out: a single normalization helper used by snapshot builder + world-agent plumbing.
- **Thread/contract refs**: `THR-01`, `C-01`
- **Contract decision (C-01): `net_allowed` hostname normalization posture**
  - **Trim/drop**: for each entry, `trim()` leading/trailing whitespace; drop entries that become empty.
  - **Casefolding**: ASCII-only lowercasing (`A-Z` → `a-z`) is applied before validation/deduping.
  - **Trailing dot**: strip all trailing `.` so `example.com.` canonicalizes to `example.com` (reject if this produces empty).
  - **IDNA posture**:
    - Do **not** apply IDNA mapping or Unicode normalization as part of canonicalization.
    - Non-ASCII hostnames are **rejected**.
    - Internationalized domains must be supplied in ASCII A-label / punycode form (e.g. `xn--...`).
  - **Allowed forms**:
    - Hostnames only: no schemes, ports, paths, queries, or fragments (examples to reject: `https://example.com`, `example.com:443`, `example.com/path`).
    - IP literals are permitted as entries (IPv4 `1.2.3.4`, IPv6 `::1`; bracketed IPv6 `[::1]` canonicalizes to `::1`).
  - **Label validation** (hostnames):
    - Labels are `a-z`, `0-9`, `-` only.
    - No empty labels, and labels must not start/end with `-`.
    - Hostnames must not start/end with `.`.
  - **Deduping**: de-dupe after canonicalization, preserving first-seen order.
  - **Wildcard posture**: unchanged from S1 acceptance criteria (literal `"*"` means allow-all and collapses the list to exactly `["*"]`; other wildcard forms are rejected when enforcement is requested).
- **Verification plan (unit tests, planning-only)**:
  - Add unit tests in `crates/agent-api-types` that lock the above behavior:
    - `policy_snapshot_v3_net_allowed_casefolds_and_strips_trailing_dot`
    - `policy_snapshot_v3_net_allowed_rejects_unicode_idna_input`
    - `policy_snapshot_v3_net_allowed_rejects_urls_ports_and_paths`
    - `policy_snapshot_v3_net_allowed_collapses_star_to_singleton`
    - `policy_snapshot_v3_net_allowed_allows_ip_literals`
  - Edge cases that must be covered: uppercase hostnames, leading/trailing whitespace, Unicode labels, trailing dot, `"*"` canonicalization.
- **Risk/rollback notes**: normalization changes are a downstream stale trigger and must be captured in seam closeout.

Checklist:
- Implement: normalization helper + validation rules
- Test: unit tests for canonicalization + normalization cases
- Validate: ensure serde default/back-compat behavior is explicitly asserted
- Cleanup: remove any duplicate ad-hoc normalization code paths
