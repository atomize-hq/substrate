//! Embedded kernel schema metadata.

/// The schema identifier for seam-0 kernel primitives.
pub const PRIMITIVES_V1_SCHEMA_ID: &str =
    "https://schemas.substrate.dev/lift/kernel/primitives.v1.json";

/// The current seam-0 kernel schema version.
pub const PRIMITIVES_V1_SCHEMA_VERSION: u32 = 1;

/// The filename for the seam-0 kernel primitives schema.
pub const PRIMITIVES_V1_SCHEMA_FILE: &str = "primitives.v1.json";

/// The embedded seam-0 kernel primitives schema source.
pub const PRIMITIVES_V1_SCHEMA_JSON: &str = include_str!("../../schemas/kernel/primitives.v1.json");
