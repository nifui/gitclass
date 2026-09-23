CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    username TEXT NOT NULL,
    display_name TEXT NOT NULL,
    email TEXT NOT NULL,
    password_hash TEXT NOT NULL,
    system_role system_role NOT NULL DEFAULT 'USER',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ,
    auth_version INTEGER NOT NULL DEFAULT 0
);

CREATE UNIQUE INDEX users_username_ci_idx ON users (lower(username));
CREATE UNIQUE INDEX users_email_ci_idx
    ON users (lower(email))
    WHERE email IS NOT NULL;


