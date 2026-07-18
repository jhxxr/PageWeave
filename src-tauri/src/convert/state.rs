use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

/// One running convert task. The runner owns the child process; cancel sends a
/// signal so the runner can kill and reap it.
pub struct RunningConvert {
    pub cancel_tx: mpsc::UnboundedSender<()>,
    pub status: String,
}

/// Convert-local registry. At most one running task (busy check on start).
/// Independent from translate's TaskRegistry.
#[derive(Default)]
pub struct ConvertRegistry {
    inner: Mutex<HashMap<String, RunningConvert>>,
}

impl ConvertRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    /// Whether any convert task is currently running. Kept for diagnostics /
    /// future UI probes; start uses `try_begin` for the atomic busy gate.
    #[allow(dead_code)]
    pub async fn is_busy(&self) -> bool {
        let g = self.inner.lock().await;
        g.values().any(|t| t.status == "running")
    }

    /// Atomically claim the single convert slot. Returns false if already busy.
    ///
    /// Call this from `start_convert` *before* spawning the runner so two concurrent
    /// starts cannot both pass a separate busy check.
    pub async fn try_begin(
        &self,
        task_id: String,
        cancel_tx: mpsc::UnboundedSender<()>,
    ) -> bool {
        let mut g = self.inner.lock().await;
        if g.values().any(|t| t.status == "running") {
            return false;
        }
        g.insert(
            task_id,
            RunningConvert {
                cancel_tx,
                status: "running".into(),
            },
        );
        true
    }

    pub async fn set_status(&self, task_id: &str, status: &str) {
        let mut g = self.inner.lock().await;
        if let Some(t) = g.get_mut(task_id) {
            t.status = status.to_string();
        }
    }

    /// Signal the runner to kill the child. Returns true if the task exists.
    pub async fn kill(&self, task_id: &str) -> bool {
        let mut g = self.inner.lock().await;
        let Some(task) = g.get_mut(task_id) else {
            return false;
        };
        task.status = "cancelled".into();
        let _ = task.cancel_tx.send(());
        true
    }

    pub async fn status(&self, task_id: &str) -> Option<String> {
        let g = self.inner.lock().await;
        g.get(task_id).map(|t| t.status.clone())
    }

    pub async fn remove(&self, task_id: &str) {
        let mut g = self.inner.lock().await;
        g.remove(task_id);
    }
}
