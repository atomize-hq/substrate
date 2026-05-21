//! Legacy Work Lift v1 dotted-path compatibility helpers.
//!
//! This module intentionally quarantines dotted field path logic at the
//! score-app edge. The engine remains JSON-Pointer-native.

#![allow(dead_code)]

use crate::kernel::{JsonPointer, KernelError, KernelResult};

/// Converts a dotted Lift v1 field path into a canonical JSON Pointer.
pub(crate) fn dotted_to_json_pointer(input: &str) -> KernelResult<JsonPointer> {
    if input.is_empty() {
        return Err(invalid_pointer(input));
    }

    let mut pointer = JsonPointer::root();
    for segment in input.split('.') {
        if segment.is_empty() || !is_v1_segment(segment) {
            return Err(invalid_pointer(input));
        }
        pointer = pointer.push_token(segment);
    }

    Ok(pointer)
}

/// Converts a canonical JSON Pointer into a dotted Lift v1 field path.
pub(crate) fn json_pointer_to_dotted(pointer: &JsonPointer) -> KernelResult<String> {
    let raw = pointer.as_str();
    if raw.is_empty() {
        return Err(invalid_pointer(raw));
    }

    let mut segments = Vec::new();
    for token in raw[1..].split('/') {
        if token.is_empty() {
            return Err(invalid_pointer(raw));
        }

        let segment = unescape_token(token, raw)?;
        if !is_v1_segment(&segment) {
            return Err(invalid_pointer(raw));
        }
        segments.push(segment);
    }

    Ok(segments.join("."))
}

fn is_v1_segment(segment: &str) -> bool {
    let mut chars = segment.chars();
    let Some(first) = chars.next() else {
        return false;
    };

    if !(first.is_ascii_lowercase() || first == '_') {
        return false;
    }

    chars.all(|ch| ch.is_ascii_lowercase() || ch.is_ascii_digit() || ch == '_')
}

fn unescape_token(token: &str, raw: &str) -> KernelResult<String> {
    let mut result = String::with_capacity(token.len());
    let mut chars = token.chars();

    while let Some(ch) = chars.next() {
        if ch != '~' {
            result.push(ch);
            continue;
        }

        match chars.next() {
            Some('0') => result.push('~'),
            Some('1') => result.push('/'),
            _ => return Err(invalid_pointer(raw)),
        }
    }

    Ok(result)
}

fn invalid_pointer(input: &str) -> KernelError {
    KernelError::InvalidJsonPointer {
        input: input.to_owned(),
    }
}

#[cfg(all(test, feature = "compat-v1"))]
mod tests {
    use super::{dotted_to_json_pointer, json_pointer_to_dotted};
    use crate::kernel::JsonPointer;

    #[test]
    fn dotted_examples_convert_to_pointer_form() {
        assert_eq!(
            dotted_to_json_pointer("touch.crates_touched")
                .expect("pointer")
                .as_str(),
            "/touch/crates_touched"
        );
        assert_eq!(
            dotted_to_json_pointer("contract.behavior_deltas")
                .expect("pointer")
                .as_str(),
            "/contract/behavior_deltas"
        );
        assert_eq!(
            dotted_to_json_pointer("risk.unknowns_high")
                .expect("pointer")
                .as_str(),
            "/risk/unknowns_high"
        );
    }

    #[test]
    fn dotted_paths_reject_invalid_segments() {
        assert!(dotted_to_json_pointer("").is_err());
        assert!(dotted_to_json_pointer("touch..crates_touched").is_err());
        assert!(dotted_to_json_pointer(".touch").is_err());
        assert!(dotted_to_json_pointer("missing-inputs.value").is_err());
        assert!(dotted_to_json_pointer("0leading.segment").is_err());
    }

    #[test]
    fn pointer_examples_convert_to_dotted_form() {
        assert_eq!(
            json_pointer_to_dotted(&JsonPointer::parse("/touch/crates_touched").expect("pointer"))
                .expect("dotted"),
            "touch.crates_touched"
        );
        assert_eq!(
            json_pointer_to_dotted(
                &JsonPointer::parse("/contract/behavior_deltas").expect("pointer")
            )
            .expect("dotted"),
            "contract.behavior_deltas"
        );
        assert_eq!(
            json_pointer_to_dotted(&JsonPointer::parse("/risk/unknowns_high").expect("pointer"))
                .expect("dotted"),
            "risk.unknowns_high"
        );
    }

    #[test]
    fn pointers_reject_root_and_non_identifier_tokens() {
        assert!(json_pointer_to_dotted(&JsonPointer::root()).is_err());
        assert!(json_pointer_to_dotted(
            &JsonPointer::parse("/touch//crates_touched").expect("pointer")
        )
        .is_err());
        assert!(JsonPointer::parse("/touch/~2bad").is_err());
        assert!(
            json_pointer_to_dotted(&JsonPointer::parse("/touch/has~1slash").expect("pointer"))
                .is_err()
        );
        assert!(
            json_pointer_to_dotted(&JsonPointer::parse("/touch/has~0tilde").expect("pointer"))
                .is_err()
        );
    }

    #[test]
    fn examples_round_trip_cleanly() {
        for dotted in [
            "touch.crates_touched",
            "contract.behavior_deltas",
            "risk.unknowns_high",
        ] {
            let pointer = dotted_to_json_pointer(dotted).expect("pointer");
            assert_eq!(json_pointer_to_dotted(&pointer).expect("dotted"), dotted);
        }
    }
}
