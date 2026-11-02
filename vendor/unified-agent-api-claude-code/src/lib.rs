#![forbid(unsafe_code)]
//! Async helper around the Claude Code CLI (`claude`) focused on the headless `--print` flow.
//!
//! This crate intentionally does **not** attempt to wrap interactive default mode (no `--print`)
//! as a parity target. It shells out to a locally installed/pinned `claude` binary.

use std::{
    future::Future,
    pin::Pin,
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
};

use futures_core::Stream;
use tokio::sync::Notify;

mod builder;
mod cli;
mod client;
mod commands;
mod error;
mod home;
mod process;
mod process_custody;
pub use process_custody::{ClaudeProcessCustody, ClaudeSettlementError};
mod stream_json;
pub mod wrapper_coverage_manifest;

pub use builder::ClaudeClientBuilder;
pub use client::ClaudeClient;
pub use client::ClaudeSetupTokenSession;
pub use commands::command::ClaudeCommandRequest;
pub use commands::doctor::ClaudeDoctorRequest;
pub use commands::mcp::{
    McpAddFromClaudeDesktopRequest, McpAddJsonRequest, McpAddRequest, McpGetRequest,
    McpRemoveRequest, McpScope, McpServeRequest, McpTransport,
};
pub use commands::plugin::{
    PluginDisableRequest, PluginEnableRequest, PluginInstallRequest, PluginListRequest,
    PluginManifestMarketplaceRequest, PluginManifestRequest, PluginMarketplaceAddRequest,
    PluginMarketplaceListRequest, PluginMarketplaceRemoveRequest, PluginMarketplaceRepoRequest,
    PluginMarketplaceRequest, PluginMarketplaceUpdateRequest, PluginRequest,
    PluginUninstallRequest, PluginUpdateRequest, PluginValidateRequest,
};
pub use commands::print::{
    ClaudeChromeMode, ClaudeInputFormat, ClaudeOutputFormat, ClaudePrintRequest,
};
pub use commands::setup_token::ClaudeSetupTokenRequest;
pub use commands::update::ClaudeUpdateRequest;
pub use error::{ClaudeCodeError, StreamJsonLineError};
pub use home::{
    ClaudeHomeLayout, ClaudeHomeSeedLevel, ClaudeHomeSeedOutcome, ClaudeHomeSeedRequest,
};
pub use stream_json::{parse_stream_json_lines, StreamJsonLine, StreamJsonLineOutcome};
pub use stream_json::{
    ClaudeStreamEvent, ClaudeStreamJsonErrorCode, ClaudeStreamJsonEvent,
    ClaudeStreamJsonParseError, ClaudeStreamJsonParser,
};

pub use process::CommandOutput;

pub type DynClaudeStreamJsonEventStream =
    Pin<Box<dyn Stream<Item = Result<ClaudeStreamJsonEvent, ClaudeStreamJsonParseError>> + Send>>;

pub type DynClaudeStreamJsonCompletion =
    Pin<Box<dyn Future<Output = Result<std::process::ExitStatus, ClaudeCodeError>> + Send>>;

#[derive(Clone)]
pub struct ClaudeTerminationHandle {
    inner: Arc<ClaudeTerminationInner>,
    custody: Option<ClaudeProcessCustody>,
}

#[derive(Debug)]
struct ClaudeTerminationInner {
    requested: AtomicBool,
    notify: Notify,
}

impl ClaudeTerminationHandle {
    fn new() -> Self {
        Self {
            custody: None,
            inner: Arc::new(ClaudeTerminationInner {
                requested: AtomicBool::new(false),
                notify: Notify::new(),
            }),
        }
    }

    pub(crate) fn with_custody(custody: ClaudeProcessCustody) -> Self {
        Self {
            custody: Some(custody),
            ..Self::new()
        }
    }

    pub fn request_termination(&self) {
        if let Some(custody) = &self.custody {
            custody.request_termination();
        }
        if !self.inner.requested.swap(true, Ordering::SeqCst) {
            self.inner.notify.notify_waiters();
        }
    }

    fn is_requested(&self) -> bool {
        self.inner.requested.load(Ordering::SeqCst)
    }
}

impl std::fmt::Debug for ClaudeTerminationHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudeTerminationHandle")
            .field("requested", &self.is_requested())
            .finish()
    }
}

pub struct ClaudePrintStreamJsonHandle {
    pub events: DynClaudeStreamJsonEventStream,
    pub completion: DynClaudeStreamJsonCompletion,
}

impl std::fmt::Debug for ClaudePrintStreamJsonHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudePrintStreamJsonHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .finish()
    }
}

pub struct ClaudePrintStreamJsonControlHandle {
    pub events: DynClaudeStreamJsonEventStream,
    pub completion: DynClaudeStreamJsonCompletion,
    pub termination: ClaudeTerminationHandle,
}

impl std::fmt::Debug for ClaudePrintStreamJsonControlHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClaudePrintStreamJsonControlHandle")
            .field("events", &"<stream>")
            .field("completion", &"<future>")
            .field("termination", &self.termination)
            .finish()
    }
}
