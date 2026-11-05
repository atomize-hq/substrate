//! Owned observers for the actual attempts issued by one ordinary helper.
//! This is process lifetime custody, with no durable registry or effect grant.

use anyhow::Result;
use std::sync::Mutex;
use tokio::task::JoinHandle;

#[derive(Default)]
pub(super) struct OrdinaryWorldObserverTasksV1 {
    issued: Mutex<Vec<JoinHandle<()>>>,
}

impl OrdinaryWorldObserverTasksV1 {
    pub(super) fn retain_issued_world_observer(&self, task: JoinHandle<()>) {
        // Preserve custody on poison as well: a rejected lock must not silently
        // detach an already-issued observer. The vector remains memory-safe.
        self.issued
            .lock()
            .unwrap_or_else(|error| error.into_inner())
            .push(task);
    }

    /// Call only after local ingress and its accepted handlers have closed.
    /// Join this finite issued set, including one-off descendants recorded by
    /// those observers. Never poll a global work registry or grant another send.
    pub(super) async fn join_issued_world_observers(&self) -> Result<()> {
        let mut failures = Vec::new();
        loop {
            let tasks = {
                let mut issued = self
                    .issued
                    .lock()
                    .unwrap_or_else(|error| error.into_inner());
                std::mem::take(&mut *issued)
            };
            if tasks.is_empty() {
                break;
            }
            for task in tasks {
                if let Err(error) = task.await {
                    failures.push(error.to_string());
                }
            }
        }
        if !failures.is_empty() {
            anyhow::bail!(
                "ordinary World observer joins failed: {}",
                failures.join("; ")
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    };

    #[tokio::test]
    async fn test_issued_world_observer_lifetime_joins_after_receipt_and_includes_descendants() {
        let owner = Arc::new(OrdinaryWorldObserverTasksV1::default());
        let completed = Arc::new(AtomicBool::new(false));
        let (receipt_tx, receipt_rx) = tokio::sync::oneshot::channel();
        let (release_tx, release_rx) = tokio::sync::oneshot::channel();
        let task_owner = Arc::clone(&owner);
        let task_completed = Arc::clone(&completed);
        owner.retain_issued_world_observer(tokio::spawn(async move {
            receipt_tx.send(()).unwrap();
            release_rx.await.unwrap();
            task_owner.retain_issued_world_observer(tokio::spawn(async move {
                tokio::task::yield_now().await;
                task_completed.store(true, Ordering::SeqCst);
            }));
        }));
        receipt_rx.await.unwrap();
        assert!(
            !completed.load(Ordering::SeqCst),
            "receipt return does not terminate observation"
        );
        let joined_owner = Arc::clone(&owner);
        let join = tokio::spawn(async move { joined_owner.join_issued_world_observers().await });
        tokio::task::yield_now().await;
        assert!(
            !join.is_finished(),
            "local completion must retain the actual pending observer"
        );
        release_tx.send(()).unwrap();
        join.await.unwrap().unwrap();
        assert!(
            completed.load(Ordering::SeqCst),
            "late descendant must finish before the helper lifetime ends"
        );
        owner.join_issued_world_observers().await.unwrap();
    }
}
