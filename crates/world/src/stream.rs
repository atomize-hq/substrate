//! Global stream sink used to forward incremental stdout/stderr chunks.

use std::sync::{Arc, Mutex, OnceLock};

/// Distinguish stdout vs stderr payloads when emitting chunks.
#[derive(Copy, Clone, Debug)]
pub enum StreamKind {
    Stdout,
    Stderr,
}

/// Trait implemented by consumers that wish to receive streamed process output.
pub trait StreamSink: Send + Sync + 'static {
    fn write(&self, kind: StreamKind, chunk: &[u8]);
}

static STREAM_SINK: OnceLock<Mutex<Option<Arc<dyn StreamSink>>>> = OnceLock::new();

fn sink_cell() -> &'static Mutex<Option<Arc<dyn StreamSink>>> {
    STREAM_SINK.get_or_init(|| Mutex::new(None))
}

/// RAII guard that clears the global stream sink when dropped.
pub struct StreamSinkGuard {
    cell: &'static Mutex<Option<Arc<dyn StreamSink>>>,
    cleared: bool,
}

impl Drop for StreamSinkGuard {
    fn drop(&mut self) {
        if !self.cleared {
            if let Ok(mut guard) = self.cell.lock() {
                guard.take();
            }
            self.cleared = true;
        }
    }
}

impl StreamSinkGuard {
    /// Explicitly clear the sink before the guard drops.
    pub fn clear(mut self) {
        if !self.cleared {
            if let Ok(mut guard) = self.cell.lock() {
                guard.take();
            }
            self.cleared = true;
        }
    }
}

/// Install a new global stream sink returning a guard that clears it on drop.
pub fn install_stream_sink(sink: Arc<dyn StreamSink>) -> StreamSinkGuard {
    let cell = sink_cell();
    if let Ok(mut guard) = cell.lock() {
        *guard = Some(sink);
    }
    StreamSinkGuard {
        cell,
        cleared: false,
    }
}

/// Emit a chunk to the currently installed sink (if any).
pub fn emit_chunk(kind: StreamKind, chunk: &[u8]) {
    if chunk.is_empty() {
        return;
    }
    let sink_opt = {
        let guard = sink_cell();
        guard.lock().ok().and_then(|g| g.as_ref().cloned())
    };
    if let Some(sink) = sink_opt {
        sink.write(kind, chunk);
    }
}
