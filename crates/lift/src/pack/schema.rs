//! Embedded pack schema metadata.

#![allow(dead_code)]

/// The schema identifier for seam-1 pack common definitions.
pub(crate) const PACK_COMMON_V1_SCHEMA_ID: &str =
    "https://schemas.substrate.dev/lift/pack/common.v1.json";

/// The current seam-1 pack common schema version.
pub(crate) const PACK_COMMON_V1_SCHEMA_VERSION: u32 = 1;

/// The filename for the seam-1 pack common schema.
pub(crate) const PACK_COMMON_V1_SCHEMA_FILE: &str = "common.v1.json";

/// The embedded seam-1 pack common schema source.
pub(crate) const PACK_COMMON_V1_SCHEMA_JSON: &str =
    include_str!("../../schemas/pack/common.v1.json");

/// The schema identifier for seam-1 profile packs.
pub(crate) const PACK_PROFILE_V1_SCHEMA_ID: &str =
    "https://schemas.substrate.dev/lift/pack/profile.v1.json";

/// The current seam-1 profile schema version.
pub(crate) const PACK_PROFILE_V1_SCHEMA_VERSION: u32 = 1;

/// The filename for the seam-1 profile schema.
pub(crate) const PACK_PROFILE_V1_SCHEMA_FILE: &str = "profile.v1.json";

/// The embedded seam-1 profile schema source.
pub(crate) const PACK_PROFILE_V1_SCHEMA_JSON: &str =
    include_str!("../../schemas/pack/profile.v1.json");
