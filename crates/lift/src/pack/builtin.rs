//! Embedded built-in packs.

use crate::pack::source::{PackFormat, PackSource};

const GENERIC_DEFAULT_PROFILE: &str = r#"kind = "profile"
version = 1
id = "generic/default"
name = "Generic default profile"
description = "Default deterministic profile for generic repositories"

[apps]
enabled = ["score"]
default = "score"

[analysis]
languages = ["json", "toml", "yaml", "rust", "python", "javascript", "typescript"]
follow_symlinks = false
max_scope_depth = 2
"#;

/// Returns a built-in profile source by logical pack name.
pub(crate) fn profile_source(logical_name: &str) -> Option<PackSource> {
    match logical_name {
        "generic/default" => Some(PackSource::Builtin {
            logical_name: "generic/default",
            format: PackFormat::Toml,
            bytes: GENERIC_DEFAULT_PROFILE.as_bytes(),
        }),
        _ => None,
    }
}
