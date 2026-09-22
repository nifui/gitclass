CREATE TABLE workflows (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    language TEXT,
    version TEXT,
    source_type workflow_source_type NOT NULL,
    source_path TEXT,
    timeout_seconds INTEGER NOT NULL DEFAULT 300
        CHECK (timeout_seconds > 0 AND timeout_seconds <= 86400),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE assignment_workflows (
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE CASCADE,
    order_index INTEGER NOT NULL DEFAULT 0 CHECK (order_index >= 0),
    required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (assignment_id, workflow_id)
);

CREATE INDEX assignment_workflows_order_idx
    ON assignment_workflows(assignment_id, order_index);

CREATE TABLE workflow_runs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflows(id) ON DELETE RESTRICT,
    status workflow_run_status NOT NULL DEFAULT 'QUEUED',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    duration_ms BIGINT CHECK (duration_ms IS NULL OR duration_ms >= 0),
    exit_code INTEGER,
    worker_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CHECK (finished_at IS NULL OR started_at IS NULL OR finished_at >= started_at)
);

CREATE INDEX workflow_runs_submission_idx
    ON workflow_runs(submission_id);

CREATE INDEX workflow_runs_status_idx
    ON workflow_runs(status);

CREATE TABLE workflow_test_results (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_run_id UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    test_name TEXT NOT NULL,
    status test_result_status NOT NULL,
    points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (points >= 0),
    max_points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (max_points >= 0),
    duration_ms BIGINT CHECK (duration_ms IS NULL OR duration_ms >= 0),
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CHECK (points <= max_points)
);

CREATE INDEX workflow_test_results_run_idx
    ON workflow_test_results(workflow_run_id);

CREATE TABLE workflow_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_run_id UUID NOT NULL REFERENCES workflow_runs(id) ON DELETE CASCADE,
    stream log_stream NOT NULL,
    storage_path TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX workflow_logs_run_idx
    ON workflow_logs(workflow_run_id);
