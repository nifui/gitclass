CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    
    session_id UUID NOT NULL REFERENCES sessions(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,    
    family_id UUID NOT NULL,
    device_name TEXT
);

CREATE INDEX refresh_tokens_user_idx
    ON refresh_tokens(user_id);

CREATE INDEX refresh_tokens_session_idx
    ON refresh_tokens(session_id);

CREATE INDEX refresh_tokens_expiry_idx
    ON refresh_tokens(expires_at);

