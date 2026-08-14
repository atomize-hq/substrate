use std::io::{Cursor, Error, ErrorKind, Read};

use substrate_r3_macos_finalizer::contract::FRAME_MAX_BYTES;
use substrate_r3_macos_finalizer::frame::{read_one_frame_to_eof, write_one_frame};

struct NeverEof {
    bytes: Cursor<Vec<u8>>,
}

impl Read for NeverEof {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let read = self.bytes.read(buffer)?;
        if read == 0 {
            Err(Error::new(
                ErrorKind::WouldBlock,
                "the peer has not sent request EOF",
            ))
        } else {
            Ok(read)
        }
    }
}

fn frame(body: &[u8]) -> Vec<u8> {
    let mut bytes = Vec::new();
    write_one_frame(&mut bytes, body).expect("write bounded test frame");
    bytes
}

#[test]
fn one_bounded_frame_followed_by_eof_is_accepted() {
    let body = br#"{"request":"only"}"#;
    let mut input = Cursor::new(frame(body));

    assert_eq!(read_one_frame_to_eof(&mut input).unwrap(), body);
}

#[test]
fn a_complete_frame_without_request_eof_is_rejected() {
    let mut input = NeverEof {
        bytes: Cursor::new(frame(b"request")),
    };

    let error = read_one_frame_to_eof(&mut input).unwrap_err();
    assert!(
        error.to_string().contains("require finalizer request EOF"),
        "unexpected missing-EOF error: {error:#}"
    );
}

#[test]
fn trailing_bytes_and_a_second_frame_are_rejected() {
    let mut trailing = frame(b"first");
    trailing.push(0);
    assert!(read_one_frame_to_eof(&mut Cursor::new(trailing)).is_err());

    let mut second = frame(b"first");
    second.extend(frame(b"second"));
    assert!(read_one_frame_to_eof(&mut Cursor::new(second)).is_err());
}

#[test]
fn oversize_and_empty_frames_are_rejected_before_body_allocation_or_write() {
    let oversize_prefix = ((FRAME_MAX_BYTES as u64) + 1).to_be_bytes();
    assert!(read_one_frame_to_eof(&mut Cursor::new(oversize_prefix)).is_err());

    assert!(write_one_frame(&mut Vec::new(), &[]).is_err());
    assert!(write_one_frame(&mut Vec::new(), &vec![0; FRAME_MAX_BYTES + 1]).is_err());
}

#[test]
fn truncated_prefix_and_body_are_rejected() {
    assert!(read_one_frame_to_eof(&mut Cursor::new(vec![0; 7])).is_err());

    let mut truncated_body = 2_u64.to_be_bytes().to_vec();
    truncated_body.push(b'x');
    assert!(read_one_frame_to_eof(&mut Cursor::new(truncated_body)).is_err());
}
