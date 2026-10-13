//! Error types for the queue abstraction.

use thiserror::Error;

use crate::types::{TaskId, TaskStatus};

/// Convenient result alias for queue operations.
pub type QueueResult<T> = Result<T, QueueError>;

/// Errors surfaced by `TaskQueue` implementations and task encoding.
///
/// Backend-specific errors are carried in [`QueueError::Backend`] so the core
/// error type stays free of any storage-engine dependency.
#[derive(Debug, Error)]
pub enum QueueError {
    #[error("task {0} was not found")]
    TaskNotFound(TaskId),

    /// The claim is no longer valid: the lease expired and the task was
    /// reclaimed or dead-lettered, or another worker already resolved it.
    #[error("claim on task {0} is no longer valid (reclaimed, dead-lettered, or already resolved)")]
    ClaimLost(TaskId),

    #[error("task {0} is in state `{1}`, which does not permit this operation")]
    InvalidState(TaskId, TaskStatus),

    #[error("a task with the same deduplication key is already pending or running (key: {0:?})")]
    DuplicateTask(Option<String>),

    #[error(
        "payload for task type `{task_type}` uses unsupported version {found} (expected {expected})"
    )]
    UnsupportedVersion {
        task_type: String,
        found: u16,
        expected: u16,
    },

    #[error("failed to decode payload for task type `{task_type}`")]
    PayloadDecode {
        task_type: String,
        source: serde_json::Error,
    },

    #[error("failed to encode payload")]
    PayloadEncode(#[source] serde_json::Error),

    /// Opaque backend failure (SQL error, Redis error, ...).
    #[error(transparent)]
    Backend(#[from] Box<dyn std::error::Error + Send + Sync>),
}
