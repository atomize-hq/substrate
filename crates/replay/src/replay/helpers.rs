/// Parse command string into command and args
pub fn parse_command(cmd_str: &str) -> (String, Vec<String>) {
    let parts: Vec<String> = cmd_str.split_whitespace().map(String::from).collect();

    if parts.is_empty() {
        return (String::new(), Vec::new());
    }

    let command = parts[0].clone();
    let args = parts[1..].to_vec();

    (command, args)
}

/// Check if world isolation backend is available.
pub fn world_isolation_available() -> bool {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let disabled = std::env::var("SUBSTRATE_WORLD")
            .map(|v| v.eq_ignore_ascii_case("disabled"))
            .unwrap_or(false)
            || std::env::var("SUBSTRATE_WORLD_ENABLED")
                .map(|v| v == "0")
                .unwrap_or(false);
        !disabled
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        false
    }
}

pub fn replay_verbose() -> bool {
    std::env::var("SUBSTRATE_REPLAY_VERBOSE").unwrap_or_default() == "1"
}
