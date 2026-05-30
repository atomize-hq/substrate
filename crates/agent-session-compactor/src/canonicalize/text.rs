pub fn canonicalize_row_text(text: &str) -> (String, String) {
    let canonical_text = canonicalize_text(text);
    let text_hash_hex = stable_text_hash_hex(&canonical_text);
    (canonical_text, text_hash_hex)
}

pub fn canonicalize_text(raw: &str) -> String {
    let without_ansi = strip_ansi_sequences(raw);
    let normalized_newlines = without_ansi.replace("\r\n", "\n").replace('\r', "\n");
    let trimmed_lines = normalized_newlines
        .lines()
        .map(|line| line.trim_end_matches([' ', '\t']))
        .collect::<Vec<_>>()
        .join("\n");
    trimmed_lines.trim().to_string()
}

pub fn stable_text_hash_hex(canonical_text: &str) -> String {
    blake3::hash(canonical_text.as_bytes()).to_hex().to_string()
}

fn strip_ansi_sequences(raw: &str) -> String {
    let mut output = String::with_capacity(raw.len());
    let mut chars = raw.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch != '\u{1b}' {
            output.push(ch);
            continue;
        }

        match chars.peek().copied() {
            Some('[') => {
                chars.next();
                while let Some(next) = chars.next() {
                    if ('@'..='~').contains(&next) {
                        break;
                    }
                }
            }
            Some(']') => {
                chars.next();
                let mut saw_escape = false;
                while let Some(next) = chars.next() {
                    if next == '\u{7}' || (saw_escape && next == '\\') {
                        break;
                    }
                    saw_escape = next == '\u{1b}';
                }
            }
            _ => {}
        }
    }

    output
}
