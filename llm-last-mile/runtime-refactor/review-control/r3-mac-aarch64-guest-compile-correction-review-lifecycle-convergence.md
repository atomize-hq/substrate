# R3 MAC aarch64 guest compile correction — lifecycle and convergence review

- Packet: `AUX-R3-MAC-AARCH64-GUEST-COMPILE-CORRECTION`
- Discovery subject: `sha256:69253bdee615875b83322aefe11bf163a0517aaf55fdcab21b148c1099088db2`
- Base: `3d6b2eb1b02b1a24a1e055d12e5cbdb9b0312774` / `d59cac92405f06024cfcde0a2c22a4cf1255c3d7`
- Review mode: fresh independent read-only discovery lens, `gpt-5.6-terra` Extra High

## Discovery result

The candidate is declaration-only: the two `open` callers, one `openat` caller, and two `linkat` arguments retain their `CString::as_ptr()` values and their original fixed flags, ownership, error, and durability behavior. The ABI declaration now uses the target C character type, which matches the aarch64 GNU Linux pointer type without an unsafe conversion.

Manual differential analysis verified that the entire Linux source is byte-identical after normalizing only these four declaration fields. Every other base-tracked path is byte-identical, therefore all macOS host and Windows/WSL product paths remain unchanged. No valid lifecycle or convergence P1/P2 was found.

## Closure

A different fresh read-only `gpt-5.6-terra` Extra High closure reviewer independently recomputed the terminal fingerprint and returned **CLEAN** with zero P1/P2/P3/P4. It confirmed that the regression archives the bound base for the red check, while the current candidate retains the declaration-only runtime change and no macOS-host or Windows/WSL path change.
