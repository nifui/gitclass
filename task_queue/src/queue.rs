//! The backend-agnostic [`TaskQueue`] trait.
//!
//! # Delivery semantics (contract for all implementations)
//!
//! The abstraction provides **at-least-once** delivery. It does *not* provide
//! exactly-once execution: a worker's side effects cannot be committed
//! atomically with the acknowledgment, so a crash between "work done" and
//! "ack written" causes re-execution. Handlers must therefore be idempotent,
//! or deduplicate by a business key.
//!
//! # Concurrency & object safety
//!
//! All methods take `&self`; implementations are expected to be internally
//! synchronized (e.g., they wrap a connection pool). The trait has no generic
//! methods and uses `async_trait`, so it is object-safe: share it as
//! `Arc<dyn TaskQueue>`.
//!
//! # Time
//!
//! Eligibility (`available_at`) and lease expiry are decided by the backend's
//! clock (in the PostgreSQL implementation: SQL `now()`), never by worker
//! clocks, so clock skew between workers cannot corrupt leases.

use std::time::Duration;

use time::OffsetDateTime;

use crate::error::QueueResult;
use crate::types::{
    ClaimOptions, ClaimedTask, FailOutcome, NewTask, TaskFailure, TaskId, TaskRecord,
};
#[allow(async_fn_in_trait)]
pub trait TaskQueue: Send + Sync + 'static {
    /// Persist a new task and return its full record (including the assigned
    /// [`TaskId`]).
    ///
    /// If `dedup_key` is set and another task with the same key is currently
    /// pending or running, this fails with `QueueError::DuplicateTask`.
    async fn enqueue(&self, task: NewTask) -> QueueResult<TaskRecord>;

    /// Atomically claim up to `options.max_tasks` eligible tasks for
    /// `options.worker_id`, granting each a fresh lease of
    /// `options.lease_duration`.
    ///
    /// Eligible means: `status = pending` and `available_at` is due, **or**
    /// `status = running` with an expired lease — i.e. a crashed worker's
    /// abandoned claim. Reclamation counts as a new attempt; an expired task
    /// whose retry budget is exhausted is dead-lettered instead (never handed
    /// to workers again) and is filtered out of the result.
    ///
    /// Ordering is best effort: priority descending, then oldest `available_at`.
    /// Returns an empty vec when nothing is eligible — poll with backoff.
    async fn claim(&self, options: ClaimOptions) -> QueueResult<Vec<ClaimedTask>>;

    /// Mark the task as successfully completed.
    ///
    /// Consumes the claim (single use). Fails with `QueueError::ClaimLost` if
    /// the task was reclaimed, dead-lettered, or already resolved, and with
    /// `QueueError::TaskNotFound` if it no longer exists.
    fn ack(&self, task: ClaimedTask) -> impl Future<Output = QueueResult<()>> + Send;

    /// Report a failed execution.
    ///
    /// Applies the task's persisted [`crate::types::RetryPolicy`]: retryable
    /// failures with remaining budget are requeued as `pending` with
    /// `available_at = now + backoff`; non-retryable failures or exhausted
    /// budgets dead-letter the task (`status = failed`). Consumes the claim.
    fn fail(
        &self,
        task: ClaimedTask,
        failure: TaskFailure,
    ) -> impl Future<Output = QueueResult<FailOutcome>> + Send;

    /// Return the task to `pending` without consuming an attempt, optionally
    /// delaying its availability. Use for "I could not run this now" (e.g. no
    /// handler registered, graceful shutdown). Consumes the claim.
    ///
    /// Caveat: because `release` never increments `attempts`, a task that is
    /// released in a loop never dead-letters. If the condition is permanent,
    /// prefer `fail(.., retryable = false)`.
    fn release(
        &self,
        task: ClaimedTask,
        delay: Option<Duration>,
    ) -> impl Future<Output = QueueResult<()>> + Send;

    /// Renew the lease ("heartbeat") for long-running handlers: sets
    /// `lease_expires_at = now + extension`. Returns the new expiry, or
    /// `QueueError::ClaimLost` if the task was reclaimed meanwhile.
    fn extend_lease(
        &self,
        task: &ClaimedTask,
        extension: Duration,
    ) -> impl Future<Output = QueueResult<OffsetDateTime>> + Send;

    /// Point lookup of a task's full record, if it exists.
    async fn get(&self, task_id: TaskId) -> QueueResult<Option<TaskRecord>>;

    /// Cancel a *pending* task (running tasks cannot be cancelled safely
    /// through the queue; use the lease/ack protocol instead). Returns whether
    /// a task was cancelled.
    async fn cancel(&self, task_id: TaskId) -> QueueResult<bool>;

    /// Permanently delete completed tasks older than `older_than`; returns the
    /// number of rows removed. Failed tasks are retained for inspection.
    async fn purge_completed(&self, older_than: Duration) -> QueueResult<u64>;
}
