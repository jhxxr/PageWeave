use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::{mpsc, Mutex};

/// One running (or recently finished) translate task, owned by the registry.
///
/// The runner owns the child process. `cancel_translate` sends a cancellation
/// signal through the registry so the runner can kill and reap the process.
pub struct RunningTask {
    pub cancel_tx: mpsc::UnboundedSender<()>,
    pub status: String,
}

#[derive(Default)]
pub struct TaskRegistry {
    inner: Mutex<HashMap<String, RunningTask>>,
}

impl TaskRegistry {
    pub fn new() -> Arc<Self> {
        Arc::new(Self::default())
    }

    pub async fn insert(&self, task_id: String, task: RunningTask) {
        let mut g = self.inner.lock().await;
        g.insert(task_id, task);
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

    /// Remove a task entry (after the runner finishes).
    pub async fn remove(&self, task_id: &str) {
        let mut g = self.inner.lock().await;
        g.remove(task_id);
    }
}
