**Kind:** foundation
**Status:** canonical
**Canonical for:** reading and update rules

## Reading and update rules

1. Treat this pack as canonical for refactor intent, slice boundaries, contracts, and gates.
2. Treat live code plus fresh runtime evidence as canonical for current artifact truth.
3. If code truth changes, update the affected crosswalk row and regression row in the same implementation PR.
4. Keep a seam below `ContractCorrectAndProven` until its actual production path is proven.
5. Do not use helper/PID/socket liveness as authority in new contracts.
6. Do not create one crate per named seam. A seam may be a module, facade, trait, type, or extracted function set.
7. Keep slice boundaries hard. Adjacent sibling seams stay in context but are not implicit scope.
8. Preserve resolved debug behavior while replacing the model that produced it.

**Source provenance:** extracted byte-for-byte from [`00-README.md#reading-and-update-rules`](../00-README.md#reading-and-update-rules), baseline lines 152–162
**Baseline span SHA-256:** `fcefeb722469f836cacbca46839854facc11f578fbe181f76914a4af97ca7457`
