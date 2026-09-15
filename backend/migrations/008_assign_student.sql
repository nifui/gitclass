CREATE TABLE IF NOT EXISTS assignment_student (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    status assignment_student_status NOT NULL DEFAULT 'ASSIGNED',
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    graded_at TIMESTAMPTZ,
    received_points NUMERIC(10,2)
        CHECK (received_points IS NULL OR received_points >= 0),
    late BOOLEAN NOT NULL DEFAULT FALSE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (assignment_id, student_id)
);

CREATE INDEX IF NOT EXISTS idx_assignment_student_student
    ON assignment_student(student_id);

CREATE INDEX IF NOT EXISTS idx_assignment_student_status
    ON assignment_student(assignment_id, status);

