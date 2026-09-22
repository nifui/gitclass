CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    password_hash TEXT,
    system_role system_role NOT NULL DEFAULT 'USER',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE UNIQUE INDEX users_username_ci_idx ON users (lower(username));
CREATE UNIQUE INDEX users_email_ci_idx
    ON users (lower(email))
    WHERE email IS NOT NULL;

-- Maps an application user to an identity at an arbitrary external provider.
-- Examples of provider values: github, gitlab, bitbucket, gitea.
CREATE TABLE external_identities (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    provider TEXT NOT NULL,
    provider_user_id TEXT NOT NULL,
    username TEXT,
    display_name TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_user_id),
    UNIQUE (user_id, provider)
);

CREATE INDEX external_identities_user_idx
    ON external_identities(user_id);

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
