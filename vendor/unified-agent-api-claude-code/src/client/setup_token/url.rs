#[derive(Debug, Default)]
pub(super) struct UrlCapture {
    buffer: String,
    found: Option<String>,
}

impl UrlCapture {
    pub(super) fn push_text(&mut self, chunk: &str) -> Option<String> {
        if self.found.is_some() {
            return None;
        }

        if self.buffer.len() > 64 * 1024 {
            // Avoid unbounded growth in the unlikely event the command is chatty before printing
            // the URL.
            self.buffer = self
                .buffer
                .chars()
                .rev()
                .take(16 * 1024)
                .collect::<String>()
                .chars()
                .rev()
                .collect::<String>();
        }

        self.buffer.push_str(chunk);
        if let Some(url) = extract_oauth_url(&self.buffer) {
            self.found = Some(url.clone());
            return Some(url);
        }
        None
    }
}

pub(super) fn extract_oauth_url(text: &str) -> Option<String> {
    let cleaned = strip_ansi(text);
    let start = cleaned.find("https://claude.ai/oauth/authorize?")?;
    let tail = &cleaned[start..];

    let mut stop = tail.len();
    if let Some(idx) = tail.find("\n\n") {
        stop = stop.min(idx);
    }
    if let Some(idx) = tail.find("Paste code") {
        stop = stop.min(idx);
    }
    if let Some(idx) = tail.find("Paste") {
        stop = stop.min(idx);
    }

    let raw = &tail[..stop];
    let flattened: String = raw.split_whitespace().collect();
    flattened
        .starts_with("https://claude.ai/oauth/authorize?")
        .then_some(flattened)
}

fn strip_ansi(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut out: Vec<u8> = Vec::with_capacity(bytes.len());
    let mut i = 0;

    while i < bytes.len() {
        if bytes[i] != 0x1b {
            out.push(bytes[i]);
            i += 1;
            continue;
        }

        // ESC sequence.
        i += 1;
        if i >= bytes.len() {
            break;
        }

        match bytes[i] {
            b'[' => {
                // CSI: ESC [ ... <final>
                i += 1;
                while i < bytes.len() {
                    let b = bytes[i];
                    i += 1;
                    if (0x40..=0x7e).contains(&b) {
                        break;
                    }
                }
            }
            b']' => {
                // OSC: ESC ] ... BEL or ESC \
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 0x07 {
                        i += 1;
                        break;
                    }
                    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            b'P' | b'^' | b'_' => {
                // DCS / PM / APC: ESC <X> ... ESC \
                i += 1;
                while i < bytes.len() {
                    if bytes[i] == 0x1b && i + 1 < bytes.len() && bytes[i + 1] == b'\\' {
                        i += 2;
                        break;
                    }
                    i += 1;
                }
            }
            _ => {
                // Other 2-byte escape; drop it.
                i += 1;
            }
        }
    }

    String::from_utf8_lossy(&out).into_owned()
}

#[cfg(test)]
mod tests {
    use super::{extract_oauth_url, strip_ansi};

    #[test]
    fn extracts_wrapped_url_from_setup_token_output() {
        let text = r#"
Browser didn't open? Use the url below to sign in (c to copy)

https://claude.ai/oauth/authorize?code=true&client_id=abc&response_type=c
ode&redirect_uri=https%3A%2F%2Fplatform.claude.com%2Foauth%2Fcode%2Fcallback&scope=user%3Ainference

Paste code here if prompted >
"#;

        let url = extract_oauth_url(text).expect("url");
        assert!(url.starts_with("https://claude.ai/oauth/authorize?"));
        assert!(url.contains("client_id=abc"));
        assert!(url.contains("response_type=code"));
    }

    #[test]
    fn strip_ansi_removes_common_sequences() {
        assert_eq!(strip_ansi("a\x1b[2Jb"), "ab");
        assert_eq!(strip_ansi("a\x1b]0;title\x07b"), "ab");
        assert_eq!(strip_ansi("a\x1b]8;;https://x\x1b\\b"), "ab");
    }
}
