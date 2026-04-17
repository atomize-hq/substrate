#![allow(dead_code)]

pub(crate) const SNAPSHOT_MANIFEST_V1_SCHEMA_ID: &str =
    "https://schemas.substrate.dev/lift/repo/snapshot_manifest.v1.json";

pub(crate) const SNAPSHOT_MANIFEST_V1_SCHEMA_VERSION: u32 = 1;

pub(crate) const SNAPSHOT_MANIFEST_V1_SCHEMA_FILE: &str = "snapshot_manifest.v1.json";

pub(crate) const SNAPSHOT_MANIFEST_V1_SCHEMA_JSON: &str =
    include_str!("../../schemas/repo/snapshot_manifest.v1.json");
