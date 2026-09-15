CREATE TABLE IF NOT EXISTS assignment (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id UUID NOT NULL REFERENCES class(id) ON DELETE CASCADE,
    title TEXT NOT NULL,
    description TEXT,
    instructions TEXT,
    assigned_at TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    total_points NUMERIC(10,2) NOT NULL DEFAULT 0
        CHECK (total_points >= 0),
    status assignment_status NOT NULL DEFAULT 'DRAFT',
    created_by UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT assignment_due_after_assigned
        CHECK (due_at IS NULL OR assigned_at IS NULL OR due_at >= assigned_at)
);

CREATE INDEX IF NOT EXISTS idx_assignment_class
    ON assignment(class_id);

CREATE INDEX IF NOT EXISTS idx_assignment_due
    ON assignment(due_at);

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
