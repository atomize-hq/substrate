use std::io::{ErrorKind, Read, Write};

use anyhow::{bail, Context, Result};

use crate::contract::{FRAME_MAX_BYTES, FRAME_PREFIX_BYTES};

/// Read exactly one big-endian u64 length-prefixed frame and require request EOF.
pub fn read_one_frame_to_eof<R: Read>(reader: &mut R) -> Result<Vec<u8>> {
    let mut prefix = [0_u8; FRAME_PREFIX_BYTES];
    reader
        .read_exact(&mut prefix)
        .context("read exact finalizer frame prefix")?;
    let claimed = u64::from_be_bytes(prefix);
    let length = usize::try_from(claimed).context("finalizer frame length exceeds usize")?;
    if length == 0 || length > FRAME_MAX_BYTES {
        bail!("finalizer frame length is outside the fixed bound")
    }
    let mut body = vec![0_u8; length];
    reader
        .read_exact(&mut body)
        .context("read exact finalizer frame body")?;
    let mut trailing = [0_u8; 1];
    loop {
        match reader.read(&mut trailing) {
            Ok(0) => break,
            Ok(_) => bail!("finalizer request has trailing bytes or a second frame"),
            Err(error) if error.kind() == ErrorKind::Interrupted => continue,
            Err(error) => return Err(error).context("require finalizer request EOF"),
        }
    }
    Ok(body)
}

/// Write exactly one big-endian u64 length-prefixed frame. The caller must then send EOF.
pub fn write_one_frame<W: Write>(writer: &mut W, body: &[u8]) -> Result<()> {
    if body.is_empty() || body.len() > FRAME_MAX_BYTES {
        bail!("finalizer response length is outside the fixed bound")
    }
    writer
        .write_all(&(body.len() as u64).to_be_bytes())
        .context("write exact finalizer frame prefix")?;
    writer
        .write_all(body)
        .context("write exact finalizer frame body")?;
    writer.flush().context("flush exact finalizer frame")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn exact_frame_and_eof_are_required() {
        let mut bytes = Vec::new();
        write_one_frame(&mut bytes, br#"{"a":1}"#).unwrap();
        assert_eq!(
            read_one_frame_to_eof(&mut Cursor::new(bytes)).unwrap(),
            br#"{"a":1}"#
        );
    }

    #[test]
    fn missing_eof_trailing_second_and_oversize_fail() {
        let mut trailing = Vec::new();
        write_one_frame(&mut trailing, b"x").unwrap();
        trailing.push(0);
        assert!(read_one_frame_to_eof(&mut Cursor::new(trailing)).is_err());

        let mut second = Vec::new();
        write_one_frame(&mut second, b"x").unwrap();
        write_one_frame(&mut second, b"y").unwrap();
        assert!(read_one_frame_to_eof(&mut Cursor::new(second)).is_err());

        let mut oversize = ((FRAME_MAX_BYTES as u64) + 1).to_be_bytes().to_vec();
        oversize.push(0);
        assert!(read_one_frame_to_eof(&mut Cursor::new(oversize)).is_err());
    }
}
