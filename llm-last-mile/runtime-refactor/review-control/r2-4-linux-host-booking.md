# R2-4 dedicated Linux host booking and restoration contract

## Status and boundary

**Conditional assignment; not start-ready. R2-4 remains unstarted.**

The future R2-4 dedicated supported-Linux proof window is assigned to the saved Codex project host
`remote-ssh-discovered:spenser-linux-codex`, live hostname `spenser-linux`, for checkout
`/home/spenser/__Active_code/substrate`. Codex project discovery maps that exact path to project ID
`2ccb802f-301c-4af4-9bd5-51d22808f0a2` on that host. Read-only inspection on 2026-07-31 observed:

- Manjaro Linux, x86-64, kernel `6.16.8-1-MANJARO`, systemd host-native;
- operator account `spenser` (`uid=1000`) in the `substrate` group;
- `substrate-world-service.socket` and `substrate-world-service.service` enabled and active;
- `/run/substrate.sock` as `root:substrate 0660` with a named `spenser:rw-` ACL;
- `substrate`, Codex CLI, `jq`, `nft`, `bwrap`, and `systemctl` present; and
- the checkout on local ext4 storage.

Those observations establish an exact future host and existing prerequisites only. They are not a
dedicated-host R2-4 product run, do not prove default/custom A or conflicting B behavior, and do not
authorize provisioning, installation, uninstall, service mutation, privileged smoke, or the R2-4
matrix. Read-only task inventory found no active R2-4 execution.

## Unsatisfied booking gates

The host is conditionally assigned but must be treated as unavailable until all of these are
recorded by the future authorized R2-4 task:

1. The human operator confirms an exclusive maintenance window for this physical host and accepts
   responsibility for restoration.
2. The operator confirms the exact sudo posture for the window. At this inspection, `sudo -n true`
   exited `1` with `sudo: a password is required`; noninteractive sudo is unavailable. A future
   interactive credential path must be present, or a noninteractive runner must stop before any
   mutation.
3. The operator reviews and accepts the complete pre-state snapshot below before the first mutating
   command.
4. The future task binds its exact commit/tree, R2-4 authority, evidence directory, A/B roots, and
   restoration owner before execution.

No current document may translate this conditional assignment into `READY`, `STARTED`, or product
proof. Failure of any gate is `Linux product host is unavailable`, the existing R2-4 stop condition.

## Required baseline snapshot

Before any future R2-4 mutation, capture machine-readable or byte-hashed pre-state for:

- host identity, OS/kernel/architecture, filesystem/mount identity, clock, current account/groups,
  and exact local/upstream/remote commit/tree/ref;
- sudo interactive/noninteractive posture without recording credentials;
- installed Substrate/Codex/tool versions and hashes of Substrate-owned binaries, scripts, payloads,
  generated configuration, install-state, shims, and manager hooks;
- existence, bytes/hashes, owner, group, mode, ACL, and symlink/type identity for every selected
  default/custom A path and every deliberately conflicting B path;
- systemd unit/drop-in bytes, enablement and active state, main PID identity, socket identity and
  ACL, and journal cursor/time boundary;
- `/var/lib/substrate`, `/run/substrate`, `/run/substrate.sock`, world-deps, gateway runtime, trace,
  policy/config/inventory, runtime-family, and Codex projection metadata required by existing gates;
- `substrate` group membership, the operator's login-session group view, named-user ACL bridges,
  and linger state; and
- relevant cgroup, namespace, nftables, process, socket, temporary-root, and mount state, plus
  unrelated sentinels used to prove preservation.

For every mutable pre-existing file, symlink, directory entry, unit/drop-in, generated artifact,
install-state file, shim, hook, or configuration object, a digest is evidence of identity but is
not a restore source. Before mutation, create an immutable recoverable copy or identify an
independently verified immutable restore source. Record its exact source and destination identity,
content digest, ownership/type/metadata coverage, access method, retention boundary, and a
non-mutating readability/integrity check. For state that cannot be copied as bytes, record the exact
reconstruction command or API, its complete inputs, and a read-only proof that those inputs can
recreate the accepted state. The restoration owner must accept this restore-source inventory before
the first mutation. Missing, inaccessible, mutable, or unverified restore material is a hard stop.

Record existence and commitments for credential-bearing state, never secret contents. A snapshot
gap, unsafe path, unexplained pre-existing artifact, or inability to preserve an unrelated sentinel
stops before mutation.

## Restoration owner and contract

The human operator controlling account `spenser` and the interactive sudo credential is the
restoration owner. An agent may execute only a separately authorized R2-4 task inside the accepted
window; it cannot self-approve the baseline or restoration.

After every attempted future run, including failure or interruption, the task must:

1. stop further matrix work and retain the exact evidence and failure boundary;
2. restore only paths and host state whose pre-state and ownership were captured, using exact
   targets and the accepted immutable restore sources rather than wildcard or broad recursive
   cleanup;
3. restore byte content, ownership, mode, ACL, symlink/type identity, group membership, linger,
   unit/drop-in bytes, enablement/active state, sockets, processes, mounts, namespaces, cgroups, and
   nftables to the accepted baseline;
4. prove unrelated sentinels and pre-existing artifacts are unchanged;
5. compare a complete post-state snapshot to the baseline and record every intentional or
   unexplained difference; and
6. obtain restoration-owner sign-off before releasing the host or claiming R2-4 completion.

Operational restoration of the booked proof host is not evidence that Substrate implements R3
cleanup/convergence. If exact restoration would require new product behavior, broad deletion,
ownership inference, or repair of an R2 defect, stop and return the issue to its owning packet. R3
alone owns product cleanup/convergence implementation.

## Additional stop conditions

Stop without starting or continuing R2-4 if the exact host mapping changes; another mutating task
uses the host; sudo posture is unresolved; service/world prerequisites are missing; baseline or
immutable restore-source identity, access, retention, or integrity is incomplete; the requested
diff needs a production file; PI-050 or any R2 row
lacks proof; the three classified non-R2-3 world-deps/report failures would be relabeled; the four
accepted P3 inventory entries would be changed without new evidence; canonical-runner eligibility
would be claimed for deferred `R2-3ZP3`; or cleanup/convergence leaks into R2.

## External process-reference limitation

The installed external skill
`/home/spenser/.agents/skills/using-agent-skills/SKILL.md` references
`references/definition-of-done.md`, but that relative file is absent. A search of the available
`/home/spenser/.agents/skills` and `/home/spenser/.codex/skills` roots found no canonical shared
copy. This repository does not own that skill installation, so this correction does not edit it or
invent a replacement path. The explicit repository and packet verification gates remain the
completion authority; repair of the external skill package is a separate owner action.
