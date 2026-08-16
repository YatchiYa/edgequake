//! Task queue implementations for background processing.
//!
//! ## Implements
//!
//! - **FEAT0920**: Task queue trait abstraction
//! - **FEAT0921**: Channel-based queue for in-process tasks
//! - **FEAT0922**: Bounded queue with backpressure
//!
//! ## Use Cases
//!
//! - **UC2610**: System enqueues document for async processing
//! - **UC2611**: Worker receives task from queue
//! - **UC2612**: System applies backpressure when queue full
//!
//! ## Enforces
//!
//! - **BR0920**: Queue capacity bounded to prevent memory exhaustion
//! - **BR0921**: Queue must support concurrent send/receive

use crate::{error::TaskResult, types::Task};
use async_trait::async_trait;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::debug;

/// Trait for task queue implementations
#[async_trait]
pub trait TaskQueue: Send + Sync {
    /// Send a task to the queue (may await when capacity is full).
    ///
    /// Prefer [`Self::try_send`] on the HTTP admit path (SPEC-132 / LAW-132-2):
    /// durable storage is the SSOT; wake must not hang handlers.
    async fn send(&self, task: Task) -> TaskResult<()>;

    /// Non-blocking wake send. Returns [`crate::error::TaskError::QueueFull`]
    /// when the channel is at capacity (SPEC-132 EC-3 / F-091-19).
    async fn try_send(&self, task: Task) -> TaskResult<()>;

    /// Receive a task from the queue (blocking)
    async fn receive(&self) -> TaskResult<Task>;

    /// Try to receive a task (non-blocking)
    async fn try_receive(&self) -> TaskResult<Option<Task>>;

    /// Get queue size (if supported)
    async fn size(&self) -> TaskResult<usize>;

    /// Check if queue is closed
    fn is_closed(&self) -> bool;
}

/// Type alias for shared queue
pub type SharedTaskQueue = Arc<dyn TaskQueue>;

/// Channel-based task queue using tokio::sync::mpsc
pub struct ChannelTaskQueue {
    sender: mpsc::Sender<Task>,
    receiver: Arc<tokio::sync::Mutex<mpsc::Receiver<Task>>>,
    capacity: usize,
    /// Approximate depth (send +1, receive -1) for observability.
    depth: Arc<AtomicUsize>,
}

impl ChannelTaskQueue {
    /// Create a new channel-based task queue
    pub fn new(capacity: usize) -> Self {
        let (sender, receiver) = mpsc::channel(capacity);

        Self {
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
            capacity,
            depth: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// Get the queue capacity
    pub fn capacity(&self) -> usize {
        self.capacity
    }

    /// Approximate in-channel task count (observability).
    pub fn approximate_depth(&self) -> usize {
        self.depth.load(Ordering::Relaxed)
    }
}

#[async_trait]
impl TaskQueue for ChannelTaskQueue {
    async fn send(&self, task: Task) -> TaskResult<()> {
        debug!("Sending task to queue: {}", task.track_id);

        self.sender
            .send(task)
            .await
            .map_err(|_| crate::error::TaskError::QueueClosed)?;
        self.depth.fetch_add(1, Ordering::Relaxed);

        Ok(())
    }

    async fn try_send(&self, task: Task) -> TaskResult<()> {
        debug!("try_send task to queue: {}", task.track_id);

        match self.sender.try_send(task) {
            Ok(()) => {
                self.depth.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(mpsc::error::TrySendError::Full(_)) => Err(crate::error::TaskError::QueueFull),
            Err(mpsc::error::TrySendError::Closed(_)) => Err(crate::error::TaskError::QueueClosed),
        }
    }

    async fn receive(&self) -> TaskResult<Task> {
        let mut receiver = self.receiver.lock().await;

        let task = receiver
            .recv()
            .await
            .ok_or(crate::error::TaskError::QueueClosed)?;
        let _ = self
            .depth
            .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                Some(v.saturating_sub(1))
            });
        Ok(task)
    }

    async fn try_receive(&self) -> TaskResult<Option<Task>> {
        let mut receiver = self.receiver.lock().await;

        match receiver.try_recv() {
            Ok(task) => {
                let _ = self
                    .depth
                    .fetch_update(Ordering::Relaxed, Ordering::Relaxed, |v| {
                        Some(v.saturating_sub(1))
                    });
                Ok(Some(task))
            }
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(crate::error::TaskError::QueueClosed)
            }
        }
    }

    async fn size(&self) -> TaskResult<usize> {
        Ok(self.approximate_depth())
    }

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

/// Unbounded channel-based task queue (use with caution in production)
pub struct UnboundedChannelTaskQueue {
    sender: mpsc::UnboundedSender<Task>,
    receiver: Arc<tokio::sync::Mutex<mpsc::UnboundedReceiver<Task>>>,
}

impl UnboundedChannelTaskQueue {
    /// Create a new unbounded channel-based task queue
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();

        Self {
            sender,
            receiver: Arc::new(tokio::sync::Mutex::new(receiver)),
        }
    }
}

impl Default for UnboundedChannelTaskQueue {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl TaskQueue for UnboundedChannelTaskQueue {
    async fn send(&self, task: Task) -> TaskResult<()> {
        debug!("Sending task to unbounded queue: {}", task.track_id);

        self.sender
            .send(task)
            .map_err(|_| crate::error::TaskError::QueueClosed)?;

        Ok(())
    }

    async fn try_send(&self, task: Task) -> TaskResult<()> {
        self.send(task).await
    }

    async fn receive(&self) -> TaskResult<Task> {
        let mut receiver = self.receiver.lock().await;

        receiver
            .recv()
            .await
            .ok_or(crate::error::TaskError::QueueClosed)
    }

    async fn try_receive(&self) -> TaskResult<Option<Task>> {
        let mut receiver = self.receiver.lock().await;

        match receiver.try_recv() {
            Ok(task) => Ok(Some(task)),
            Err(mpsc::error::TryRecvError::Empty) => Ok(None),
            Err(mpsc::error::TryRecvError::Disconnected) => {
                Err(crate::error::TaskError::QueueClosed)
            }
        }
    }

    async fn size(&self) -> TaskResult<usize> {
        Ok(0)
    }

    fn is_closed(&self) -> bool {
        self.sender.is_closed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::TaskType;

    const TEST_TENANT_ID: &str = "00000000-0000-0000-0000-000000000001";
    const TEST_WORKSPACE_ID: &str = "00000000-0000-0000-0000-000000000002";

    fn test_tenant_id() -> uuid::Uuid {
        uuid::Uuid::parse_str(TEST_TENANT_ID).unwrap()
    }

    fn test_workspace_id() -> uuid::Uuid {
        uuid::Uuid::parse_str(TEST_WORKSPACE_ID).unwrap()
    }

    #[tokio::test]
    async fn test_channel_queue_send_receive() {
        let queue = ChannelTaskQueue::new(10);
        let task = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Upload,
            serde_json::json!({"file": "test.pdf"}),
        );
        let track_id = task.track_id.clone();

        queue.send(task).await.unwrap();

        let received = queue.receive().await.unwrap();
        assert_eq!(received.track_id, track_id);
    }

    #[tokio::test]
    async fn test_channel_queue_capacity() {
        let queue = ChannelTaskQueue::new(2);

        let task1 = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({}),
        );
        let task2 = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({}),
        );

        queue.send(task1).await.unwrap();
        queue.send(task2).await.unwrap();

        // Queue is now full (capacity=2)
        assert_eq!(queue.capacity(), 2);
    }

    /// SPEC-132 EC-3 / F-091-19: try_send must return QueueFull, never hang.
    #[tokio::test]
    async fn test_try_send_returns_queue_full_when_at_capacity() {
        let queue = ChannelTaskQueue::new(1);
        let task1 = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({"n": 1}),
        );
        let task2 = Task::new(
            test_tenant_id(),
            test_workspace_id(),
            TaskType::Insert,
            serde_json::json!({"n": 2}),
        );

        queue.try_send(task1).await.unwrap();
        let err = queue.try_send(task2).await.unwrap_err();
        assert!(
            matches!(err, crate::error::TaskError::QueueFull),
            "expected QueueFull, got {err:?}"
        );
    }

    #[tokio::test]
    async fn test_try_receive_empty() {
        let queue = ChannelTaskQueue::new(10);

        let result = queue.try_receive().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_unbounded_queue() {
        let queue = UnboundedChannelTaskQueue::new();

        // Send many tasks
        for i in 0..100 {
            let task = Task::new(
                test_tenant_id(),
                test_workspace_id(),
                TaskType::Insert,
                serde_json::json!({"index": i}),
            );
            queue.send(task).await.unwrap();
        }

        // Receive all tasks
        for _ in 0..100 {
            let _task = queue.receive().await.unwrap();
        }

        // Queue should be empty
        let result = queue.try_receive().await.unwrap();
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_queue_not_closed() {
        let queue = ChannelTaskQueue::new(10);
        assert!(!queue.is_closed());
    }
}
