//! Byte-span primitives.

use serde::{Deserialize, Serialize};

use crate::kernel::error::{KernelError, KernelResult};

/// Half-open byte interval `[start_byte, end_byte)`.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(try_from = "ByteSpanRepr")]
pub struct ByteSpan {
    /// Inclusive start byte offset.
    pub start_byte: u64,
    /// Exclusive end byte offset.
    pub end_byte: u64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize, Deserialize)]
struct ByteSpanRepr {
    start_byte: u64,
    end_byte: u64,
}

impl ByteSpan {
    /// Creates a validated half-open byte span.
    pub fn new(start_byte: u64, end_byte: u64) -> KernelResult<Self> {
        if start_byte > end_byte {
            return Err(KernelError::InvalidByteSpan {
                start_byte,
                end_byte,
            });
        }

        Ok(Self {
            start_byte,
            end_byte,
        })
    }

    /// Returns the byte length of the span.
    pub fn len(&self) -> u64 {
        self.end_byte - self.start_byte
    }

    /// Returns true when the span covers zero bytes.
    pub fn is_empty(&self) -> bool {
        self.start_byte == self.end_byte
    }
}

impl TryFrom<ByteSpanRepr> for ByteSpan {
    type Error = KernelError;

    fn try_from(value: ByteSpanRepr) -> KernelResult<Self> {
        Self::new(value.start_byte, value.end_byte)
    }
}

#[cfg(test)]
mod tests {
    use super::ByteSpan;
    use crate::kernel::error::KernelError;

    #[test]
    fn accepts_empty_and_non_empty_spans() {
        let empty = ByteSpan::new(4, 4).expect("empty span should be valid");
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let span = ByteSpan::new(4, 9).expect("span should be valid");
        assert!(!span.is_empty());
        assert_eq!(span.len(), 5);
    }

    #[test]
    fn rejects_reversed_spans() {
        let error = ByteSpan::new(9, 4).expect_err("span should fail");
        assert_eq!(
            error,
            KernelError::InvalidByteSpan {
                start_byte: 9,
                end_byte: 4,
            }
        );
    }

    #[test]
    fn serde_rejects_reversed_spans() {
        assert!(serde_json::from_str::<ByteSpan>(r#"{"start_byte":9,"end_byte":4}"#).is_err());
    }
}
