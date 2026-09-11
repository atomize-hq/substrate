use std::collections::BTreeMap;

use serde::de::{DeserializeOwned, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};

use crate::ConfigProjectionFailureV1;

/// Canonical JSON and domain-separated hashing for the E3 V1 authority.
pub struct ConfigProjectionCodecV1;

impl ConfigProjectionCodecV1 {
    pub fn encode_canonical_json<T: Serialize>(
        value: &T,
    ) -> Result<Vec<u8>, ConfigProjectionFailureV1> {
        let mut value =
            serde_json::to_value(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        reject_floats(&value)?;
        sort_object_keys(&mut value);
        let mut bytes = Vec::new();
        write_canonical_json(&value, &mut bytes)?;
        Ok(bytes)
    }

    pub fn decode_canonical_json<T>(bytes: &[u8]) -> Result<T, ConfigProjectionFailureV1>
    where
        T: DeserializeOwned + Serialize,
    {
        let mut deserializer = serde_json::Deserializer::from_slice(bytes);
        let strict = StrictJsonValue::deserialize(&mut deserializer)
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        deserializer
            .end()
            .map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        let value = strict.into_json();
        let decoded =
            serde_json::from_value(value).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
        if Self::encode_canonical_json(&decoded)? != bytes {
            return Err(ConfigProjectionFailureV1::Malformed);
        }
        Ok(decoded)
    }

    /// Hashes canonical bytes after adding `domain` to an object payload.
    ///
    /// Passing an empty domain hashes the value itself, which is useful for
    /// ordinary byte digests and the canonical codec's golden vectors.
    pub fn domain_sha256<T: Serialize>(
        domain: &str,
        payload: &T,
    ) -> Result<String, ConfigProjectionFailureV1> {
        let bytes = if domain.is_empty() {
            Self::encode_canonical_json(payload)?
        } else {
            let value =
                serde_json::to_value(payload).map_err(|_| ConfigProjectionFailureV1::Malformed)?;
            let serde_json::Value::Object(values) = value else {
                return Err(ConfigProjectionFailureV1::Malformed);
            };
            if values.contains_key("domain") {
                return Err(ConfigProjectionFailureV1::Malformed);
            }
            let mut preimage = BTreeMap::new();
            preimage.insert(
                "domain".to_string(),
                serde_json::Value::String(domain.to_string()),
            );
            preimage.extend(values);
            Self::encode_canonical_json(&preimage)?
        };
        Ok(lower_hex(&Sha256::digest(bytes)))
    }
}

fn sort_object_keys(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Array(values) => values.iter_mut().for_each(sort_object_keys),
        serde_json::Value::Object(values) => {
            let mut entries = std::mem::take(values).into_iter().collect::<Vec<_>>();
            entries.sort_by(|(left, _), (right, _)| left.as_bytes().cmp(right.as_bytes()));
            for (key, mut value) in entries {
                sort_object_keys(&mut value);
                values.insert(key, value);
            }
        }
        _ => {}
    }
}

fn write_canonical_json(
    value: &serde_json::Value,
    output: &mut Vec<u8>,
) -> Result<(), ConfigProjectionFailureV1> {
    match value {
        serde_json::Value::Null => output.extend_from_slice(b"null"),
        serde_json::Value::Bool(false) => output.extend_from_slice(b"false"),
        serde_json::Value::Bool(true) => output.extend_from_slice(b"true"),
        serde_json::Value::Number(number) if number.is_i64() || number.is_u64() => {
            output.extend_from_slice(number.to_string().as_bytes());
        }
        serde_json::Value::Number(_) => return Err(ConfigProjectionFailureV1::Malformed),
        serde_json::Value::String(value) => write_canonical_string(value, output),
        serde_json::Value::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_json(value, output)?;
            }
            output.push(b']');
        }
        serde_json::Value::Object(values) => {
            output.push(b'{');
            for (index, (key, value)) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                write_canonical_string(key, output);
                output.push(b':');
                write_canonical_json(value, output)?;
            }
            output.push(b'}');
        }
    }
    Ok(())
}

fn write_canonical_string(value: &str, output: &mut Vec<u8>) {
    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(br#"\""#),
            '\\' => output.extend_from_slice(br#"\\"#),
            '\u{8}' => output.extend_from_slice(br"\b"),
            '\t' => output.extend_from_slice(br"\t"),
            '\n' => output.extend_from_slice(br"\n"),
            '\u{c}' => output.extend_from_slice(br"\f"),
            '\r' => output.extend_from_slice(br"\r"),
            '\0'..='\u{1f}' | '\u{7f}' => {
                const HEX: &[u8; 16] = b"0123456789abcdef";
                let code = character as u8;
                output.extend_from_slice(br"\u00");
                output.push(HEX[usize::from(code >> 4)]);
                output.push(HEX[usize::from(code & 0x0f)]);
            }
            _ => {
                let mut buffer = [0_u8; 4];
                output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    output.push(b'"');
}

fn reject_floats(value: &serde_json::Value) -> Result<(), ConfigProjectionFailureV1> {
    match value {
        serde_json::Value::Number(number) if !number.is_i64() && !number.is_u64() => {
            Err(ConfigProjectionFailureV1::Malformed)
        }
        serde_json::Value::Array(values) => values.iter().try_for_each(reject_floats),
        serde_json::Value::Object(values) => values.values().try_for_each(reject_floats),
        _ => Ok(()),
    }
}

fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    output
}

enum StrictJsonValue {
    Null,
    Bool(bool),
    Number(serde_json::Number),
    String(String),
    Array(Vec<Self>),
    Object(BTreeMap<String, Self>),
}

impl StrictJsonValue {
    fn into_json(self) -> serde_json::Value {
        match self {
            Self::Null => serde_json::Value::Null,
            Self::Bool(value) => serde_json::Value::Bool(value),
            Self::Number(value) => serde_json::Value::Number(value),
            Self::String(value) => serde_json::Value::String(value),
            Self::Array(values) => serde_json::Value::Array(
                values.into_iter().map(StrictJsonValue::into_json).collect(),
            ),
            Self::Object(values) => serde_json::Value::Object(
                values
                    .into_iter()
                    .map(|(key, value)| (key, value.into_json()))
                    .collect(),
            ),
        }
    }
}

impl<'de> Deserialize<'de> for StrictJsonValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_any(StrictJsonVisitor)
    }
}

struct StrictJsonVisitor;

impl<'de> Visitor<'de> for StrictJsonVisitor {
    type Value = StrictJsonValue;

    fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str("canonical integer-only JSON")
    }

    fn visit_unit<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Null)
    }

    fn visit_none<E>(self) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Null)
    }

    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Bool(value))
    }

    fn visit_i64<E>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Number(value.into()))
    }

    fn visit_u64<E>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::Number(value.into()))
    }

    fn visit_f64<E>(self, _value: f64) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Err(E::custom("floating-point JSON is forbidden"))
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::String(value.to_string()))
    }

    fn visit_string<E>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictJsonValue::String(value))
    }

    fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
    where
        A: SeqAccess<'de>,
    {
        let mut values = Vec::new();
        while let Some(value) = sequence.next_element()? {
            values.push(value);
        }
        Ok(StrictJsonValue::Array(values))
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut values = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, StrictJsonValue>()? {
            if values.insert(key, value).is_some() {
                return Err(serde::de::Error::custom("duplicate JSON object key"));
            }
        }
        Ok(StrictJsonValue::Object(values))
    }
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use serde::{Deserialize, Serialize};

    use super::ConfigProjectionCodecV1;
    use crate::ConfigProjectionRefV1;

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        a: String,
        #[serde(rename = "é")]
        snow: String,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    struct OrderingFixture {
        z: u64,
        a: u64,
        nested: serde_json::Value,
    }

    #[test]
    fn e3_b_codec_emits_golden_bytes_and_domain_hash() {
        let fixture = Fixture {
            a: "\u{8}\t\n\u{c}\r\0".to_string(),
            snow: "雪".to_string(),
        };
        let bytes = ConfigProjectionCodecV1::encode_canonical_json(&fixture).unwrap();
        assert_eq!(bytes, r#"{"a":"\b\t\n\f\r\u0000","é":"雪"}"#.as_bytes());
        assert_eq!(
            ConfigProjectionCodecV1::domain_sha256("", &fixture).unwrap(),
            "f56970062a492ea5987431971823cbce9374f4aa9f75ebe2e0d78645d2eb6867"
        );

        let ordering = OrderingFixture {
            z: 2,
            a: 1,
            nested: serde_json::json!({"z": 2, "b": {"y": 1, "a": 0}}),
        };
        assert_eq!(
            ConfigProjectionCodecV1::encode_canonical_json(&ordering).unwrap(),
            br#"{"a":1,"nested":{"b":{"a":0,"y":1},"z":2},"z":2}"#
        );
        assert_eq!(
            ConfigProjectionCodecV1::encode_canonical_json(&serde_json::json!({
                "control": "\u{7f}"
            }))
            .unwrap(),
            br#"{"control":"\u007f"}"#
        );

        let mut payload = BTreeMap::new();
        payload.insert("record", serde_json::json!({"z": 2, "a": 1}));
        assert_eq!(
            ConfigProjectionCodecV1::domain_sha256("substrate.test.v1", &payload)
                .unwrap()
                .len(),
            64
        );
    }

    #[test]
    fn e3_b_codec_rejects_noncanonical_duplicate_float_unknown_and_bad_reference() {
        assert!(ConfigProjectionCodecV1::decode_canonical_json::<Fixture>(
            r#"{"é":"雪","a":"x"}"#.as_bytes()
        )
        .is_err());
        assert!(ConfigProjectionCodecV1::decode_canonical_json::<Fixture>(
            r#"{"a":"x","a":"y","é":"雪"}"#.as_bytes()
        )
        .is_err());
        assert!(
            ConfigProjectionCodecV1::decode_canonical_json::<serde_json::Value>(b"1.0").is_err()
        );
        assert!(ConfigProjectionCodecV1::decode_canonical_json::<Fixture>(
            r#"{"a":"x","extra":0,"é":"雪"}"#.as_bytes()
        )
        .is_err());
        assert!(ConfigProjectionCodecV1::decode_canonical_json::<ConfigProjectionRefV1>(
            br#"{"authority_store_id":"cpa_bad","record_hash":"00","record_id":"cpr_bad","revision":1,"series_id":"cps_bad"}"#
        )
        .is_err());
    }
}
