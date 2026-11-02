// This file is derived from committed repo truth.
// Validate it with `cargo test -p xtask --all-targets`.

const CODEX_RUNTIME_SUPPORT: &[EmbeddedRuntimeSupportRecord] = &[
    EmbeddedRuntimeSupportRecord {
        target_triple: "aarch64-apple-darwin",
        latest_validated: Some("0.125.0"),
    },
    EmbeddedRuntimeSupportRecord {
        target_triple: "aarch64-unknown-linux-musl",
        latest_validated: Some("0.125.0"),
    },
    EmbeddedRuntimeSupportRecord {
        target_triple: "x86_64-pc-windows-msvc",
        latest_validated: Some("0.125.0"),
    },
    EmbeddedRuntimeSupportRecord {
        target_triple: "x86_64-unknown-linux-musl",
        latest_validated: Some("0.125.0"),
    },
];
