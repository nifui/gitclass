CREATE TABLE IF NOT EXISTS workflow (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    description TEXT,
    language TEXT,
    version TEXT,
    source_type workflow_source_type NOT NULL,
    source_path TEXT,
    timeout_seconds INTEGER NOT NULL DEFAULT 300
        CHECK (timeout_seconds > 0 AND timeout_seconds <= 86400),
    enabled BOOLEAN NOT NULL DEFAULT TRUE,
    created_by UUID NOT NULL REFERENCES app_user(id) ON DELETE RESTRICT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS assignment_workflow (
    assignment_id UUID NOT NULL REFERENCES assignment(id) ON DELETE CASCADE,
    workflow_id UUID NOT NULL REFERENCES workflow(id) ON DELETE CASCADE,
    order_index INTEGER NOT NULL DEFAULT 0 CHECK (order_index >= 0),
    required BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (assignment_id, workflow_id)
);

CREATE INDEX IF NOT EXISTS idx_assignment_workflow_order
    ON assignment_workflow(assignment_id, order_index);
