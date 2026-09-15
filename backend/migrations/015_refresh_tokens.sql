CREATE TABLE IF NOT EXISTS refresh_token (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    device_name TEXT
);

CREATE INDEX IF NOT EXISTS idx_refresh_token_user
    ON refresh_token(user_id);

CREATE INDEX IF NOT EXISTS idx_refresh_token_expiry
    ON refresh_token(expires_at);
