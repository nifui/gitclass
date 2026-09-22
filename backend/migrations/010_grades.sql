-- A submission can be graded multiple times. Keeping grades as rows gives
-- the application a history instead of overwriting the previous grade.
CREATE TABLE grades (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    submission_id UUID REFERENCES submissions(id) ON DELETE SET NULL,
    points NUMERIC(10,2) NOT NULL CHECK (points >= 0),
    max_points NUMERIC(10,2) NOT NULL CHECK (max_points >= 0),
    graded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    grading_type grading_type NOT NULL,
    feedback TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CHECK (points <= max_points)
);

CREATE INDEX grades_assignment_student_idx
    ON grades(assignment_id, student_id);

CREATE INDEX grades_submission_idx
    ON grades(submission_id);


