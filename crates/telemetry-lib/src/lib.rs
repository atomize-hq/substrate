#![allow(non_camel_case_types)]

use anyhow::{anyhow, Context, Result};
use chrono::Utc;
use lazy_static::lazy_static;
use serde::Serialize;
use std::ffi::CStr;
use std::fs::OpenOptions;
use std::io::Write;
use std::os::raw::c_char;
use std::sync::Mutex;
use std::time::Instant;

mod correlation;
#[cfg(unix)]
mod exec;
#[cfg(unix)]
mod file;
#[cfg(unix)]
mod network;

pub use correlation::*;
#[cfg(unix)]
pub use exec::*;
#[cfg(unix)]
#[allow(unused_imports)]
pub use file::*;
#[cfg(unix)]
pub use network::*;

lazy_static! {
    static ref INIT: Mutex<bool> = Mutex::new(false);
    static ref START_TIME: Instant = Instant::now();
    static ref TRACE_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);
}

#[derive(Debug, Serialize)]
struct TelemetryEvent {
    ts: String,
    event_type: String,
    session_id: String,
    span_id: String,
    parent_span: Option<String>,
    component: String,
    syscall: String,
    args: Vec<String>,
    result: Option<String>,
    elapsed_us: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
}

fn init_telemetry() {
    if let Err(err) = init_telemetry_inner() {
        eprintln!("[telemetry] initialization failed: {err}");
    }
}

fn init_telemetry_inner() -> Result<()> {
    let mut initialized = INIT
        .lock()
        .map_err(|e| anyhow!("Failed to acquire telemetry init lock: {}", e))?;
    if *initialized {
        return Ok(());
    }

    let session = get_session_info_result()?;
    let trace_path = session.trace_log.clone();

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&trace_path)
        .with_context(|| format!("failed to open trace log {}", trace_path))?;

    let mut trace_file = TRACE_FILE
        .lock()
        .map_err(|e| anyhow!("Failed to acquire trace file lock: {}", e))?;
    *trace_file = Some(file);

    *initialized = true;

    #[cfg(feature = "debug")]
    eprintln!(
        "[telemetry] Initialized - session: {}, trace: {}",
        session.session_id, trace_path
    );

    Ok(())
}

pub fn log_syscall(
    syscall: &str,
    args: Vec<String>,
    result: Option<String>,
    error: Option<String>,
    elapsed_us: u64,
) {
    if let Err(err) = log_syscall_inner(syscall, args, result, error, elapsed_us) {
        eprintln!("[telemetry] failed to log syscall {syscall}: {err}");
    }
}

fn log_syscall_inner(
    syscall: &str,
    args: Vec<String>,
    result: Option<String>,
    error: Option<String>,
    elapsed_us: u64,
) -> Result<()> {
    init_telemetry_inner()?;

    let session = get_session_info_result()?;
    let event = TelemetryEvent {
        ts: Utc::now().to_rfc3339(),
        event_type: "syscall".to_string(),
        session_id: session.session_id,
        span_id: generate_span_id(),
        parent_span: session.parent_span_id,
        component: "telemetry".to_string(),
        syscall: syscall.to_string(),
        args,
        result,
        elapsed_us,
        error,
    };

    let json = serde_json::to_string(&event).context("failed to serialize telemetry event")?;
    let mut file = TRACE_FILE
        .lock()
        .map_err(|e| anyhow!("Failed to acquire trace file lock: {}", e))?;
    if let Some(ref mut f) = *file {
        writeln!(f, "{}", json).context("failed to write telemetry event")?;
        f.flush().context("failed to flush telemetry event")?;
    }

    Ok(())
}

/// Convert a C string pointer to a Rust `String`.
///
/// # Safety
/// - `ptr` must be either null or a valid, null-terminated C string pointer.
pub unsafe fn c_str_to_string(ptr: *const c_char) -> String {
    if ptr.is_null() {
        String::new()
    } else {
        CStr::from_ptr(ptr).to_string_lossy().into_owned()
    }
}

/// Convert a null-terminated array of C string pointers to a `Vec<String>`.
///
/// # Safety
/// - `ptr` must be either null or a valid, null-terminated array of C string pointers.
pub unsafe fn c_str_array_to_vec(ptr: *const *const c_char) -> Vec<String> {
    let mut vec = Vec::new();
    if !ptr.is_null() {
        let mut i = 0;
        loop {
            let str_ptr = *ptr.offset(i);
            if str_ptr.is_null() {
                break;
            }
            vec.push(c_str_to_string(str_ptr));
            i += 1;
        }
    }
    vec
}

#[cfg(target_os = "linux")]
#[no_mangle]
#[link_section = ".init_array"]
#[used]
static INIT_ARRAY: extern "C" fn() = {
    extern "C" fn init_constructor() {
        init_telemetry();
    }
    init_constructor
};

#[cfg(target_os = "macos")]
#[no_mangle]
#[link_section = "__DATA,__mod_init_func"]
#[used]
static INIT_ARRAY: extern "C" fn() = {
    extern "C" fn init_constructor() {
        init_telemetry();
    }
    init_constructor
};
