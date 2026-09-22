
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
