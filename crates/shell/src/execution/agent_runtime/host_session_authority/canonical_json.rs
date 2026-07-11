use std::collections::BTreeMap;
use std::fmt;

use serde::de::{DeserializeOwned, IntoDeserializer, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq};
use serde::Serialize;

#[derive(Clone, Debug, PartialEq, Eq)]
enum CanonicalValue {
    Null,
    Bool(bool),
    I64(i64),
    U64(u64),
    String(String),
    Array(Vec<CanonicalValue>),
    Object(BTreeMap<String, CanonicalValue>),
    EnumTag(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub(crate) struct CanonicalJsonError(String);

impl CanonicalJsonError {
    fn new(message: impl Into<String>) -> Self {
        Self(message.into())
    }
}

impl fmt::Display for CanonicalJsonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

impl std::error::Error for CanonicalJsonError {}

impl serde::ser::Error for CanonicalJsonError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::new(message.to_string())
    }
}

impl serde::de::Error for CanonicalJsonError {
    fn custom<T: fmt::Display>(message: T) -> Self {
        Self::new(message.to_string())
    }
}

pub(crate) fn to_vec<T: Serialize>(value: &T) -> Result<Vec<u8>, CanonicalJsonError> {
    let value = value.serialize(ValueSerializer)?;
    if contains_unbound_enum_tag(&value) {
        return Err(CanonicalJsonError::new(
            "A1 enum tags are valid only inside a kind field",
        ));
    }
    let mut output = Vec::new();
    encode_value(&value, &mut output);
    Ok(output)
}

pub(crate) fn from_slice<T: DeserializeOwned + Serialize>(
    bytes: &[u8],
) -> Result<T, CanonicalJsonError> {
    let input = std::str::from_utf8(bytes)
        .map_err(|_| CanonicalJsonError::new("CanonicalJsonV1 requires valid UTF-8"))?;
    let mut parser = Parser { input, offset: 0 };
    let value = parser.parse_value()?;
    if parser.offset != input.len() {
        return Err(CanonicalJsonError::new(
            "CanonicalJsonV1 rejects trailing bytes",
        ));
    }
    let decoded = T::deserialize(value)?;
    if to_vec(&decoded)? != bytes {
        return Err(CanonicalJsonError::new(
            "CanonicalJsonV1 requires every declared field and exact typed encoding",
        ));
    }
    Ok(decoded)
}

struct ValueSerializer;

impl serde::Serializer for ValueSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;
    type SerializeSeq = SequenceSerializer;
    type SerializeTuple = SequenceSerializer;
    type SerializeTupleStruct = SequenceSerializer;
    type SerializeTupleVariant = serde::ser::Impossible<CanonicalValue, CanonicalJsonError>;
    type SerializeMap = ObjectSerializer;
    type SerializeStruct = ObjectSerializer;
    type SerializeStructVariant = serde::ser::Impossible<CanonicalValue, CanonicalJsonError>;

    fn serialize_bool(self, value: bool) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::Bool(value))
    }

    fn serialize_i8(self, value: i8) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i16(self, value: i16) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i32(self, value: i32) -> Result<Self::Ok, Self::Error> {
        self.serialize_i64(value.into())
    }

    fn serialize_i64(self, value: i64) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::I64(value))
    }

    fn serialize_i128(self, value: i128) -> Result<Self::Ok, Self::Error> {
        i64::try_from(value)
            .map(CanonicalValue::I64)
            .map_err(|_| CanonicalJsonError::new("signed integer exceeds declared V1 range"))
    }

    fn serialize_u8(self, value: u8) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(value.into())
    }

    fn serialize_u16(self, value: u16) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(value.into())
    }

    fn serialize_u32(self, value: u32) -> Result<Self::Ok, Self::Error> {
        self.serialize_u64(value.into())
    }

    fn serialize_u64(self, value: u64) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::U64(value))
    }

    fn serialize_u128(self, value: u128) -> Result<Self::Ok, Self::Error> {
        u64::try_from(value)
            .map(CanonicalValue::U64)
            .map_err(|_| CanonicalJsonError::new("unsigned integer exceeds declared V1 range"))
    }

    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalJsonError::new(
            "CanonicalJsonV1 rejects floating-point values",
        ))
    }

    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalJsonError::new(
            "CanonicalJsonV1 rejects floating-point values",
        ))
    }

    fn serialize_char(self, value: char) -> Result<Self::Ok, Self::Error> {
        self.serialize_str(&value.to_string())
    }

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::String(value.to_string()))
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalJsonError::new(
            "CanonicalJsonV1 does not coerce raw bytes into JSON",
        ))
    }

    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::Null)
    }

    fn serialize_some<T: ?Sized + Serialize>(self, value: &T) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::Null)
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        self.serialize_unit()
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::EnumTag(variant.to_string()))
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        Err(CanonicalJsonError::new(
            "A1 enums must use the closed kind/value representation",
        ))
    }

    fn serialize_seq(self, length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        Ok(SequenceSerializer(Vec::with_capacity(length.unwrap_or(0))))
    }

    fn serialize_tuple(self, length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        self.serialize_seq(Some(length))
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        Err(CanonicalJsonError::new(
            "A1 enums must use the closed kind/value representation",
        ))
    }

    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        Ok(ObjectSerializer::default())
    }

    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        Ok(ObjectSerializer::default())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        Err(CanonicalJsonError::new(
            "A1 enums must use the closed kind/value representation",
        ))
    }
}

struct SequenceSerializer(Vec<CanonicalValue>);

impl SerializeSeq for SequenceSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        self.0.push(value.serialize(ValueSerializer)?);
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::Array(self.0))
    }
}

impl serde::ser::SerializeTuple for SequenceSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;

    fn serialize_element<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

impl serde::ser::SerializeTupleStruct for SequenceSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        SerializeSeq::serialize_element(self, value)
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        SerializeSeq::end(self)
    }
}

#[derive(Default)]
struct ObjectSerializer {
    values: BTreeMap<String, CanonicalValue>,
    next_key: Option<String>,
}

impl SerializeMap for ObjectSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;

    fn serialize_key<T: ?Sized + Serialize>(&mut self, key: &T) -> Result<(), Self::Error> {
        self.next_key = Some(key.serialize(MapKeySerializer)?);
        Ok(())
    }

    fn serialize_value<T: ?Sized + Serialize>(&mut self, value: &T) -> Result<(), Self::Error> {
        let key = self
            .next_key
            .take()
            .ok_or_else(|| CanonicalJsonError::new("map value is missing its string key"))?;
        let value = bind_enum_tag(&key, value.serialize(ValueSerializer)?)?;
        if self.values.insert(key.clone(), value).is_some() {
            return Err(CanonicalJsonError::new(format!(
                "duplicate object key {key}"
            )));
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        if self.next_key.is_some() {
            return Err(CanonicalJsonError::new("map key is missing its value"));
        }
        Ok(CanonicalValue::Object(self.values))
    }
}

impl serde::ser::SerializeStruct for ObjectSerializer {
    type Ok = CanonicalValue;
    type Error = CanonicalJsonError;

    fn serialize_field<T: ?Sized + Serialize>(
        &mut self,
        key: &'static str,
        value: &T,
    ) -> Result<(), Self::Error> {
        let value = bind_enum_tag(key, value.serialize(ValueSerializer)?)?;
        if self.values.insert(key.to_string(), value).is_some() {
            return Err(CanonicalJsonError::new(format!(
                "duplicate struct field {key}"
            )));
        }
        Ok(())
    }

    fn end(self) -> Result<Self::Ok, Self::Error> {
        Ok(CanonicalValue::Object(self.values))
    }
}

struct MapKeySerializer;

impl serde::Serializer for MapKeySerializer {
    type Ok = String;
    type Error = CanonicalJsonError;
    type SerializeSeq = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeTuple = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeTupleStruct = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeTupleVariant = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeMap = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeStruct = serde::ser::Impossible<String, CanonicalJsonError>;
    type SerializeStructVariant = serde::ser::Impossible<String, CanonicalJsonError>;

    fn serialize_str(self, value: &str) -> Result<Self::Ok, Self::Error> {
        Ok(value.to_string())
    }

    fn serialize_newtype_struct<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        value.serialize(self)
    }

    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
    ) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }

    fn serialize_bool(self, _value: bool) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_i8(self, _value: i8) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_i16(self, _value: i16) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_i32(self, _value: i32) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_i64(self, _value: i64) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_i128(self, _value: i128) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_u8(self, _value: u8) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_u16(self, _value: u16) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_u32(self, _value: u32) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_u64(self, _value: u64) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_u128(self, _value: u128) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_f32(self, _value: f32) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_f64(self, _value: f64) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_char(self, _value: char) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_bytes(self, _value: &[u8]) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_none(self) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_some<T: ?Sized + Serialize>(self, _value: &T) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_unit(self) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_unit_struct(self, _name: &'static str) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_newtype_variant<T: ?Sized + Serialize>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<Self::Ok, Self::Error> {
        invalid_map_key()
    }
    fn serialize_seq(self, _length: Option<usize>) -> Result<Self::SerializeSeq, Self::Error> {
        invalid_map_key()
    }
    fn serialize_tuple(self, _length: usize) -> Result<Self::SerializeTuple, Self::Error> {
        invalid_map_key()
    }
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleStruct, Self::Error> {
        invalid_map_key()
    }
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeTupleVariant, Self::Error> {
        invalid_map_key()
    }
    fn serialize_map(self, _length: Option<usize>) -> Result<Self::SerializeMap, Self::Error> {
        invalid_map_key()
    }
    fn serialize_struct(
        self,
        _name: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStruct, Self::Error> {
        invalid_map_key()
    }
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _length: usize,
    ) -> Result<Self::SerializeStructVariant, Self::Error> {
        invalid_map_key()
    }
}

fn invalid_map_key<T>() -> Result<T, CanonicalJsonError> {
    Err(CanonicalJsonError::new(
        "CanonicalJsonV1 object keys must be strings",
    ))
}

fn bind_enum_tag(key: &str, value: CanonicalValue) -> Result<CanonicalValue, CanonicalJsonError> {
    match value {
        CanonicalValue::EnumTag(variant) if key == "kind" => Ok(CanonicalValue::String(variant)),
        CanonicalValue::EnumTag(_) => Err(CanonicalJsonError::new(
            "A1 enum tags are valid only inside a kind field",
        )),
        value => Ok(value),
    }
}

fn contains_unbound_enum_tag(value: &CanonicalValue) -> bool {
    match value {
        CanonicalValue::EnumTag(_) => true,
        CanonicalValue::Array(values) => values.iter().any(contains_unbound_enum_tag),
        CanonicalValue::Object(values) => values.values().any(contains_unbound_enum_tag),
        _ => false,
    }
}

fn encode_value(value: &CanonicalValue, output: &mut Vec<u8>) {
    match value {
        CanonicalValue::Null => output.extend_from_slice(b"null"),
        CanonicalValue::Bool(true) => output.extend_from_slice(b"true"),
        CanonicalValue::Bool(false) => output.extend_from_slice(b"false"),
        CanonicalValue::I64(value) => output.extend_from_slice(value.to_string().as_bytes()),
        CanonicalValue::U64(value) => output.extend_from_slice(value.to_string().as_bytes()),
        CanonicalValue::String(value) => encode_string(value, output),
        CanonicalValue::Array(values) => {
            output.push(b'[');
            for (index, value) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                encode_value(value, output);
            }
            output.push(b']');
        }
        CanonicalValue::Object(values) => {
            output.push(b'{');
            for (index, (key, value)) in values.iter().enumerate() {
                if index != 0 {
                    output.push(b',');
                }
                encode_string(key, output);
                output.push(b':');
                encode_value(value, output);
            }
            output.push(b'}');
        }
        CanonicalValue::EnumTag(_) => unreachable!("unbound enum tags are rejected before encode"),
    }
}

fn encode_string(value: &str, output: &mut Vec<u8>) {
    output.push(b'"');
    for character in value.chars() {
        match character {
            '"' => output.extend_from_slice(br#"\""#),
            '\\' => output.extend_from_slice(br#"\\"#),
            character if character <= '\u{1f}' => {
                output.extend_from_slice(format!("\\u00{:02x}", character as u32).as_bytes());
            }
            character => {
                let mut buffer = [0_u8; 4];
                output.extend_from_slice(character.encode_utf8(&mut buffer).as_bytes());
            }
        }
    }
    output.push(b'"');
}

struct Parser<'a> {
    input: &'a str,
    offset: usize,
}

impl Parser<'_> {
    fn parse_value(&mut self) -> Result<CanonicalValue, CanonicalJsonError> {
        match self.peek() {
            Some(b'n') => self.parse_literal("null", CanonicalValue::Null),
            Some(b't') => self.parse_literal("true", CanonicalValue::Bool(true)),
            Some(b'f') => self.parse_literal("false", CanonicalValue::Bool(false)),
            Some(b'"') => self.parse_string().map(CanonicalValue::String),
            Some(b'[') => self.parse_array(),
            Some(b'{') => self.parse_object(),
            Some(b'-' | b'0'..=b'9') => self.parse_integer(),
            Some(_) => Err(CanonicalJsonError::new(
                "CanonicalJsonV1 contains an invalid token or whitespace",
            )),
            None => Err(CanonicalJsonError::new("CanonicalJsonV1 is empty")),
        }
    }

    fn parse_literal(
        &mut self,
        literal: &str,
        value: CanonicalValue,
    ) -> Result<CanonicalValue, CanonicalJsonError> {
        if self.input[self.offset..].starts_with(literal) {
            self.offset += literal.len();
            Ok(value)
        } else {
            Err(CanonicalJsonError::new("invalid CanonicalJsonV1 literal"))
        }
    }

    fn parse_array(&mut self) -> Result<CanonicalValue, CanonicalJsonError> {
        self.offset += 1;
        let mut values = Vec::new();
        if self.take(b']') {
            return Ok(CanonicalValue::Array(values));
        }
        loop {
            values.push(self.parse_value()?);
            if self.take(b']') {
                return Ok(CanonicalValue::Array(values));
            }
            self.expect(b',')?;
        }
    }

    fn parse_object(&mut self) -> Result<CanonicalValue, CanonicalJsonError> {
        self.offset += 1;
        let mut values = BTreeMap::new();
        let mut previous: Option<String> = None;
        if self.take(b'}') {
            return Ok(CanonicalValue::Object(values));
        }
        loop {
            let key = self.parse_string()?;
            if previous
                .as_ref()
                .is_some_and(|previous| previous.as_bytes() >= key.as_bytes())
            {
                return Err(CanonicalJsonError::new(
                    "CanonicalJsonV1 object keys must be unique and byte-sorted",
                ));
            }
            previous = Some(key.clone());
            self.expect(b':')?;
            values.insert(key, self.parse_value()?);
            if self.take(b'}') {
                return Ok(CanonicalValue::Object(values));
            }
            self.expect(b',')?;
        }
    }

    fn parse_string(&mut self) -> Result<String, CanonicalJsonError> {
        self.expect(b'"')?;
        let mut output = String::new();
        loop {
            match self.peek() {
                Some(b'"') => {
                    self.offset += 1;
                    return Ok(output);
                }
                Some(b'\\') => {
                    self.offset += 1;
                    match self.peek() {
                        Some(b'"') => {
                            self.offset += 1;
                            output.push('"');
                        }
                        Some(b'\\') => {
                            self.offset += 1;
                            output.push('\\');
                        }
                        Some(b'u') => {
                            self.offset += 1;
                            let escape = self
                                .input
                                .as_bytes()
                                .get(self.offset..self.offset + 4)
                                .ok_or_else(|| {
                                    CanonicalJsonError::new("truncated Unicode escape")
                                })?;
                            if escape[0] != b'0'
                                || escape[1] != b'0'
                                || !is_lower_hex(escape[2])
                                || !is_lower_hex(escape[3])
                            {
                                return Err(CanonicalJsonError::new(
                                    "CanonicalJsonV1 permits only lowercase \\u00xx control escapes",
                                ));
                            }
                            let value = (hex_value(escape[2]) << 4) | hex_value(escape[3]);
                            if value > 0x1f {
                                return Err(CanonicalJsonError::new(
                                    "CanonicalJsonV1 does not escape non-control Unicode",
                                ));
                            }
                            output.push(char::from(value));
                            self.offset += 4;
                        }
                        _ => {
                            return Err(CanonicalJsonError::new(
                                "CanonicalJsonV1 rejects alternate string escapes",
                            ))
                        }
                    }
                }
                Some(byte) if byte < 0x20 => {
                    return Err(CanonicalJsonError::new(
                        "CanonicalJsonV1 requires escaped control characters",
                    ))
                }
                Some(_) => {
                    let character = self.input[self.offset..]
                        .chars()
                        .next()
                        .ok_or_else(|| CanonicalJsonError::new("unterminated string"))?;
                    self.offset += character.len_utf8();
                    output.push(character);
                }
                None => return Err(CanonicalJsonError::new("unterminated string")),
            }
        }
    }

    fn parse_integer(&mut self) -> Result<CanonicalValue, CanonicalJsonError> {
        let start = self.offset;
        let negative = self.take(b'-');
        let first = self
            .peek()
            .ok_or_else(|| CanonicalJsonError::new("truncated integer"))?;
        if !first.is_ascii_digit() {
            return Err(CanonicalJsonError::new("invalid integer"));
        }
        self.offset += 1;
        if first == b'0' && self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            return Err(CanonicalJsonError::new(
                "CanonicalJsonV1 rejects leading zeroes",
            ));
        }
        while self.peek().is_some_and(|byte| byte.is_ascii_digit()) {
            self.offset += 1;
        }
        let raw = &self.input[start..self.offset];
        if raw == "-0" {
            return Err(CanonicalJsonError::new(
                "CanonicalJsonV1 rejects negative zero",
            ));
        }
        if negative {
            raw.parse::<i64>()
                .map(CanonicalValue::I64)
                .map_err(|_| CanonicalJsonError::new("signed integer exceeds V1 range"))
        } else {
            raw.parse::<u64>()
                .map(CanonicalValue::U64)
                .map_err(|_| CanonicalJsonError::new("unsigned integer exceeds V1 range"))
        }
    }

    fn peek(&self) -> Option<u8> {
        self.input.as_bytes().get(self.offset).copied()
    }

    fn take(&mut self, expected: u8) -> bool {
        if self.peek() == Some(expected) {
            self.offset += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, expected: u8) -> Result<(), CanonicalJsonError> {
        if self.take(expected) {
            Ok(())
        } else {
            Err(CanonicalJsonError::new(format!(
                "expected CanonicalJsonV1 byte {}",
                char::from(expected)
            )))
        }
    }
}

fn is_lower_hex(byte: u8) -> bool {
    byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)
}

fn hex_value(byte: u8) -> u8 {
    match byte {
        b'0'..=b'9' => byte - b'0',
        b'a'..=b'f' => byte - b'a' + 10,
        _ => unreachable!("validated lowercase hex"),
    }
}

impl<'de> serde::Deserializer<'de> for CanonicalValue {
    type Error = CanonicalJsonError;

    fn deserialize_any<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            CanonicalValue::Null => visitor.visit_unit(),
            CanonicalValue::Bool(value) => visitor.visit_bool(value),
            CanonicalValue::I64(value) => visitor.visit_i64(value),
            CanonicalValue::U64(value) => visitor.visit_u64(value),
            CanonicalValue::String(value) => visitor.visit_string(value),
            CanonicalValue::Array(values) => visitor.visit_seq(CanonicalSeqAccess {
                values: values.into_iter(),
            }),
            CanonicalValue::Object(values) => visitor.visit_map(CanonicalMapAccess {
                values: values.into_iter(),
                next_value: None,
            }),
            CanonicalValue::EnumTag(_) => {
                Err(CanonicalJsonError::new("unbound enum tag reached decoder"))
            }
        }
    }

    fn deserialize_option<V: Visitor<'de>>(self, visitor: V) -> Result<V::Value, Self::Error> {
        match self {
            CanonicalValue::Null => visitor.visit_none(),
            value => visitor.visit_some(value),
        }
    }

    fn deserialize_newtype_struct<V: Visitor<'de>>(
        self,
        _name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        visitor.visit_newtype_struct(self)
    }

    fn deserialize_f32<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(CanonicalJsonError::new(
            "CanonicalJsonV1 rejects floating-point values",
        ))
    }

    fn deserialize_f64<V: Visitor<'de>>(self, _visitor: V) -> Result<V::Value, Self::Error> {
        Err(CanonicalJsonError::new(
            "CanonicalJsonV1 rejects floating-point values",
        ))
    }

    fn deserialize_enum<V: Visitor<'de>>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error> {
        match self {
            CanonicalValue::String(variant) => visitor.visit_enum(variant.into_deserializer()),
            _ => Err(CanonicalJsonError::new(
                "CanonicalJsonV1 enum variant must be a string tag",
            )),
        }
    }

    serde::forward_to_deserialize_any! {
        bool i8 i16 i32 i64 i128 u8 u16 u32 u64 u128 char str string
        bytes byte_buf unit unit_struct seq tuple tuple_struct map struct
        identifier ignored_any
    }
}

struct CanonicalSeqAccess {
    values: std::vec::IntoIter<CanonicalValue>,
}

impl<'de> SeqAccess<'de> for CanonicalSeqAccess {
    type Error = CanonicalJsonError;

    fn next_element_seed<T: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: T,
    ) -> Result<Option<T::Value>, Self::Error> {
        self.values
            .next()
            .map(|value| seed.deserialize(value))
            .transpose()
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

struct CanonicalMapAccess {
    values: std::collections::btree_map::IntoIter<String, CanonicalValue>,
    next_value: Option<CanonicalValue>,
}

impl<'de> MapAccess<'de> for CanonicalMapAccess {
    type Error = CanonicalJsonError;

    fn next_key_seed<K: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: K,
    ) -> Result<Option<K::Value>, Self::Error> {
        let Some((key, value)) = self.values.next() else {
            return Ok(None);
        };
        self.next_value = Some(value);
        seed.deserialize(CanonicalValue::String(key)).map(Some)
    }

    fn next_value_seed<V: serde::de::DeserializeSeed<'de>>(
        &mut self,
        seed: V,
    ) -> Result<V::Value, Self::Error> {
        seed.deserialize(
            self.next_value
                .take()
                .ok_or_else(|| CanonicalJsonError::new("object value missing"))?,
        )
    }

    fn size_hint(&self) -> Option<usize> {
        Some(self.values.len())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::Deserialize;

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(deny_unknown_fields)]
    struct Fixture {
        z_last: u64,
        a_first: String,
        optional: Option<String>,
        signed: i64,
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(tag = "kind", content = "value")]
    enum FixtureEnum {
        Unit,
        Fields { count: u32, note: Option<String> },
    }

    #[derive(Debug, Deserialize, Eq, PartialEq, Serialize)]
    enum ForbiddenSerdeEnum {
        Unit,
        Newtype(u8),
    }

    #[test]
    fn canonical_json_v1_emits_exact_sorted_bytes_and_explicit_null() {
        let fixture = Fixture {
            z_last: u64::MAX,
            a_first: "quote=\" slash=\\ control=\n snow=☃".to_string(),
            optional: None,
            signed: i64::MIN,
        };

        let bytes = to_vec(&fixture).expect("canonical fixture");

        assert_eq!(
            String::from_utf8(bytes.clone()).unwrap(),
            r#"{"a_first":"quote=\" slash=\\ control=\u000a snow=☃","optional":null,"signed":-9223372036854775808,"z_last":18446744073709551615}"#
        );
        assert_eq!(from_slice::<Fixture>(&bytes).unwrap(), fixture);
    }

    #[test]
    fn canonical_json_v1_uses_closed_kind_value_enum_shape() {
        assert_eq!(to_vec(&FixtureEnum::Unit).unwrap(), br#"{"kind":"Unit"}"#);
        assert_eq!(
            to_vec(&FixtureEnum::Fields {
                count: 7,
                note: None,
            })
            .unwrap(),
            br#"{"kind":"Fields","value":{"count":7,"note":null}}"#
        );
        assert_eq!(
            from_slice::<FixtureEnum>(br#"{"kind":"Fields","value":{"count":7,"note":null}}"#)
                .unwrap(),
            FixtureEnum::Fields {
                count: 7,
                note: None,
            }
        );
    }

    #[test]
    fn canonical_json_v1_rejects_bare_and_externally_tagged_enum_shapes() {
        assert!(to_vec(&ForbiddenSerdeEnum::Unit).is_err());
        assert!(to_vec(&ForbiddenSerdeEnum::Newtype(1)).is_err());
        assert!(from_slice::<ForbiddenSerdeEnum>(br#""Unit""#).is_err());
        assert!(from_slice::<ForbiddenSerdeEnum>(br#"{"Newtype":1}"#).is_err());
    }

    #[test]
    fn canonical_json_v1_rejects_noncanonical_and_ambiguous_inputs() {
        let invalid: &[&[u8]] = &[
            br#" {"a_first":"x","optional":null,"signed":0,"z_last":1}"#,
            br#"{"z_last":1,"signed":0,"optional":null,"a_first":"x"}"#,
            br#"{"a_first":"x","a_first":"y","optional":null,"signed":0,"z_last":1}"#,
            br#"{"a_first":"x\n","optional":null,"signed":0,"z_last":1}"#,
            br#"{"a_first":"\u0020","optional":null,"signed":0,"z_last":1}"#,
            br#"{"a_first":"x","optional":null,"signed":-0,"z_last":1}"#,
            br#"{"a_first":"x","optional":null,"signed":00,"z_last":1}"#,
            br#"{"a_first":"x","extra":true,"optional":null,"signed":0,"z_last":1}"#,
            &[0xff],
        ];
        for bytes in invalid {
            assert!(from_slice::<Fixture>(bytes).is_err(), "accepted {bytes:?}");
        }

        assert!(from_slice::<Fixture>(br#"{"a_first":"x","signed":0,"z_last":1}"#,).is_err());
        assert!(from_slice::<FixtureEnum>(br#"{"kind":"Fields","value":{"count":7}}"#).is_err());
    }

    #[test]
    fn canonical_json_v1_rejects_float_and_raw_byte_serialization() {
        assert!(to_vec(&1.5_f64).is_err());
        assert!(to_vec(&serde_bytes_fixture::RawBytes(&[1, 2, 3])).is_err());
        assert!(from_slice::<f32>(b"1").is_err());
        assert!(from_slice::<f64>(b"1").is_err());
    }

    #[test]
    fn canonical_json_v1_enforces_each_declared_integer_range() {
        assert_eq!(from_slice::<u8>(b"255").unwrap(), u8::MAX);
        assert!(from_slice::<u8>(b"256").is_err());
        assert_eq!(from_slice::<i8>(b"-128").unwrap(), i8::MIN);
        assert!(from_slice::<i8>(b"-129").is_err());
        assert_eq!(
            from_slice::<u64>(b"18446744073709551615").unwrap(),
            u64::MAX
        );
        assert!(from_slice::<u64>(b"18446744073709551616").is_err());
        assert!(to_vec(&(i64::MAX as i128 + 1)).is_err());
        assert!(to_vec(&(u64::MAX as u128 + 1)).is_err());
    }

    mod serde_bytes_fixture {
        use serde::Serialize;

        pub(super) struct RawBytes<'a>(pub(super) &'a [u8]);

        impl Serialize for RawBytes<'_> {
            fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
                serializer.serialize_bytes(self.0)
            }
        }
    }
}
