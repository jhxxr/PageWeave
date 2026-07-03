use std::collections::HashMap;
use std::sync::Arc;

use tokio::process::Child;
use tokio::sync::Mutex;

/// One running (or recently finished) translate task, owned by the registry.
///
/// `child` is shared via `Arc<Mutex<Option<Child>>>`: the runner holds one clone
/// (and performs `wait` on the `Child` it takes out), the registry holds another.
/// `cancel_translate` calls `start_kill()` on the borrowed child.
pub struct RunningTask {
    pub child: Arc<Mutex<Option<Child>>>,
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

    /// Signal the child to kill. Returns true if there was a live child.
    pub async fn kill(&self, task_id: &str) -> bool {
        let g = self.inner.lock().await;
        let Some(task) = g.get(task_id) else {
            return false;
        };
        let mut guard = task.child.lock().await;
        if let Some(child) = guard.as_mut() {
            let _ = child.start_kill();
            return true;
        }
        false
    }

    /// Remove a task entry (after the runner finishes).
    pub async fn remove(&self, task_id: &str) {
        let mut g = self.inner.lock().await;
        g.remove(task_id);
    }
}
