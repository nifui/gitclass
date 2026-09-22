CREATE TABLE submissions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE RESTRICT,
    commit_sha TEXT NOT NULL,
    branch TEXT,
    submitted_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    status submission_status NOT NULL DEFAULT 'PENDING',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX submissions_assignment_student_idx
    ON submissions(assignment_id, student_id);

CREATE INDEX submissions_student_idx
    ON submissions(student_id);

CREATE INDEX submissions_status_idx
    ON submissions(status);

CREATE INDEX submissions_repository_idx
    ON submissions(repository_id);


