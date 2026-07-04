# NOTE-64: Stop World Worker Authority Closeout Proof

Status: slice-local closeout evidence for review

This note records the exact Linux-first proof paths exercised for Slice `64` closeout. It does not redesign any public stop surface, redefine transport delivery as success, claim non-Linux semantic parity, or introduce a typed stop-outcome taxonomy.

## Linux-First Proof Paths Exercised

### Positive proof path

Command:

```bash
cargo test -p shell dispatch_contract_stop_world_worker_spec64_recovery_harness_drives_the_real_refreshed_transport_failure_branch -- --nocapture
```

Result:

- `ok`: passed
- the stop episode emitted exactly three retry events: `InitialAttempt`, `RetryWaitStarted`, and `RetryAttempt`
- both attempts stayed on the same exact private stop socket path
- the exact retained worker closed out as `Stopped`
- the stop summary contained `detached durable closeout`

### Negative proof path

Command:

```bash
cargo test -p shell --test repl_world_first_routing_v1 c3_internal_toolbox_stop_world_worker_treats_disappearing_private_stop_delivery_as_fail_closed -- --nocapture
```

Result:

- `ok`: passed
- caller-visible result stayed non-success with `ok = false`
- the error preserved `missing_transport:` plus `owner_unreachable: recovery_failed:`
- the error also preserved `durable stop closeout was not observed`
- the retained worker remained authoritative-live in `ready` state with no persisted termination reason
- no downstream cancel delivery was fabricated

## Review Notes

1. Linux-first here is proof posture only.
2. Non-Linux remains explicitly fail-closed in this slice.
3. The landed caller-visible contract keeps the existing shape and uses stable wording for `missing_transport`, `refused_transport`, `stale_authority`, and `recovery_failed`.
