-- Junction tables for assignment as well. 
CREATE TABLE assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id UUID NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    instructions TEXT,
    assigned_at TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    total_points NUMERIC(10,2) NOT NULL DEFAULT 0
        CHECK (total_points >= 0),
    status assignment_status NOT NULL DEFAULT 'DRAFT',
    created_by UUID NOT NULL REFERENCES users(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    CHECK (due_at IS NULL OR assigned_at IS NULL OR due_at >= assigned_at)
);

CREATE INDEX assignments_class_idx ON assignments(class_id);
CREATE INDEX assignments_due_idx ON assignments(due_at);

CREATE TABLE assignment_students (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    student_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    status assignment_student_status NOT NULL DEFAULT 'ASSIGNED',
    assigned_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    started_at TIMESTAMPTZ,
    submitted_at TIMESTAMPTZ,
    graded_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (assignment_id, student_id)
);

CREATE INDEX assignment_students_student_idx
    ON assignment_students(student_id);

CREATE INDEX assignment_students_status_idx
    ON assignment_students(assignment_id, status);

-- The same repository can be associated with multiple assignments.
-- student_id is retained here because an assignment can explicitly map a
-- repository to a student before any submission exists.
CREATE TABLE assignment_repositories (
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    repository_id UUID NOT NULL REFERENCES repositories(id) ON DELETE CASCADE,
    student_id UUID REFERENCES users(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (assignment_id, repository_id)
);

CREATE INDEX assignment_repositories_student_idx
    ON assignment_repositories(student_id);

CREATE INDEX assignment_repositories_assignment_student_idx
    ON assignment_repositories(assignment_id, student_id);
