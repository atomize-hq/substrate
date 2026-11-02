use std::collections::VecDeque;
use std::future::Future;
use std::pin::Pin;
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Mutex,
};
use std::task::{Context, Poll};
use std::time::Duration;

use futures_core::Stream;
use futures_util::{task::noop_waker, StreamExt};
use tokio::sync::{mpsc, oneshot};

use super::super::test_support::{
    success_exit_status, toy_kind, ToyAdapter, ToyBackendError, ToyCompletion, ToyEvent,
};
use super::super::{contract, DEFAULT_EVENT_CHANNEL_CAPACITY};
use super::*;
use crate::AgentWrapperEventKind;

mod cancel_signal;
mod cancellation_integration;
mod completion_sender;
mod control_entrypoint;
mod pump_backend_events;
mod pump_backend_events_with_cancel;
mod spawn_error_disposition;
mod startup_cancellation_precedence;
mod support;

mod settlement_custody;
