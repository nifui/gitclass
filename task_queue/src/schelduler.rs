//! [`TaskScheduler`] — decides *when* tasks should become eligible and owns
//! the enqueue path. It knows nothing about handlers or execution.

use std::sync::Arc;
use std::time::Duration;

use time::OffsetDateTime;

use crate::error::QueueResult;
use crate::queue::TaskQueue;
use crate::task::Task;
use crate::types::{TaskMetadata, TaskRecord};

/// Thin façade over a shared [`TaskQueue`]. Generic methods live here (not on
/// the trait) so the trait stays object-safe.
#[derive(Clone)]
pub struct TaskScheduler<Q>
where
    Q: TaskQueue,
{
    queue: Arc<Q>,
}

impl<Q> TaskScheduler<Q>
where
    Q: TaskQueue,
{
    pub const fn new(queue: Arc<Q>) -> Self {
        Self { queue }
    }

    pub const fn queue(&self) -> &Arc<Q> {
        &self.queue
    }

    /// Enqueue a task using its own [`Task::metadata`].
    pub async fn schedule<T: Task>(&self, task: &T) -> QueueResult<TaskRecord> {
        self.queue.enqueue(task.into_new_task()?).await
    }

    /// Enqueue with fully explicit metadata (overrides `Task::metadata`).
    pub async fn schedule_with<T: Task>(
        &self,
        task: &T,
        metadata: TaskMetadata,
    ) -> QueueResult<TaskRecord> {
        self.queue.enqueue(task.into_new_task_with(metadata)?).await
    }

    /// Delayed execution: eligible `delay` from now.
    pub async fn schedule_after<T: Task>(
        &self,
        task: &T,
        delay: Duration,
    ) -> QueueResult<TaskRecord> {
        let mut new_task = task.into_new_task()?;
        new_task.run_at = OffsetDateTime::now_utc() + delay;
        self.queue.enqueue(new_task).await
    }

    /// Scheduled execution: eligible at (or after) an absolute instant.
    pub async fn schedule_at<T: Task>(
        &self,
        task: &T,
        run_at: OffsetDateTime,
    ) -> QueueResult<TaskRecord> {
        let mut new_task = task.into_new_task()?;
        new_task.run_at = run_at;
        self.queue.enqueue(new_task).await
    }
}
