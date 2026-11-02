#[path = "mcp_management_v1/capabilities.rs"]
mod capabilities;
#[cfg(feature = "claude_code")]
#[path = "mcp_management_v1/claude_capabilities.rs"]
mod claude_capabilities;
#[cfg(feature = "claude_code")]
#[path = "mcp_management_v1/claude_direct_backend.rs"]
mod claude_direct_backend;
#[cfg(feature = "claude_code")]
#[path = "mcp_management_v1/claude_failures.rs"]
mod claude_failures;
#[cfg(feature = "claude_code")]
#[path = "mcp_management_v1/claude_mapping.rs"]
mod claude_mapping;
#[cfg(feature = "claude_code")]
#[path = "mcp_management_v1/claude_support.rs"]
mod claude_support;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_direct_backend.rs"]
mod codex_direct_backend;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_failures.rs"]
mod codex_failures;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_read_ops.rs"]
mod codex_read_ops;
#[cfg(feature = "codex")]
#[path = "mcp_management_v1/codex_write_ops.rs"]
mod codex_write_ops;
#[path = "mcp_management_v1/support.rs"]
mod support;
#[path = "mcp_management_v1/support_smoke.rs"]
mod support_smoke;
