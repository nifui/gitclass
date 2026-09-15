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


