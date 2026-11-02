#[allow(dead_code)]
mod session_selectors;

#[cfg(any(feature = "codex", feature = "claude_code", test))]
mod termination;

#[cfg(any(
    feature = "codex",
    feature = "claude_code",
    feature = "aider",
    feature = "gemini_cli",
    feature = "opencode",
    test
))]
pub(crate) mod spawn_path;

#[cfg(test)]
pub(crate) mod test_support;

#[cfg(feature = "codex")]
pub mod codex;

#[cfg(feature = "claude_code")]
pub mod claude_code;

#[cfg(feature = "gemini_cli")]
pub mod gemini_cli;

#[cfg(feature = "aider")]
pub mod aider;

#[cfg(feature = "opencode")]
pub mod opencode;
