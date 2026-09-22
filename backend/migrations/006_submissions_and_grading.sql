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

CREATE TABLE rubrics (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    description TEXT,
    max_points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (max_points >= 0),
    order_index INTEGER NOT NULL DEFAULT 0 CHECK (order_index >= 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX rubrics_assignment_order_idx
    ON rubrics(assignment_id, order_index);

CREATE TABLE rubric_grades (
    rubric_id UUID NOT NULL REFERENCES rubrics(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    points NUMERIC(10,2) NOT NULL DEFAULT 0 CHECK (points >= 0),
    feedback TEXT,
    graded_by UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (rubric_id, student_id)
);
