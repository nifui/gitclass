CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),

    user_id UUID NOT NULL
        REFERENCES users(id)
        ON DELETE CASCADE,

    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_used_at TIMESTAMPTZ,

    revoked_at TIMESTAMPTZ,

    device_name TEXT,
    ip_address INET,
    user_agent TEXT
);

CREATE INDEX sessions_user_idx
ON sessions(user_id);
