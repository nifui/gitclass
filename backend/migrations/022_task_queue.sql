CREATE TABLE IF NOT EXISTS task_queue_tasks (
    id               UUID PRIMARY KEY,
    task_type        TEXT        NOT NULL,
    payload_version  INT         NOT NULL DEFAULT 1,
    payload          BYTEA       NOT NULL,
    status           TEXT        NOT NULL DEFAULT 'pending'
                     CONSTRAINT task_queue_tasks_status_check
                     CHECK (status IN ('pending', 'running', 'completed', 'failed', 'cancelled')),
    priority         SMALLINT    NOT NULL DEFAULT 0,
    attempts         INT         NOT NULL DEFAULT 0,
    max_attempts     INT         NOT NULL DEFAULT 3,
    retry_policy     JSONB       NOT NULL,
    available_at     TIMESTAMPTZ NOT NULL DEFAULT now(),
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    completed_at     TIMESTAMPTZ,
    claimed_at       TIMESTAMPTZ,
    claimed_by       TEXT,
    lease_expires_at TIMESTAMPTZ,
    last_error       TEXT,
    dedup_key        TEXT
);

-- Claim polling: pending tasks, highest priority first, then oldest available.
CREATE INDEX IF NOT EXISTS task_queue_tasks_pending_claim_idx
    ON task_queue_tasks (priority DESC, available_at ASC)
    WHERE status = 'pending';

-- Lease reclamation: running tasks whose lease has expired (crashed workers).
CREATE INDEX IF NOT EXISTS task_queue_tasks_running_lease_idx
    ON task_queue_tasks (lease_expires_at ASC)
    WHERE status = 'running';

-- Observability / per-type queries.
CREATE INDEX IF NOT EXISTS task_queue_tasks_type_idx
    ON task_queue_tasks (task_type, created_at DESC);

-- Purging completed tasks.
CREATE INDEX IF NOT EXISTS task_queue_tasks_completed_at_idx
    ON task_queue_tasks (completed_at ASC)
    WHERE status = 'completed';

-- Enqueue-time idempotency: at most one active (pending/running) task per key.
-- The key is freed automatically when the task reaches a terminal state.
CREATE UNIQUE INDEX IF NOT EXISTS task_queue_tasks_dedup_active_uidx
    ON task_queue_tasks (dedup_key)
    WHERE dedup_key IS NOT NULL AND status IN ('pending', 'running');
