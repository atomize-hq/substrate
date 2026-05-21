//! Stable identifier primitives.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use super::error::{KernelError, KernelResult};

const DIGEST_LEN: usize = 64;
const KIND_SEPARATOR: &str = ":sha256:";
const IDENTITY_PREFIX: &str = "lift-kernel\0v1\0";

/// A stable identifier with the wire format `<kind>:sha256:<64-lower-hex>`.
#[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct StableId(String);

impl StableId {
    /// Parses and validates a stable identifier.
    pub fn parse(input: &str) -> KernelResult<Self> {
        validate_stable_id(input, None)?;
        Ok(Self(input.to_owned()))
    }

    /// Builds a deterministic stable identifier from a kind and identity lemma.
    pub fn from_identity(kind: &'static str, identity_lemma: &str) -> Self {
        let mut hasher = Sha256::new();
        hasher.update(IDENTITY_PREFIX.as_bytes());
        hasher.update(kind.as_bytes());
        hasher.update([0]);
        hasher.update(identity_lemma.as_bytes());

        let digest = lower_hex(&hasher.finalize());
        Self(format!("{kind}{KIND_SEPARATOR}{digest}"))
    }

    /// Returns the kind prefix of the stable identifier.
    pub fn kind(&self) -> &str {
        self.0
            .split_once(KIND_SEPARATOR)
            .map_or("", |(kind, _)| kind)
    }

    /// Returns the stable identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    fn parse_typed(input: &str, expected_kind: &'static str) -> KernelResult<Self> {
        validate_stable_id(input, Some(expected_kind))?;
        Ok(Self(input.to_owned()))
    }
}

impl TryFrom<String> for StableId {
    type Error = KernelError;

    fn try_from(value: String) -> KernelResult<Self> {
        Self::parse(&value)
    }
}

impl From<StableId> for String {
    fn from(value: StableId) -> Self {
        value.0
    }
}

macro_rules! typed_stable_id {
    ($name:ident, $kind:literal, $doc:literal) => {
        #[doc = $doc]
        #[derive(Clone, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
        #[serde(try_from = "String", into = "String")]
        pub struct $name(StableId);

        impl $name {
            /// The stable identifier kind for this typed wrapper.
            pub const KIND: &'static str = $kind;

            /// Parses and validates a typed stable identifier.
            pub fn parse(input: &str) -> KernelResult<Self> {
                Ok(Self(StableId::parse_typed(input, Self::KIND)?))
            }

            /// Builds a deterministic typed stable identifier from an identity lemma.
            pub fn from_identity(identity_lemma: &str) -> Self {
                Self(StableId::from_identity(Self::KIND, identity_lemma))
            }

            /// Returns the identifier as a string slice.
            pub fn as_str(&self) -> &str {
                self.0.as_str()
            }
        }

        impl TryFrom<String> for $name {
            type Error = KernelError;

            fn try_from(value: String) -> KernelResult<Self> {
                Self::parse(&value)
            }
        }

        impl From<$name> for String {
            fn from(value: $name) -> Self {
                value.0 .0
            }
        }

        impl From<$name> for StableId {
            fn from(value: $name) -> Self {
                value.0
            }
        }
    };
}

typed_stable_id!(FileId, "file", "Typed stable identifier for file entities.");
typed_stable_id!(
    SymbolId,
    "symbol",
    "Typed stable identifier for symbol entities."
);
typed_stable_id!(
    ComponentId,
    "component",
    "Typed stable identifier for component entities."
);
typed_stable_id!(
    BoundaryId,
    "boundary",
    "Typed stable identifier for boundary entities."
);
typed_stable_id!(FactId, "fact", "Typed stable identifier for fact entities.");
typed_stable_id!(
    EvidenceId,
    "evidence",
    "Typed stable identifier for evidence entities."
);
typed_stable_id!(RuleId, "rule", "Typed stable identifier for rule entities.");
typed_stable_id!(
    QueryId,
    "query",
    "Typed stable identifier for query entities."
);
typed_stable_id!(
    RecipeId,
    "recipe",
    "Typed stable identifier for recipe entities."
);
typed_stable_id!(
    MatchId,
    "match",
    "Typed stable identifier for match entities."
);

fn validate_stable_id(input: &str, expected_kind: Option<&'static str>) -> KernelResult<()> {
    let (kind, digest) =
        input
            .split_once(KIND_SEPARATOR)
            .ok_or_else(|| KernelError::InvalidStableId {
                input: input.to_owned(),
                expected_kind,
            })?;

    if !valid_kind(kind) || digest.len() != DIGEST_LEN || !digest.bytes().all(is_lower_hex) {
        return Err(KernelError::InvalidStableId {
            input: input.to_owned(),
            expected_kind,
        });
    }

    if let Some(expected_kind) = expected_kind {
        if kind != expected_kind {
            return Err(KernelError::InvalidStableId {
                input: input.to_owned(),
                expected_kind: Some(expected_kind),
            });
        }
    }

    Ok(())
}

fn valid_kind(kind: &str) -> bool {
    let mut chars = kind.chars();
    matches!(chars.next(), Some(ch) if ch.is_ascii_lowercase())
        && chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || matches!(byte, b'a'..=b'f')
}

fn lower_hex(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        out.push(nibble_to_hex(byte >> 4));
        out.push(nibble_to_hex(byte & 0x0f));
    }
    out
}

fn nibble_to_hex(nibble: u8) -> char {
    match nibble {
        0..=9 => (b'0' + nibble) as char,
        10..=15 => (b'a' + (nibble - 10)) as char,
        _ => unreachable!("nibble must be in 0..=15"),
    }
}

#[cfg(test)]
mod tests {
    use super::{FileId, QueryId, StableId};

    #[test]
    fn stable_id_from_identity_is_deterministic() {
        let left = StableId::from_identity("file", "src/lib.rs");
        let right = StableId::from_identity("file", "src/lib.rs");

        assert_eq!(left, right);
        assert_eq!(
            left.as_str(),
            "file:sha256:13ec57d807bcff8c7ee2ffc89b3adfb999e5b183d93474f425ebbe2ce371c416"
        );
    }

    #[test]
    fn typed_ids_reject_wrong_kinds() {
        let err = QueryId::parse(
            "file:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef",
        )
        .expect_err("query ids must reject file ids");

        match err {
            crate::kernel::KernelError::InvalidStableId {
                expected_kind: Some(kind),
                ..
            } => assert_eq!(kind, QueryId::KIND),
            other => panic!("unexpected error: {other:?}"),
        }
    }

    #[test]
    fn serde_round_trip_uses_validation() {
        let stable: StableId = serde_json::from_str(
            "\"file:sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef\"",
        )
        .expect("stable id should parse");
        assert_eq!(stable.kind(), "file");
        assert!(serde_json::from_str::<StableId>("\"bad\"").is_err());

        let file = FileId::from_identity("src/lib.rs");
        let round_trip: FileId =
            serde_json::from_str(&serde_json::to_string(&file).expect("serialize")).expect("parse");
        assert_eq!(round_trip.as_str(), file.as_str());
    }
}
