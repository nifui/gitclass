CREATE TABLE IF NOT EXISTS assignment_repository (
    assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repository(id) ON DELETE CASCADE,
    student_id UUID REFERENCES app_user(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (assignment_id, repository_id)
);

CREATE INDEX IF NOT EXISTS idx_assignment_repository_student
    ON assignment_repository(student_id);

CREATE INDEX IF NOT EXISTS idx_assignment_repository_assignment_student
    ON assignment_repository(assignment_id, student_id);
