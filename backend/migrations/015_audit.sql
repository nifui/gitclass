CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID REFERENCES users(id) ON DELETE SET NULL,
    class_id UUID REFERENCES classes(id) ON DELETE SET NULL,
    action audit_action NOT NULL,
    entity_type audit_entity_type NOT NULL,
    entity_id UUID,
    metadata JSONB NOT NULL DEFAULT '{}'::jsonb,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX audit_logs_class_time_idx
    ON audit_logs(class_id, created_at DESC);

CREATE INDEX audit_logs_user_time_idx
    ON audit_logs(user_id, created_at DESC);

CREATE INDEX audit_logs_entity_idx
    ON audit_logs(entity_type, entity_id);
