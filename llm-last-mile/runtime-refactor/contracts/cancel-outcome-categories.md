**Kind:** contract
**Status:** canonical
**Canonical for:** complete extracted `CancelWorldWorkOutcomeV1` enum and rules 1–6 covering exact identity and world-binding resolution, `NoActiveCancelableWork` versus `OwnerUnreachable`, pending closeout, terminal monotonicity, and stop-versus-cancel non-aliasing
**Source provenance:** extracted byte-for-byte from [`../04-contracts-and-gates.md#10-cancel-outcome-categories`](../04-contracts-and-gates.md#10-cancel-outcome-categories), baseline lines 119–143; the exact 1179-byte source body is preserved between the boundary markers below
**Baseline span SHA-256:** `e8f373c9e7cbf16d286ff8f9a9146306644e430e3d630b7078ec7501b72ed4b3`

<!-- exact-extracted-body:start -->
## 10. Cancel outcome categories

```rust
enum CancelWorldWorkOutcomeV1 {
    CancelledViaLiveTransport { active_run_id, terminal_ref },
    CancelAcceptedPendingCloseout { active_run_id, cancel_request_id },
    AlreadyTerminal { active_run_id, terminal_ref },
    NoActiveCancelableWork { worker_or_task_ref },
    OwnerUnreachable { active_run_id, durable_state, retryability },
    InvalidTarget { reason },
    WorldBindingMismatch { expected, actual },
    AmbiguousTarget { candidates },
    PolicyDenied { denial_code, explanation },
}
```

Rules:

1. Exact identity and world binding resolve before transport use.
2. `NoActiveCancelableWork` means valid routing context but no accepted non-terminal cancelable receipt.
3. `OwnerUnreachable` means a valid active receipt exists but the live cancellation route is unavailable and durable policy cannot yet declare closeout.
4. `CancelAcceptedPendingCloseout` is not terminal success; inspect remains able to observe eventual closeout.
5. Repeated cancel after terminal returns `AlreadyTerminal`; it never regresses the receipt.
6. Stop targets worker lifecycle. Cancel targets one active task/turn. They are not aliases.

<!-- exact-extracted-body:end -->
