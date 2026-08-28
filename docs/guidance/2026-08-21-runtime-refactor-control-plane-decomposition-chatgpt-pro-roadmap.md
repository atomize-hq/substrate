# ChatGPT Pro advisory: runtime-refactor control-plane decomposition

- Date: 2026-08-21
- Task: preservation-first roadmap for decomposing the runtime-refactor control-plane documents into smaller focused documents.
- Source chat: https://chatgpt.com/c/6a88ba0f-c554-83ea-ac1a-6a671d69e893
- Current approach: advisory planning only; no roadmap item authorizes an implementation change.

> Advisory only; verify against local project truth and authoritative docs; do not reduce scope without user approval.

---

46e9461b-fac5-4763-9375-531e6fdf9ed2.zip
Zip Archive
The attached ZIP is a bounded, content-preserving snapshot of the Substrate runtime-refactor control-plane documentation.

Read BUNDLE_PROVENANCE.md first, then bundle-manifest.json, then worktree.patch, then the source under repo/. Treat excluded paths as unavailable; do not infer their contents.

Task: produce an advisory, safe roadmap to decompose the current runtime-refactor control-plane documentation into smaller, focused documents.

This is a first-phase planning exercise. Preserve all existing substantive text, authority, history, and links: do not delete, condense, rewrite semantically, or propose code/test/installer changes. The desired first migration decomposes material into focused documents by slice, seam, packet/task, gate/contract, scheduling decision, and evidence/receipt, before any later selective minimization.

Give:
1. a document taxonomy and canonical-authority/projection rules;
2. an extraction/decomposition sequence of independently landable docs-only slices;
3. proposed directory/index/navigation structure;
4. link, provenance, historical-record, and active-decision preservation rules;
5. validation/review/rollback gates for each slice; and
6. the smallest safe first implementation slice.

Do not reduce the task or replace it with an easier alternative. Return a roadmap only. Do not produce a patch.
Show more
Worked for 24m 4s
Advisory roadmap: preservation-first decomposition of the runtime-refactor control plane
Bounded baseline

This roadmap treats the uploaded ZIP as the sole evidence source.

46e9461b-fac5-4763-9375-531e6fd…

The bundle establishes several constraints that should remain visible throughout the migration:

BUNDLE_PROVENANCE.md identifies source commit 6ab6032fe6f0f917be68982adba416d2ded707e3, source branch origin/feat/a1-3-public-adoption-doc-correction, and included source root llm-last-mile/runtime-refactor/.

bundle-manifest.json separately records head_sha 7fdacfd12590baec3c258e909b65caadee4212f8. Those are two distinct provenance fields from two bundle records; the migration should preserve both without trying to infer or reconcile their relationship.

worktree.patch is empty.

The manifest names no excluded paths, but the authority boundary is still the included source root. Material referenced outside that root remains unavailable and must not be inferred.

The principal decomposition targets are 00-README.md through 05-debug-regression-ledger.md. 06-review-finding-inventory.md, the focused decision/packet directories, and review-control/ already embody important parts of the desired smaller-document model and should initially remain path-stable.

This is not primarily a file-size split. The desired boundary is one clearly typed semantic authority per document, with separate navigation projections.

The first phase should preserve every substantive statement somewhere in the canonical tree. No text should be discarded merely because it is old, superseded, repetitive, or no longer active.

1. Document taxonomy and canonical-authority rules
1.1 Document taxonomy

Document kind and document status must remain separate. For example, a held packet is still a packet, not merely “history”; a superseded scheduling decision is still a decision and may remain an important authority fence.

Kind	Canonical responsibility	Must not do
Foundation	Stable vocabulary, semantic status model, normative conventions, reading rules, and cross-cutting definitions.	Select active work or record runtime proof.
Architecture	Target ownership, authority relationships, invariants, and durable model boundaries.	Become a chronological packet ledger or current schedule.
Inventory	Enumerate bounded observed items with stable classifications and ownership fields.	Silently promote observations into contracts or decisions.
Seam	Own the current classification, target owner, transition boundary, evidence requirement, and unresolved state of one seam or inseparable seam family.	Claim completion from neighboring primitives or component-only evidence.
Slice	Define one independently reviewable scope, dependencies, non-goals, outputs, and completion conditions.	Reorder the global schedule unless a decision authorizes that order.
Task	Define a bounded authorized action inside a slice or packet, including exact subject and exclusions.	Expand beyond its owning packet or invent successor authority.
Packet	Assemble one selected body of work: decision, tasks, seams, contracts, gates, evidence, and historical context.	Duplicate the normative bodies of all those documents.
Contract	Own a named record, protocol, state machine, validation rule, or acceptance contract.	State that a gate passed or that evidence exists.
Gate	Own admission, promotion, exit, stop, authority-required, and completion criteria.	Define the underlying contract more broadly than necessary.
Decision	Own a dated scheduling, authority, platform, or supersession choice within an explicit scope.	Rewrite the history or imply authority outside the stated scope.
Evidence	Record observations, attempts, proof results, limitations, failures, and closeout facts.	Create policy, authorize implementation, or replace a gate.
Receipt / review record	Preserve exact machine-auditable review-cycle or receipt bytes and their stated interpretation.	Replace product proof or packet authority.
Historical record	Preserve a prior decision, packet, checkpoint, failure, or composite chronology exactly enough to remain independently interpretable.	Be treated as automatically executable merely because it remains in the active tree.
Projection / index	Present current navigation, status, ownership, or packet assembly derived from canonical documents.	Contain unique normative prose or become an alternate authority source.
Migration record	Bind source spans, old anchors, destination documents, hashes, authority transfer, and rollback state.	Alter the semantics of the migrated material.

A section that cannot yet be separated without changing meaning should be retained as a compound canonical document. Smaller is not safer when two contract versions, a decision and its qualification, or a table and its classification rules are semantically inseparable.

1.2 Canonical-authority rules
Rule 1 — one canonical owner

Every named concept, packet, decision, gate, contract, inventory, or evidence record must have exactly one canonical owner at a given revision.

A projection may repeat identifiers, status tokens, dates, and links, but it may not contain unique normative text.

Rule 2 — authority is assigned explicitly, not inferred from recency

Do not select authority merely because a document is newer, appears later in a root file, or uses stronger language. Record an explicit authority entry containing:

stable ID;

document kind;

canonical path and anchor;

authority scope;

current status;

explicit predecessor or supersession relationship;

projection locations; and

original source spans.

Rule 3 — supersession is scoped

Preserve qualifiers such as:

“for active scheduling only”;

“held historical fence”;

“documentation-only decision”;

“non-executable historical selection”; and

“does not authorize the successor.”

A scheduling decision can supersede an earlier ordering without superseding its engineering history, proof facts, platform boundaries, or threat-model content.

Rule 4 — projections are mechanically bounded

A projection may contain:

an existing stable ID;

an exact status token;

a date;

a one-line scope label;

a canonical link; and

an explicitly marked exact quotation when unavoidable.

It should not introduce new must, shall, next, complete, blocked, authorized, or supersedes claims. Those belong in the canonical decision, gate, contract, or packet.

Rule 5 — canonical transfer is atomic

An extraction slice should land together:

the destination containing the complete substantive source span;

the old heading or explicit compatibility anchor;

a pointer from the old location to the new canonical location;

every necessary navigation update;

the authority-registry update; and

the extraction-ledger record.

There must never be an intermediate landed state where neither location is clearly canonical or where part of a contract is authoritative in each location.

Rule 6 — exact preservation precedes editorial improvement

During first-phase extraction:

do not paraphrase;

do not reflow paragraphs or tables;

do not normalize terminology;

do not renumber contracts;

do not combine similar histories;

do not remove repetitions;

do not rewrite old paths to make them look current;

do not run a broad formatter over extracted text.

New metadata and navigation text may surround a preserved span, but the migrated substantive span should remain exact whenever structurally possible.

Rule 7 — history remains typed

Do not send everything inactive into a generic archive. Preserve the type:

held packet;

superseded scheduling decision;

closed gate;

bounded-stop review record;

failed attempt;

archived platform architecture;

completed evidence record.

history/ should be used only for cross-cutting composite snapshots or legacy projections that cannot be assigned safely to a more precise kind.

Rule 8 — evidence never grants authority

The separation already stated in review-control/README.md should become a system-wide rule:

Process evidence and receipts may prove what was reviewed or observed; they do not replace product proof, gate satisfaction, or packet authority.

The same applies in reverse: a packet or decision cannot claim evidence that the evidence record does not contain.

Rule 9 — preserve location-locked exceptions

Three existing constraints should not be silently overridden by this decomposition:

02-seam-crosswalk.md says the A0 authority-leak inventory is populated in that file and explicitly says not to create a separate control-pack file. Leave that inventory canonical in 02 during the first migration. Moving it requires a separate, explicit documentation-location authority amendment.

06-review-finding-inventory.md identifies itself as the single durable inventory for valid non-blocking P3 and P4 findings. Keep it singular, canonical, and path-stable.

Existing review-control JSON, SHA-256, validator, tests, receipts, and review records should remain path- and byte-stable. First-phase work may index them but should not reorganize them.

Rule 10 — preserve compound contract units

Keep together during the first split:

versioned contract pairs whose compatibility relationship is defined jointly;

a gate and an immediately attached exception or stop rule;

ordered fields whose order is meaningful;

a fenced example and the prose that defines its normative status;

a decision and its explicit scope qualification;

one chronological attempt and its result;

table headers, field definitions, and rows when a row is not independently interpretable.

Rule 11 — conflict stops extraction

When two included passages appear to claim canonical authority for the same thing and the included text does not resolve the conflict:

preserve both;

record an authority-classification-blocker;

do not choose the easier location;

do not rewrite either passage;

land no authority transfer for that subject until the conflict receives a separate docs-only decision.

Rule 12 — root paths remain compatibility surfaces

00-README.md through 05-debug-regression-ledger.md should progressively become indexes and compatibility-anchor surfaces, but they should not disappear during this phase.

Old anchors remain supported through the entire first migration. Removing those stubs is later selective minimization and needs separate authority.

2. Extraction sequence of independently landable docs-only slices
Pre-landing baseline — no repository change

Before the first extraction, freeze a read-only baseline containing:

both bundle provenance identifiers;

file inventory and byte hashes;

all Markdown links and anchors;

all links leaving the included root;

source heading tree;

current authority assignments;

explicit supersession edges;

path-frozen review subjects;

current status tokens;

exact source spans selected for the first slice.

This is preparation, not a landed migration slice.

D1 — semantic-status pilot plus minimum migration ledger

Extract only the ## Semantic status labels section from 00-README.md.

This is the smallest substantive pilot and is detailed in section 6 below.

D2 — stable foundations, one section per landing

After D1 proves the mechanics, extract small stable foundation sections individually:

authority vocabulary;

reading and update rules;

context-assembly rules;

stable document-status terminology;

projection terminology;

canonical-location and compatibility-anchor policy, while retaining the canonical root declaration in 00.

Each section should be its own revertible slice unless two sections explicitly define each other.

D3 — review and remediation governance

Extract from 04-contracts-and-gates.md:

normative conventions;

development review and remediation contract;

finding-priority interpretation;

closure-review requirements;

bounded-stop handling;

relationships among review evidence, product proof, and packet authority.

Do not move 06-review-finding-inventory.md or change review-control/ in this slice.

The old 04 headings remain as compatibility anchors.

D4 — current-decision consolidation

Stabilize current scheduling authority before decomposing historical packet material.

Canonical owners should remain path-stable:

linux-first-runtime-resumption/DECISION.md for global Linux-first scheduling;

linux-first-runtime-resumption/A1.3-P1-LINUX-FIRST-ATOMIC-PUBLIC-ADOPTION-PACKET.md for the active packet;

macos-dev-parity/DECISION.md for the separate macOS platform lane.

Then, in one atomic docs-only slice:

create a decision index;

create a visibly non-authoritative current-state projection;

classify corresponding “current scheduling” sections in 00, 02, 03, 04, and 05 as projections;

preserve prior A1.3 and A1.3-P0 material as held, non-executable historical fences;

preserve the macOS decision’s limited “for active scheduling only” supersession language;

extract any current AUTHORITY_REQUIRED:* gate bodies into gate documents where they can be moved without changing their meaning.

This must be atomic because a partial update could leave conflicting live schedules in the root documents.

D5 — completed vertical packet pilot: R2-4

Use completed R2-4 as the first cross-document or “vertical” extraction.

Move its complete material from the root documents into typed owners:

architecture disposition from 01;

seam/crosswalk disposition from 02;

slice and task description from 03;

terminal gate disposition from 04;

evidence and regression material from 05;

a packet index that assembles those canonical links.

Keep the existing R2-4 evidence, subject hash, review-cycle record, review history, and lens records under review-control/ unchanged.

R2-4 is a suitable pilot because it is terminal, bounded, claims no new architecture or seam promotion, and already has a clear evidence boundary.

D6 — packet and task families, one family per landing

Repeat the vertical pattern one packet or tightly bound packet family at a time. Prefer reverse active chronology so present authority becomes clear before deep historical material is moved:

A1.3-P1 and duplicate root projections;

held A1.3-P0 and A1.3 fences;

macOS parity and its duplicated root projections;

R3 macOS evidence-recovery planning records;

R3 manifest/platform/lifecycle/closeout families;

R2-3;

renewed R2-2 closeout and its subpackets;

F and RP families;

B1, B2.1, B3.1, C1, and related packet families;

A1.2 and earlier packet histories.

Each landing should include all material for that packet from every affected root document. Do not leave architecture canonical in 01, gates canonical in 04, and evidence canonical in a newly created packet file without an explicit authority map.

Existing focused directories remain in place. The new packet index should link to them rather than relocating them.

D7 — shared target architecture and invariants

Once packet-specific architecture appendices have been removed from 01-target-architecture.md, extract the stable architecture:

executive target decision;

authority map;

each numbered invariant or one inseparable invariant family;

shared ownership boundaries;

stable review questions.

Preserve invariant numbering exactly. Do not renumber merely because the invariants become separate files.

A useful granularity is one invariant per file only when it remains independently interpretable. Otherwise retain a small invariant family.

D8 — seam families and inventories

Decompose 02-seam-crosswalk.md by seam or tightly bound seam family:

host/session authority;

persistence and compatibility projection;

dispatch and episode transport;

obligations;

retained-worker runtime;

world-service/runtime-event transport;

UAA/provider realization;

configuration and gateway adoption;

platform/lifecycle boundaries.

For each seam document, retain:

exact seam name;

observed implementation surface;

semantic status label;

current classification;

target owner;

transition rule;

evidence required for promotion;

explicit non-promotions and exclusions.

Shared table-column meanings should have one canonical family definition. A copied header in an index must be marked as a projection.

Exceptions:

leave the A0 authority-leak inventory in 02;

do not split a row whose meaning depends on adjacent rows until the family can be moved intact;

never promote a seam simply because the definition moved.

D9 — slice, track, and task decomposition

Decompose 03-phase-slice-map.md into:

one shared sequencing/dependency document;

one track index per major track;

one document per independently reviewable slice;

task documents for separately authorized substeps;

packet links for slices selected into execution packets.

Preserve distinctions such as:

independent slice;

checkpoint within a parent slice;

held slice;

completed slice;

bounded stop;

undispatched successor;

authority-required future work.

Do not promote checkpoints into independent slices merely to obtain uniform files.

The current schedule remains a projection of decision documents, not an authority owned by slices/.

D10 — contracts and gates

Decompose 04-contracts-and-gates.md after packet-local gate material and review governance have moved.

Recommended order:

shared canonical encoding and identity conventions;

home/bootstrap/root contracts;

object-reference, hash-input, and commitment contracts;

state-root and publication/write protocols;

durable session-authority contracts;

transition intent and episode contracts;

event-carrier and retained-worker-envelope contracts;

acceptance and receipt contracts;

manifest and policy contracts;

execution-envelope and secret-handoff contracts;

cancellation, supervision, and reconciliation contracts;

differential, monotonicity, and promotion contracts;

packet-local gates;

authority-required gates.

Each named contract receives one canonical document. Keep version pairs together when their transition or compatibility is part of the contract.

Do not split very large compound contracts, such as a manifest lifecycle contract, merely because they contain many headings. First move the whole compound unit; finer decomposition can follow only after its internal joins are explicitly mapped.

Fenced code, ordered field sets, literal identifiers, and validation sequences must remain exact.

D11 — evidence and regression records

Decompose 05-debug-regression-ledger.md into typed evidence:

canonical issue ledger;

packet-specific evidence;

failed-attempt records;

bounded-stop records;

platform-evidence limitations;

baseline behavior;

regression obligations;

smoke scenarios;

closeout evidence;

current-state projection.

Evidence should be organized by stable evidence or issue ID and by packet, not merely by date.

The four repo-relative links that leave the included root must remain lexically unchanged and opaque:

../../substrate-runtime-refactor-directive-revised.md;

../../RUN_WORLD_TASK_DEBUG_CANONICAL.md;

../../CONTINUE_WORLD_WORKER_BLOCKING_DEVIATION_DEBUG.md;

../../CODEX_WORLD_DISPATCH_GAP_WRITEUP.md.

Do not infer their contents, change their targets, or claim that they resolve from the bounded snapshot.

review-control/ remains the canonical receipt/review-record store. evidence/receipts/README.md should only index it and explain its authority boundary.

D12 — root compatibility-index cutover

Only after every substantive span in 00–05 appears in the extraction ledger:

make 00-README.md the stable human entry point;

make 01–05 typed compatibility indexes;

retain every legacy heading or explicit anchor;

provide canonical links at each former location;

provide by-ID, by-kind, by-packet, and current-state navigation;

leave 06-review-finding-inventory.md unchanged and canonical;

retain all focused directories and review-control/.

This concludes decomposition, not minimization. Removing compatibility stubs, merging repeated historical statements, shortening ledgers, or deleting superseded material belongs to a later separately reviewed phase.

3. Proposed directory, index, and navigation structure
llm-last-mile/runtime-refactor/
├── 00-README.md
├── 01-target-architecture.md
├── 02-seam-crosswalk.md
├── 03-phase-slice-map.md
├── 04-contracts-and-gates.md
├── 05-debug-regression-ledger.md
├── 06-review-finding-inventory.md
│
├── index/
│   ├── README.md
│   ├── current.md
│   ├── by-id.md
│   ├── by-kind.md
│   └── by-packet.md
│
├── foundations/
│   ├── README.md
│   ├── semantic-status-labels.md
│   ├── authority-vocabulary.md
│   ├── reading-and-update-rules.md
│   └── normative-conventions.md
│
├── architecture/
│   ├── README.md
│   ├── executive-target.md
│   ├── authority-map.md
│   └── invariants/
│       ├── README.md
│       └── <existing-number>-<stable-slug>.md
│
├── inventories/
│   ├── README.md
│   └── <inventory-id>.md
│
├── seams/
Navigation behavior
00-README.md

Remain the canonical entry point and answer:

What is this control plane?

What is active?

What is held or deferred?

Which documents must be loaded for a packet?

Where are architecture, seams, slices, contracts, gates, decisions, and evidence?

What are the compatibility root files?

It should not become a second owner for the content it indexes.

index/current.md

A clearly labeled non-authoritative projection containing:

controlling global decision;

active packet;

lane-local active decisions;

held packets;

authority-required gates;

undispatched successors;

latest relevant evidence or bounded stop.

Every row links to a canonical owner.

index/by-id.md

A mechanical registry for:

packet and task IDs;

seam IDs;

contract names and versions;

gate IDs;

evidence and issue IDs;

review-finding IDs;

decision IDs or stable titles.

Packet indexes

packets/<packet-id>/README.md should provide a selective-load order:

controlling decision;

packet scope;

tasks;

affected architecture and seams;

slice/dependencies;

contracts;

gates;

evidence and receipts;

historical predecessors;

explicit exclusions.

The packet index should assemble context through links rather than repeat the bodies.

Metadata on new canonical documents

Each new canonical document should carry a small non-substantive header such as:

Kind:
Stable ID:
Canonical for:
Status:
Authority scope:
Source span:
Supersedes:
Superseded by:
Projection consumers:

This metadata does not replace the original body and should not reinterpret it.

4. Preservation rules
4.1 Link and anchor preservation

Record every original heading anchor before extraction.

Keep the old heading in its root document or add an explicit compatibility anchor with the same identifier.

Place the canonical pointer immediately beneath the old anchor.

Update relative links only as needed to preserve the same destination.

Preserve link display text unless it would become factually false; any such case is a review blocker rather than an automatic rewrite.

Do not alter external URLs.

Preserve out-of-bundle relative links exactly and treat them as opaque.

Do not declare an old anchor removable merely because there are no in-bundle inbound links; external consumers are outside the snapshot and unknown.

Validate fragment identifiers as well as destination files.

Historical path lists remain historical text even when those paths later lead to compatibility indexes.

4.2 Provenance preservation

For every extraction, record:

bundle provenance source commit;

manifest head_sha;

original path;

original heading;

original anchor;

baseline line range as an aid, not as sole identity;

SHA-256 of the exact extracted source bytes;

destination path and anchor;

extraction slice ID;

authority-transfer mode;

old-anchor disposition;

rollback source.

Do not collapse the two bundle revision identifiers into one “correct” commit.

4.3 Historical-record preservation

Preserve exactly:

dates;

commit and tree identifiers;

SHA-256 values;

statuses;

failed and incomplete attempts;

bounded-stop conclusions;

evidence limitations;

held or archived classifications;

“unstarted” and “undispatched” statements;

supersession scope;

exact old path and subject lists;

predecessor ordering as stated at that historical checkpoint.

When a historical statement no longer describes the current state, add a canonical current projection elsewhere. Do not edit the historical statement to make it current.

In particular, path sets frozen by r3-mac-evidence-recovery/TASKS.md or review-control subject records remain exact historical subjects. New navigation may accompany them, but the old subject must not be rewritten.

4.4 Active-decision preservation

A decision migration must preserve separately:

the decision that was made;

its effective date;

its exact scope;

its authority gate;

its selected packet;

what it supersedes;

what it explicitly does not supersede;

parallel-lane relationships;

held and deferred successors;

later corrective replans;

whether the record is active, historical, or documentation-only.

Do not derive current scheduling by sorting decisions by date. Follow explicit controlling and supersession language.

4.5 Contract and gate preservation

Preserve stable contract names and version suffixes.

Preserve field order where the source presents an ordered contract.

Preserve literals, schemas, hashes, commands, code blocks, and validation sequences.

Preserve negative requirements and exclusions alongside positive requirements.

Do not separate a gate from the exception that prevents it from being interpreted broadly.

Do not make a gate “cleaner” by removing old failed or blocked states.

Do not convert evidence of one condition into satisfaction of a broader gate.

4.6 Evidence and receipt preservation

Evidence records are append-only in meaning.

A failed attempt remains visible after later success.

A bounded stop remains a bounded stop.

Evidence limitations remain adjacent to the result they qualify.

Receipt and review-cycle files under review-control/ remain unchanged.

A new receipt index links to exact artifacts and their existing interpretation; it does not restate them as broader proof.

Existing external-artifact disclaimers remain intact.

4.7 Formatting preservation

During first-phase extraction:

no global Markdown formatter;

no table normalization;

no heading renumbering;

no wrapping cleanup;

no code-fence language changes;

no capitalization normalization;

no “obvious typo” correction inside preserved text;

no removal of apparent redundancy.

Formatting improvements require a later, separately bounded maintenance slice.

4.8 Complete-span accounting

Every substantive line removed from a root document must be represented in exactly one of:

a canonical destination;

an exact typed historical record;

an explicitly marked compatibility projection; or

a recorded authority-classification blocker that prevented extraction.

The root-index cutover cannot occur while the extraction ledger contains an unmapped substantive span.

5. Validation, review, and rollback gates
5.1 Gates common to every slice
Scope gate

The diff must be limited to documentation and migration-index paths under llm-last-mile/runtime-refactor/.

No runtime, test, installer, Lima, platform, native-operation, JSON receipt, SHA-256 receipt, or validator change belongs in these slices.

Preservation gate

For every moved span:

source-byte hash matches the recorded baseline;

destination contains the full span;

no paragraph, row, list item, fenced block, qualifier, or attached note is omitted;

no substantive content is present only in the deleted source location;

no unreviewed semantic rewrite appears.

Authority gate

exactly one canonical owner after landing;

every duplicate is marked projection, compatibility stub, or history;

no projection contains unique normative language;

no supersession scope changes;

no held packet becomes executable;

no gate status changes.

Link gate

all in-root relative links still resolve;

fragment links resolve;

legacy anchors remain;

external URLs are unchanged;

the four out-of-bundle relative links retain their exact normalized targets;

path-frozen historical subjects are unchanged.

Provenance gate

authority registry updated;

extraction ledger updated;

old and new paths recorded;

exact span hash recorded;

both bundle revision identifiers retained;

rollback recipe recorded.

Historical gate

dates, IDs, hashes, status words, failure outcomes, and evidence limitations remain exact;

active and historical statements are not blended;

bounded-stop records remain bounded stops;

archived records are not converted into “obsolete” records.

Review gate

Apply the existing review/remediation discipline already defined by the control plane:

freeze the docs-only subject;

perform discovery review;

classify findings by the existing priority rules;

remediate blocking findings within the authorized docs slice only;

run fresh closure review;

record valid non-blocking P3/P4 findings in 06 only when already properly dispositioned;

stop boundedly on scope expansion or unresolved authority ambiguity.

No new review validator or test implementation is needed for this roadmap.

Rollback gate

Each slice must be revertible without reconstructing text from memory:

preserve the exact pre-slice source bytes;

make one atomic commit per semantic slice;

identify new files to remove;

identify old files and spans to restore;

identify index and registry rows to revert;

prohibit partial rollback of a packet family or current-decision consolidation.

After later slices depend on an earlier destination, rollback proceeds in reverse dependency order.

5.2 Slice-specific gates
Slice	Additional validation gate	Rollback unit
D1 status pilot	Exact status-section hash; all five labels and promotion rule present; old anchor resolves.	Restore the one 00 span and remove the destination and ledger entry.
D2 foundations	Every definition has one owner; no term’s meaning changes; current scheduling prose excluded.	One foundation section at a time.
D3 review governance	P1/P2/P3/P4 behavior, bounded-stop handling, and product-proof distinction remain exact; 06 and review-control/ unchanged.	Entire governance extraction or one independently complete subsection.
D4 current decisions	Controlling decision, active packet, held packets, parallel macOS lane, and deferred work match across every projection.	All affected current projections and decision indexes atomically.
D5 R2-4 pilot	Architecture, seam, slice, gate, and evidence components are all accounted for; no new R2-4 claim; review artifacts byte-stable.	Entire R2-4 vertical family.
D6 packet families	No packet-local section remains uniquely canonical in a root file; packet exclusions and predecessor relationships preserved.	One complete packet or inseparable packet family.
D7 architecture	Invariant numbering and cross-references unchanged; no packet history misclassified as an invariant.	One invariant or inseparable invariant family.
D8 seams	Every table field retained; semantic status unchanged; no seam promotion; A0 remains in 02.	One seam or seam family.
D9 slices/tasks	Dependency graph and exact statuses preserved; checkpoints not promoted; no schedule decision introduced.	One slice/task family plus its indexes.
D10 contracts/gates	Names, versions, ordered fields, code blocks, negative requirements, and exceptions exact; old contract anchors resolve.	One contract/gate unit or required version family.
D11 evidence	Attempt chronology and limitations exact; receipts unchanged; issue IDs stable; no evidence promoted into authority.	One evidence family or packet ledger.
D12 root cutover	Extraction ledger shows complete span coverage; all old anchors work; all canonical destinations are indexed.	Entire root compatibility-index cutover.
6. Smallest safe first implementation slice
Recommended slice: extract only the semantic status model

The smallest safe substantive migration is the section:

00-README.md
## Semantic status labels

In this snapshot, the exact source span is lines 143–156, including the heading, table, promotion rule, and trailing separator newline. Its SHA-256 is:

d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736

It defines:

ContractCorrectAndProven;

UsefulFootholdButWrongBoundary;

DefensiveScaffoldingOnly;

MislandedWrongModel;

MissingSeam; and

the rule preventing component-only evidence from promoting a bypassed production seam.

These labels are used throughout the crosswalk, but no in-bundle Markdown link targets the section’s current heading. The old anchor should nevertheless be retained because out-of-bundle consumers are unavailable.

Exact first-slice path set
ADD    llm-last-mile/runtime-refactor/foundations/semantic-status-labels.md
ADD    llm-last-mile/runtime-refactor/migration/extraction-ledger.md
EDIT   llm-last-mile/runtime-refactor/00-README.md

No other path should change.

Destination content

foundations/semantic-status-labels.md should contain:

a small non-substantive metadata header identifying it as the canonical foundation;

the exact original section bytes, unchanged;

a source-provenance reference to 00-README.md and the baseline section hash.

The five labels and promotion paragraph should not be rewritten into a more polished standalone explanation.

Source compatibility behavior

In 00-README.md:

retain the exact ## Semantic status labels heading so the old generated anchor survives;

replace the moved body with a direct statement that canonical authority is now in foundations/semantic-status-labels.md;

add the new document to the control-pack map or context-assembly navigation;

make no other substantive edit.

The compatibility statement should not restate or reinterpret the labels.

Extraction-ledger row

The initial ledger entry should record at least:

Field	Value
Slice	D1
Source	00-README.md
Source heading	Semantic status labels
Legacy anchor	semantic-status-labels
Baseline span	lines 143–156
Span SHA-256	d9179c4258ed8d17f4a2127729d5b537fad56c284ea05d6e0488eb9f66853736
Destination	foundations/semantic-status-labels.md
Kind	foundation
Authority transfer	canonical destination; source compatibility anchor
Substantive rewrite	none
Rollback	restore exact source span; remove destination and ledger entry
First-slice acceptance gate

The slice lands only when all of the following are true:

the destination’s preserved span hashes to the recorded SHA-256;

all five labels are byte-for-byte present;

the full promotion rule is present;

the old 00-README.md#semantic-status-labels anchor resolves;

00-README.md links directly to the new canonical document;

no other root section changes;

all existing in-root links still resolve;

all external and out-of-bundle targets remain unchanged;

the diff contains only the three authorized Markdown paths;

review finds no semantic change or competing authority.

First-slice rollback

Rollback is entirely mechanical:

restore the exact original section bytes in 00-README.md;

remove foundations/semantic-status-labels.md;

remove the D1 extraction-ledger entry, or remove the ledger file if D1 created it and it has no later entries;

verify the restored source-span hash;

verify the original link baseline.

This pilot avoids active scheduling, packet authority, contract field layouts, platform decisions, runtime evidence, review receipts, A0’s location-locked inventory, and the singular P3/P4 inventory. It tests the essential migration mechanism—exact extraction, canonical transfer, old-anchor continuity, provenance, review, and rollback—using the smallest self-contained substantive section in the control pack.

Sources
ChatGPT can make mistakes. Check important info.

Pro
