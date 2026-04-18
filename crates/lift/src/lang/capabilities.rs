use std::collections::BTreeSet;
use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::pack::QueryEngineKind;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize, Default)]
#[serde(default)]
pub(crate) struct AdapterCapabilities {
    pub emits_local_edges: bool,
    pub emits_surface_markers: bool,
    pub query_engines: BTreeSet<QueryEngineKind>,
}

impl Serialize for QueryEngineKind {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for QueryEngineKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_str(QueryEngineKindVisitor)
    }
}

struct QueryEngineKindVisitor;

impl Visitor<'_> for QueryEngineKindVisitor {
    type Value = QueryEngineKind;

    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a supported query engine kind")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: de::Error,
    {
        match value {
            "tree_sitter" => Ok(QueryEngineKind::TreeSitter),
            _ => Err(E::unknown_variant(value, &["tree_sitter"])),
        }
    }
}
