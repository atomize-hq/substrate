# world-deps-apt-provisioning — draft-path compatibility shim

This draft-path document is preserved as a compatibility shim for references that
still point at the historical pack location.

Authoritative contract:
- `docs/project_management/packs/implemented/world-deps-apt-provisioning/contract.md`

Key preserved operator/runtime guidance:
- Provisioning remediation remains:

  ```text
  substrate world enable --provision-deps
  ```

- On Windows, runtime/provisioning guidance remains unsupported on Windows.
- On Linux host-native backends, Substrate will not mutate the host OS.

For the full CLI, platform/backend, exit-code, and fail-early contract, follow the
implemented contract path above.
