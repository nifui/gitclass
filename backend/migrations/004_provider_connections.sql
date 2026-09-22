-- OAuth/API credentials are deliberately separate from identity data.
-- *_encrypted values must be encrypted by the application before storage.
CREATE TABLE provider_connections (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    external_identity_id UUID NOT NULL
        REFERENCES external_identities(id) ON DELETE CASCADE,
    access_token_encrypted TEXT,
    refresh_token_encrypted TEXT,
    access_token_expires_at TIMESTAMPTZ,
    scopes TEXT[] NOT NULL DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (external_identity_id)
);
