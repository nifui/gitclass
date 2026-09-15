CREATE TABLE IF NOT EXISTS submission (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repository(id) ON DELETE RESTRICT,
    commit_sha TEXT NOT NULL,
    branch TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    status submission_status NOT NULL DEFAULT 'PENDING',
    is_late BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_submission_assignment_student
    ON submission(assignment_id, student_id);

CREATE INDEX IF NOT EXISTS idx_submission_student
    ON submission(student_id);

CREATE INDEX IF NOT EXISTS idx_submission_status
    ON submission(status);

CREATE INDEX IF NOT EXISTS idx_submission_repo
    ON submission(repository_id);

CREATE TABLE IF NOT EXISTS workflow_run (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    submission_id UUID NOT NULL REFERENCES submission(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflow(id) ON DELETE RESTRICT,
    status workflow_run_status NOT NULL DEFAULT 'QUEUED',
    started_at TIMESTAMPTZ,
    finished_at TIMESTAMPTZ,
    duration_ms BIGINT CHECK (duration_ms IS NULL OR duration_ms >= 0),
    exit_code INTEGER,
    worker_id TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT workflow_run_finished_after_started
        CHECK (
            finished_at IS NULL
            OR started_at IS NULL
            OR finished_at >= started_at
        )
);

CREATE INDEX IF NOT EXISTS idx_workflow_run_submission
    ON workflow_run(submission_id);

CREATE INDEX IF NOT EXISTS idx_workflow_run_status
    ON workflow_run(status);

CREATE TABLE IF NOT EXISTS workflow_test_result (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    workflow_run_id UUID NOT NULL REFERENCES workflow_run(id) ON DELETE CASCADE,
    test_name TEXT NOT NULL,
    status test_result_status NOT NULL,
    points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (points >= 0),
    max_points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (max_points >= 0),
    duration_ms BIGINT CHECK (duration_ms IS NULL OR duration_ms >= 0),
    message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT test_result_points_valid CHECK (points <= max_points)
);

CREATE INDEX IF NOT EXISTS idx_workflow_test_result_run
    ON workflow_test_result(workflow_run_id);
