CREATE TABLE IF NOT EXISTS grade (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    submission_id UUID REFERENCES submission(id) ON DELETE SET NULL,
    points NUMERIC(10,2) NOT NULL CHECK (points >= 0),
    max_points NUMERIC(10,2) NOT NULL CHECK (max_points >= 0),
    graded_by UUID REFERENCES app_user(id) ON DELETE SET NULL,
    grading_type grading_type NOT NULL,
    feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT grade_points_valid CHECK (points <= max_points)
);

CREATE INDEX IF NOT EXISTS idx_grade_assignment_student
    ON grade(assignment_id, student_id);

CREATE INDEX IF NOT EXISTS idx_grade_submission
    ON grade(submission_id);


