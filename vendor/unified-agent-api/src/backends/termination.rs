use std::sync::{
    atomic::{AtomicBool, Ordering},
    Mutex,
};

pub(crate) trait TerminationHandle: Send + Sync + 'static {
    fn request_termination(&self);
}

#[derive(Debug)]
pub(crate) struct TerminationState<H: TerminationHandle> {
    requested: AtomicBool,
    handle: Mutex<Option<H>>,
}

impl<H: TerminationHandle> TerminationState<H> {
    pub(crate) fn new() -> Self {
        Self {
            requested: AtomicBool::new(false),
            handle: Mutex::new(None),
        }
    }

    pub(crate) fn request(&self) {
        self.requested.store(true, Ordering::SeqCst);

        let guard = match self.handle.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        if let Some(handle) = guard.as_ref() {
            handle.request_termination();
        }
    }

    pub(crate) fn set_handle(&self, handle: H) {
        let mut guard = match self.handle.lock() {
            Ok(guard) => guard,
            Err(poisoned) => poisoned.into_inner(),
        };

        *guard = Some(handle);

        if self.requested.load(Ordering::SeqCst) {
            if let Some(handle) = guard.as_ref() {
                handle.request_termination();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::Ordering;
    use std::sync::Arc;

    #[derive(Clone, Debug)]
    struct CountingHandle {
        calls: Arc<std::sync::atomic::AtomicUsize>,
    }

    impl TerminationHandle for CountingHandle {
        fn request_termination(&self) {
            self.calls.fetch_add(1, Ordering::SeqCst);
        }
    }

    #[test]
    fn request_is_idempotent_and_safe_across_late_handle_install() {
        let state = TerminationState::<CountingHandle>::new();

        state.request();
        state.request();

        let calls = Arc::new(std::sync::atomic::AtomicUsize::new(0));
        state.set_handle(CountingHandle {
            calls: calls.clone(),
        });
        assert_eq!(
            calls.load(Ordering::SeqCst),
            1,
            "late handle install should observe prior termination request"
        );

        state.request();
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }
}
