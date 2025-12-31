use std::borrow::Cow;

pub(crate) fn parse_bool_flag(raw: &str) -> Option<bool> {
    match normalize_bool(raw)?.as_ref() {
        "true" | "1" | "yes" | "on" => Some(true),
        "false" | "0" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn normalize_bool(raw: &str) -> Option<Cow<'_, str>> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    Some(Cow::Owned(trimmed.to_ascii_lowercase()))
}
