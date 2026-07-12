use std::fmt;

use super::schema::TimestampV1;

const KEY_FILE_PREFIX: &[u8] = b"substrate.a1.commitment-key-file.v1\0";
const SCHEMA_VERSION: u32 = 1;
const HMAC_SHA256_TAG: u8 = 0x01;
const SECRET_KEY_LENGTH: u32 = 32;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum TempNameV1 {
    Init {
        authority_store_id: String,
        nonce: String,
    },
    Key {
        key_id: String,
        nonce: String,
    },
    Object {
        ref_id: String,
        nonce: String,
    },
    Root {
        root_revision: u64,
        nonce: String,
    },
}

impl TempNameV1 {
    pub(crate) fn parse(value: &str) -> Result<Self, StoreFormatError> {
        let body = value
            .strip_suffix(".tmp")
            .ok_or(StoreFormatError("authority temp suffix is invalid"))?;
        let mut parts = body.split("--");
        let operation = parts
            .next()
            .ok_or(StoreFormatError("authority temp operation is missing"))?;
        let identity = parts
            .next()
            .ok_or(StoreFormatError("authority temp identity is missing"))?;
        let nonce = parts
            .next()
            .ok_or(StoreFormatError("authority temp nonce is missing"))?;
        if parts.next().is_some() || !is_lower_hex_32(nonce) {
            return Err(StoreFormatError("authority temp grammar is invalid"));
        }
        match operation {
            "init" if is_prefixed_id(identity, "as_") => Ok(Self::Init {
                authority_store_id: identity.to_string(),
                nonce: nonce.to_string(),
            }),
            "key" if is_prefixed_id(identity, "ak_") => Ok(Self::Key {
                key_id: identity.to_string(),
                nonce: nonce.to_string(),
            }),
            "object" if is_prefixed_id(identity, "ao_") => Ok(Self::Object {
                ref_id: identity.to_string(),
                nonce: nonce.to_string(),
            }),
            "root" => {
                let revision = identity
                    .strip_prefix('r')
                    .filter(|revision| {
                        !revision.is_empty()
                            && (revision.as_bytes()[0] != b'0' || revision.len() == 1)
                            && revision.bytes().all(|byte| byte.is_ascii_digit())
                    })
                    .ok_or(StoreFormatError("root temp revision is invalid"))?
                    .parse::<u64>()
                    .map_err(|_| StoreFormatError("root temp revision is invalid"))?;
                if revision == 0 {
                    return Err(StoreFormatError("root temp revision must be positive"));
                }
                Ok(Self::Root {
                    root_revision: revision,
                    nonce: nonce.to_string(),
                })
            }
            _ => Err(StoreFormatError("authority temp operation is invalid")),
        }
    }

    pub(crate) fn file_name(&self) -> String {
        match self {
            Self::Init {
                authority_store_id,
                nonce,
            } => format!("init--{authority_store_id}--{nonce}.tmp"),
            Self::Key { key_id, nonce } => format!("key--{key_id}--{nonce}.tmp"),
            Self::Object { ref_id, nonce } => format!("object--{ref_id}--{nonce}.tmp"),
            Self::Root {
                root_revision,
                nonce,
            } => format!("root--r{root_revision}--{nonce}.tmp"),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct AuthorityStoreCommitmentKeyFileV1 {
    pub(crate) authority_store_id: String,
    pub(crate) key_id: String,
    pub(crate) created_at: TimestampV1,
    pub(crate) secret_key: [u8; 32],
}

impl AuthorityStoreCommitmentKeyFileV1 {
    pub(crate) fn encode(&self) -> Result<Vec<u8>, StoreFormatError> {
        validate_store_id(&self.authority_store_id)?;
        validate_key_id(&self.key_id)?;
        let mut output = Vec::with_capacity(
            KEY_FILE_PREFIX.len()
                + self.authority_store_id.len()
                + self.key_id.len()
                + self.created_at.as_str().len()
                + self.secret_key.len()
                + 41,
        );
        output.extend_from_slice(KEY_FILE_PREFIX);
        output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
        append_len64(&mut output, self.authority_store_id.as_bytes());
        append_len64(&mut output, self.key_id.as_bytes());
        output.push(HMAC_SHA256_TAG);
        append_len64(&mut output, self.created_at.as_str().as_bytes());
        output.extend_from_slice(&SECRET_KEY_LENGTH.to_be_bytes());
        output.extend_from_slice(&self.secret_key);
        Ok(output)
    }

    pub(crate) fn decode(bytes: &[u8]) -> Result<Self, StoreFormatError> {
        let mut cursor = Cursor { bytes, offset: 0 };
        cursor.read_exact(KEY_FILE_PREFIX)?;
        if cursor.read_u32()? != SCHEMA_VERSION {
            return Err(StoreFormatError("commitment key schema is unsupported"));
        }
        let authority_store_id = cursor.read_string()?;
        validate_store_id(&authority_store_id)?;
        let key_id = cursor.read_string()?;
        validate_key_id(&key_id)?;
        if cursor.read_byte()? != HMAC_SHA256_TAG {
            return Err(StoreFormatError("commitment key algorithm is unsupported"));
        }
        let created_at = TimestampV1::parse(&cursor.read_string()?)
            .map_err(|_| StoreFormatError("commitment key timestamp is invalid"))?;
        if cursor.read_u32()? != SECRET_KEY_LENGTH {
            return Err(StoreFormatError("commitment key length is invalid"));
        }
        let secret_key: [u8; 32] = cursor
            .take(SECRET_KEY_LENGTH as usize)?
            .try_into()
            .map_err(|_| StoreFormatError("commitment key length is invalid"))?;
        if cursor.offset != bytes.len() {
            return Err(StoreFormatError("commitment key has trailing bytes"));
        }
        Ok(Self {
            authority_store_id,
            key_id,
            created_at,
            secret_key,
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct StoreFormatError(&'static str);

impl fmt::Display for StoreFormatError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.0)
    }
}

impl std::error::Error for StoreFormatError {}

pub(crate) fn store_id(bytes: [u8; 16]) -> String {
    prefixed_id("as_", bytes)
}

pub(crate) fn key_id(bytes: [u8; 16]) -> String {
    prefixed_id("ak_", bytes)
}

pub(crate) fn object_ref_id(bytes: [u8; 16]) -> String {
    prefixed_id("ao_", bytes)
}

pub(crate) fn nonce(bytes: [u8; 16]) -> String {
    lower_hex(&bytes)
}

pub(crate) fn validate_store_id(value: &str) -> Result<(), StoreFormatError> {
    validate_prefixed_id(value, "as_", "authority store ID is invalid")
}

pub(crate) fn validate_key_id(value: &str) -> Result<(), StoreFormatError> {
    validate_prefixed_id(value, "ak_", "commitment key ID is invalid")
}

pub(crate) fn validate_ref_id(value: &str) -> Result<(), StoreFormatError> {
    validate_prefixed_id(value, "ao_", "authority object ref ID is invalid")
}

fn validate_prefixed_id(
    value: &str,
    prefix: &str,
    error: &'static str,
) -> Result<(), StoreFormatError> {
    if is_prefixed_id(value, prefix) {
        Ok(())
    } else {
        Err(StoreFormatError(error))
    }
}

fn prefixed_id(prefix: &str, bytes: [u8; 16]) -> String {
    format!("{prefix}{}", lower_hex(&bytes))
}

fn is_prefixed_id(value: &str, prefix: &str) -> bool {
    value.strip_prefix(prefix).is_some_and(is_lower_hex_32)
}

fn is_lower_hex_32(value: &str) -> bool {
    value.len() == 32
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn lower_hex(bytes: &[u8]) -> String {
    use std::fmt::Write as _;

    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut output, "{byte:02x}").expect("writing to String cannot fail");
    }
    output
}

fn append_len64(output: &mut Vec<u8>, value: &[u8]) {
    output.extend_from_slice(&(value.len() as u64).to_be_bytes());
    output.extend_from_slice(value);
}

struct Cursor<'a> {
    bytes: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    fn take(&mut self, length: usize) -> Result<&'a [u8], StoreFormatError> {
        let end = self
            .offset
            .checked_add(length)
            .filter(|end| *end <= self.bytes.len())
            .ok_or(StoreFormatError("commitment key envelope is truncated"))?;
        let value = &self.bytes[self.offset..end];
        self.offset = end;
        Ok(value)
    }

    fn read_exact(&mut self, expected: &[u8]) -> Result<(), StoreFormatError> {
        if self.take(expected.len())? == expected {
            Ok(())
        } else {
            Err(StoreFormatError("commitment key prefix is invalid"))
        }
    }

    fn read_byte(&mut self) -> Result<u8, StoreFormatError> {
        Ok(self.take(1)?[0])
    }

    fn read_u32(&mut self) -> Result<u32, StoreFormatError> {
        Ok(u32::from_be_bytes(self.take(4)?.try_into().map_err(
            |_| StoreFormatError("commitment key envelope is truncated"),
        )?))
    }

    fn read_u64(&mut self) -> Result<u64, StoreFormatError> {
        Ok(u64::from_be_bytes(self.take(8)?.try_into().map_err(
            |_| StoreFormatError("commitment key envelope is truncated"),
        )?))
    }

    fn read_string(&mut self) -> Result<String, StoreFormatError> {
        let length: usize = self
            .read_u64()?
            .try_into()
            .map_err(|_| StoreFormatError("commitment key length is unsupported"))?;
        std::str::from_utf8(self.take(length)?)
            .map(str::to_string)
            .map_err(|_| StoreFormatError("commitment key string is not UTF-8"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn timestamp() -> TimestampV1 {
        TimestampV1::parse("2026-07-11T12:49:11.000000000Z").unwrap()
    }

    #[test]
    fn closed_temp_grammar_and_key_envelope_are_exact() {
        let nonce = "11111111111111111111111111111111";
        let store = "as_22222222222222222222222222222222";
        let key = "ak_33333333333333333333333333333333";
        let object = "ao_44444444444444444444444444444444";
        for name in [
            format!("init--{store}--{nonce}.tmp"),
            format!("key--{key}--{nonce}.tmp"),
            format!("object--{object}--{nonce}.tmp"),
            format!("root--r1--{nonce}.tmp"),
        ] {
            let parsed = TempNameV1::parse(&name).unwrap();
            assert_eq!(parsed.file_name(), name);
        }
        for invalid in [
            format!("root--r01--{nonce}.tmp"),
            format!("root--r0--{nonce}.tmp"),
            format!("init--{store}--AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA.tmp"),
            format!("unknown--{object}--{nonce}.tmp"),
            format!("object--{object}--{nonce}.tmp.extra"),
        ] {
            assert!(TempNameV1::parse(&invalid).is_err(), "accepted {invalid}");
        }

        let key_file = AuthorityStoreCommitmentKeyFileV1 {
            authority_store_id: store.into(),
            key_id: key.into(),
            created_at: timestamp(),
            secret_key: std::array::from_fn(|index| index as u8),
        };
        let encoded = key_file.encode().unwrap();
        assert!(encoded.starts_with(KEY_FILE_PREFIX));
        assert_eq!(
            AuthorityStoreCommitmentKeyFileV1::decode(&encoded).unwrap(),
            key_file
        );

        let mut trailing = encoded.clone();
        trailing.push(0);
        assert!(AuthorityStoreCommitmentKeyFileV1::decode(&trailing).is_err());
        let mut wrong_tag = encoded;
        let tag_offset = KEY_FILE_PREFIX.len() + 4 + 8 + store.len() + 8 + key.len();
        wrong_tag[tag_offset] = 2;
        assert!(AuthorityStoreCommitmentKeyFileV1::decode(&wrong_tag).is_err());
    }

    #[test]
    fn generated_identifiers_have_the_closed_lower_hex_shape() {
        assert_eq!(store_id([0xab; 16]), "as_abababababababababababababababab");
        assert_eq!(key_id([0xcd; 16]), "ak_cdcdcdcdcdcdcdcdcdcdcdcdcdcdcdcd");
        assert_eq!(
            object_ref_id([0xef; 16]),
            "ao_efefefefefefefefefefefefefefefef"
        );
        assert_eq!(nonce([0x12; 16]), "12121212121212121212121212121212");
        assert!(validate_store_id("as_ABABABABABABABABABABABABABABABAB").is_err());
        assert!(validate_key_id("ak_short").is_err());
        assert!(validate_ref_id("ao_00000000000000000000000000000000").is_ok());
    }
}
