//! PostgreSQL implementation of [`TaskQueue`] on top of `sqlx::PgPool`.
//!
//! Correctness notes:
//!
//! * Claiming is a **single atomic statement**: a `FOR UPDATE SKIP LOCKED`
//!   sub-select candidate rows, then transitions them to `running`. Concurrent
//!   claimers never block each other and never see the same row, even under
//!   READ COMMITTED.
//! * Expired-lease reclamation is folded into the same statement: an expired
//!   `running` task whose retry budget remains is re-claimed (attempts + 1);
//!   one whose budget is exhausted is dead-lettered in the same update and
//!   filtered from the returned claims. No separate reaper process is needed.
//! * `ack`/`fail`/`release` are guarded by `status = 'running' AND
//!   claimed_by = $worker`, so a stale worker cannot resolve a task that was
//!   reclaimed by someone else.
//! * `fail` runs in an explicit transaction with `SELECT ... FOR UPDATE` on
//!   the single row while the retry policy is evaluated in Rust.
//! * All eligibility/lease arithmetic uses SQL `now()` (database clock).

use std::time::Duration;

use serde_json::Value as JsonValue;
use sqlx::postgres::PgPoolOptions;
use sqlx::{Executor, PgPool, Postgres};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::error::{QueueError, QueueResult};
use crate::queue::TaskQueue;
use crate::types::{
    ClaimOptions, ClaimedTask, FailOutcome, NewTask, RetryPolicy, TaskFailure, TaskId, TaskRecord,
    TaskStatus, WorkerId,
};

/// Map sqlx errors into the backend-agnostic [`QueueError`].
impl From<sqlx::Error> for QueueError {
    fn from(value: sqlx::Error) -> Self {
        Self::Backend(Box::new(value))
    }
}

impl From<sqlx::migrate::MigrateError> for QueueError {
    fn from(value: sqlx::migrate::MigrateError) -> Self {
        Self::Backend(Box::new(value))
    }
}

/// The single candidate statement does four things atomically:
/// 1. selects due pending tasks **and** expired-lease running tasks,
/// 2. skips rows locked by concurrent claimers (`SKIP LOCKED`),
/// 3. dead-letters expired tasks with no retry budget left,
/// 4. claims the rest with a fresh lease and an incremented attempt counter.
const CLAIM_SQL: &str = r#"
    UPDATE task_queue_tasks AS t
    SET status           = CASE WHEN t.attempts >= t.max_attempts THEN 'failed' ELSE 'running' END,
        attempts         = CASE WHEN t.attempts >= t.max_attempts THEN t.attempts ELSE t.attempts + 1 END,
        claimed_at       = CASE WHEN t.attempts >= t.max_attempts THEN t.claimed_at ELSE now() END,
        claimed_by       = CASE WHEN t.attempts >= t.max_attempts THEN t.claimed_by ELSE $1 END,
        lease_expires_at = CASE WHEN t.attempts >= t.max_attempts THEN NULL
                                ELSE now() + make_interval(secs => $2) END,
        updated_at       = now(),
        last_error       = CASE WHEN t.attempts >= t.max_attempts
                                THEN COALESCE(t.last_error, 'lease expired before acknowledgement')
                                ELSE NULL END
    WHERE t.id IN (
        SELECT id
        FROM task_queue_tasks
        WHERE (status = 'pending'   AND available_at    <= now())
           OR (status = 'running'   AND lease_expires_at <= now())
        ORDER BY priority DESC, available_at ASC, id ASC
        LIMIT $3
        FOR UPDATE SKIP LOCKED
    )
    RETURNING *
"#;

const ENQUEUE_SQL: &str = r#"
    INSERT INTO task_queue_tasks
        (id, task_type, payload_version, payload, status, priority, attempts, max_attempts,
         retry_policy, available_at, dedup_key)
    VALUES ($1, $2, $3, $4, 'pending', $5, 0, $6, $7, $8, $9)
    RETURNING *
"#;

const ACK_SQL: &str = r#"
    UPDATE task_queue_tasks
    SET status = 'completed', completed_at = now(), lease_expires_at = NULL,
        claimed_by = NULL, last_error = NULL, updated_at = now()
    WHERE id = $1 AND status = 'running' AND claimed_by = $2
    RETURNING id
"#;

const RELEASE_SQL: &str = r#"
    UPDATE task_queue_tasks
    SET status = 'pending',
        available_at = now() + make_interval(secs => $3),
        lease_expires_at = NULL, claimed_by = NULL, updated_at = now()
    WHERE id = $1 AND status = 'running' AND claimed_by = $2
    RETURNING id
"#;

const EXTEND_LEASE_SQL: &str = r#"
    UPDATE task_queue_tasks
    SET lease_expires_at = now() + make_interval(secs => $3), updated_at = now()
    WHERE id = $1 AND status = 'running' AND claimed_by = $2
    RETURNING lease_expires_at
"#;

const CANCEL_SQL: &str = r#"
    UPDATE task_queue_tasks
    SET status = 'cancelled', lease_expires_at = NULL, claimed_by = NULL, updated_at = now()
    WHERE id = $1 AND status = 'pending'
"#;

const PURGE_SQL: &str = r#"
    DELETE FROM task_queue_tasks
    WHERE status = 'completed' AND completed_at < now() - make_interval(secs => $1)
"#;

/// Task queue backed by PostgreSQL. Cheap to clone (shares the pool).
#[derive(Clone)]
pub struct PostgresTaskQueue {
    pool: PgPool,
}

impl PostgresTaskQueue {
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn connect(url: &str, max_connections: u32) -> QueueResult<Self> {
        let pool = PgPoolOptions::new()
            .max_connections(max_connections)
            .connect(url)
            .await?;
        Ok(Self { pool })
    }

    pub const fn pool(&self) -> &PgPool {
        &self.pool
    }
}

// ------------------------------------------------------------------ row shape

#[derive(Debug, sqlx::FromRow)]
struct TaskRow {
    id: Uuid,
    task_type: String,
    payload_version: i32,
    payload: Vec<u8>,
    status: String,
    priority: i16,
    attempts: i32,
    max_attempts: i32,
    retry_policy: JsonValue,
    available_at: OffsetDateTime,
    created_at: OffsetDateTime,
    updated_at: OffsetDateTime,
    completed_at: Option<OffsetDateTime>,
    claimed_at: Option<OffsetDateTime>,
    claimed_by: Option<String>,
    lease_expires_at: Option<OffsetDateTime>,
    last_error: Option<String>,
    dedup_key: Option<String>,
}

fn db_u32(value: i32, what: &'static str) -> QueueResult<u32> {
    u32::try_from(value)
        .map_err(|_| QueueError::Backend(format!("invalid {what} in database row").into()))
}

impl TryFrom<TaskRow> for TaskRecord {
    type Error = QueueError;

    fn try_from(row: TaskRow) -> QueueResult<Self> {
        Ok(Self {
            id: TaskId::from_uuid(row.id),
            task_type: row.task_type,
            payload_version: u16::try_from(row.payload_version)
                .map_err(|_| QueueError::Backend("invalid payload_version".into()))?,
            payload: row.payload,
            status: TaskStatus::from_db(&row.status)?,
            priority: row.priority,
            attempts: db_u32(row.attempts, "attempts")?,
            max_attempts: db_u32(row.max_attempts, "max_attempts")?,
            retry_policy: serde_json::from_value(row.retry_policy)
                .map_err(|e| QueueError::Backend(Box::new(e)))?,
            available_at: row.available_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
            completed_at: row.completed_at,
            claimed_at: row.claimed_at,
            claimed_by: row.claimed_by.map(WorkerId::new),
            lease_expires_at: row.lease_expires_at,
            last_error: row.last_error,
            dedup_key: row.dedup_key,
        })
    }
}

// ----------------------------------------------------------- error refinement

async fn status_of<'e, E>(executor: E, id: TaskId) -> QueueResult<Option<TaskStatus>>
where
    E: Executor<'e, Database = Postgres>,
{
    let row: Option<(String,)> =
        sqlx::query_as("SELECT status FROM task_queue_tasks WHERE id = $1")
            .bind(id.as_uuid())
            .fetch_optional(executor)
            .await?;
    row.map(|(s,)| TaskStatus::from_db(&s)).transpose()
}

/// Distinguish "never existed" from "existed but the claim was lost".
async fn classify_missing<'e, E>(executor: E, id: TaskId) -> QueueResult<QueueError>
where
    E: Executor<'e, Database = Postgres>,
{
    let found = status_of(executor, id).await?;
    Ok(match found {
        None => QueueError::TaskNotFound(id),
        Some(_) => QueueError::ClaimLost(id),
    })
}

// ----------------------------------------------------------------- trait impl

impl TaskQueue for PostgresTaskQueue {
    async fn enqueue(&self, task: NewTask) -> QueueResult<TaskRecord> {
        let retry_policy =
            serde_json::to_value(&task.retry_policy).map_err(QueueError::PayloadEncode)?;
        let max_attempts = i32::try_from(task.retry_policy.max_attempts).unwrap_or(i32::MAX);

        let result = sqlx::query_as::<_, TaskRow>(ENQUEUE_SQL)
            .bind(TaskId::generate().as_uuid())
            .bind(&task.task_type)
            .bind(i32::from(task.payload_version))
            .bind(&task.payload)
            .bind(task.priority)
            .bind(max_attempts)
            .bind(retry_policy)
            .bind(task.run_at)
            .bind(task.dedup_key.clone())
            .fetch_one(&self.pool)
            .await;

        match result {
            Ok(row) => TaskRecord::try_from(row),
            // The partial unique index on dedup_key rejected a duplicate.
            Err(sqlx::Error::Database(db))
                if db.constraint().is_some_and(|c| c.contains("dedup_key")) =>
            {
                Err(QueueError::DuplicateTask(task.dedup_key))
            }
            Err(e) => Err(e.into()),
        }
    }

    async fn claim(&self, options: ClaimOptions) -> QueueResult<Vec<ClaimedTask>> {
        let limit = i64::try_from(options.max_tasks.max(1)).unwrap_or(i64::MAX);
        let rows = sqlx::query_as::<_, TaskRow>(CLAIM_SQL)
            .bind(options.worker_id.as_str())
            .bind(options.lease_duration.as_secs_f64())
            .bind(limit)
            .fetch_all(&self.pool)
            .await?;

        rows.into_iter()
            // Rows dead-lettered by the same statement (expired lease, exhausted
            // budget) are records for inspection only — never execute them.
            .filter(|row| row.status == "running")
            .map(|row| {
                TaskRecord::try_from(row).map(|record| ClaimedTask {
                    record,
                    worker_id: options.worker_id.clone(),
                })
            })
            .collect()
    }

    async fn ack(&self, task: ClaimedTask) -> QueueResult<()> {
        let id = task.id();
        let updated = sqlx::query(ACK_SQL)
            .bind(id.as_uuid())
            .bind(task.worker_id.as_str())
            .execute(&self.pool)
            .await?;
        if updated.rows_affected() == 0 {
            return Err(classify_missing(&self.pool, id).await?);
        }
        Ok(())
    }

    async fn fail(&self, task: ClaimedTask, failure: TaskFailure) -> QueueResult<FailOutcome> {
        let id = task.id();

        let mut tx = self.pool.begin().await?;

        // Lock the single row while we evaluate the retry policy in Rust.
        // The predicate doubles as the lease-ownership check.
        let row: Option<(i32, i32, JsonValue)> = sqlx::query_as(
            "SELECT attempts, max_attempts, retry_policy FROM task_queue_tasks \
             WHERE id = $1 AND status = 'running' AND claimed_by = $2 FOR UPDATE",
        )
        .bind(id.as_uuid())
        .bind(task.worker_id.as_str())
        .fetch_optional(&mut *tx)
        .await?;

        let (attempts, max_attempts, policy_json) = match row {
            Some(row) => row,
            None => return Err(classify_missing(&mut *tx, id).await?),
        };

        let attempts = db_u32(attempts, "attempts")?;
        let max_attempts = db_u32(max_attempts, "max_attempts")?;
        let policy: RetryPolicy =
            serde_json::from_value(policy_json).map_err(|e| QueueError::Backend(Box::new(e)))?;

        let outcome = if !failure.retryable || attempts >= max_attempts {
            sqlx::query(
                "UPDATE task_queue_tasks SET status = 'failed', lease_expires_at = NULL, \
                 claimed_by = NULL, last_error = $3, updated_at = now() \
                 WHERE id = $1 AND status = 'running' AND claimed_by = $2",
            )
            .bind(id.as_uuid())
            .bind(task.worker_id.as_str())
            .bind(&failure.error)
            .execute(&mut *tx)
            .await?;
            FailOutcome::DeadLettered { attempts }
        } else {
            let delay = policy.delay_after_attempt(attempts);
            sqlx::query(
                "UPDATE task_queue_tasks SET status = 'pending', \
                 available_at = now() + make_interval(secs => $3), \
                 lease_expires_at = NULL, claimed_by = NULL, last_error = $4, updated_at = now() \
                 WHERE id = $1 AND status = 'running' AND claimed_by = $2",
            )
            .bind(id.as_uuid())
            .bind(task.worker_id.as_str())
            .bind(delay.as_secs_f64())
            .bind(&failure.error)
            .execute(&mut *tx)
            .await?;
            let next_attempt_at = OffsetDateTime::now_utc() + delay;
            FailOutcome::RetryScheduled {
                attempts: attempts + 1,
                next_attempt_at,
            }
        };

        tx.commit().await?;
        Ok(outcome)
    }

    async fn release(&self, task: ClaimedTask, delay: Option<Duration>) -> QueueResult<()> {
        let id = task.id();
        let updated = sqlx::query(RELEASE_SQL)
            .bind(id.as_uuid())
            .bind(task.worker_id.as_str())
            .bind(delay.unwrap_or_default().as_secs_f64())
            .execute(&self.pool)
            .await?;
        if updated.rows_affected() == 0 {
            return Err(classify_missing(&self.pool, id).await?);
        }
        Ok(())
    }

    async fn extend_lease(
        &self,
        task: &ClaimedTask,
        extension: Duration,
    ) -> QueueResult<OffsetDateTime> {
        let id = task.id();
        let row: Option<(Option<OffsetDateTime>,)> = sqlx::query_as(EXTEND_LEASE_SQL)
            .bind(id.as_uuid())
            .bind(task.worker_id.as_str())
            .bind(extension.as_secs_f64())
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some((Some(expiry),)) => Ok(expiry),
            Some((None,)) => Err(QueueError::Backend(
                "lease_expires_at unexpectedly NULL".into(),
            )),
            None => Err(classify_missing(&self.pool, id).await?),
        }
    }

    async fn get(&self, task_id: TaskId) -> QueueResult<Option<TaskRecord>> {
        let row: Option<TaskRow> = sqlx::query_as("SELECT * FROM task_queue_tasks WHERE id = $1")
            .bind(task_id.as_uuid())
            .fetch_optional(&self.pool)
            .await?;
        row.map(TaskRecord::try_from).transpose()
    }

    async fn cancel(&self, task_id: TaskId) -> QueueResult<bool> {
        let result = sqlx::query(CANCEL_SQL)
            .bind(task_id.as_uuid())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    async fn purge_completed(&self, older_than: Duration) -> QueueResult<u64> {
        let result = sqlx::query(PURGE_SQL)
            .bind(older_than.as_secs_f64())
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected())
    }
}
