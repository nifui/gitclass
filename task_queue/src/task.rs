//! The [`Task`] trait: how application-specific work units integrate with the
//! queue without backends knowing anything about them.

use serde::{Serialize, de::DeserializeOwned};
use time::OffsetDateTime;

use crate::error::{QueueError, QueueResult};
use crate::types::{NewTask, TaskMetadata};

/// A serializable unit of work.
///
/// Implement this on your payload structs. The queue never sees the concrete
/// type: it only receives an encoded [`NewTask`] and hands back opaque payload
/// bytes, so queue backends stay fully decoupled from task types.
///
/// # Object safety
/// This trait is intentionally **not** object safe (`TASK_TYPE`/`PAYLOAD_VERSION`
/// associated consts and `Self`-returning decode). That is by design: dynamic
/// dispatch belongs at the handler layer — see
/// `crate::distributor::TaskRegistry`, which routes payloads by `task_type`.
///
/// # Versioning
/// Bump [`Task::PAYLOAD_VERSION`] when the serialized shape changes
/// incompatibly. `from_payload` rejects mismatched versions; for zero-downtime
/// migrations, decode multiple versions in an override of `from_payload` and
/// normalize to the current shape.
#[allow(clippy::wrong_self_convention)]
pub trait Task: Serialize + DeserializeOwned + Send + Sync + 'static {
    /// Stable discriminator persisted next to the payload and used to route
    /// the payload to the right handler at decode time.
    const TASK_TYPE: &'static str;

    /// Payload schema version. Defaults to `1`.
    const PAYLOAD_VERSION: u16 = 1;

    /// Queue metadata (priority, retries, scheduling, dedup) for this task.
    fn metadata(&self) -> TaskMetadata {
        TaskMetadata::default()
    }

    /// Encode into the backend-agnostic representation, with explicit metadata
    /// (used by `TaskScheduler::schedule_with` to override `metadata()`).
    fn into_new_task_with(&self, metadata: TaskMetadata) -> QueueResult<NewTask> {
        Ok(NewTask {
            task_type: Self::TASK_TYPE.to_owned(),
            payload_version: Self::PAYLOAD_VERSION,
            payload: serde_json::to_vec(self).map_err(QueueError::PayloadEncode)?,
            priority: metadata.priority,
            retry_policy: metadata.retry_policy,
            run_at: metadata.run_at.unwrap_or_else(OffsetDateTime::now_utc),
            dedup_key: metadata.dedup_key,
        })
    }

    /// Encode using this task's own `metadata()`.
    fn into_new_task(&self) -> QueueResult<NewTask> {
        self.into_new_task_with(self.metadata())
    }

    /// Decode a payload produced by `into_new_task*`.
    fn from_payload(payload: &[u8], version: u16) -> QueueResult<Self>
    where
        Self: Sized,
    {
        if version != Self::PAYLOAD_VERSION {
            return Err(QueueError::UnsupportedVersion {
                task_type: Self::TASK_TYPE.to_owned(),
                found: version,
                expected: Self::PAYLOAD_VERSION,
            });
        }
        serde_json::from_slice(payload).map_err(|source| QueueError::PayloadDecode {
            task_type: Self::TASK_TYPE.to_owned(),
            source,
        })
    }
}
