CREATE TABLE IF NOT EXISTS app_user (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    git_user_id BIGINT UNIQUE,
    username TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    role user_role NOT NULL DEFAULT 'STUDENT',
    grade NUMERIC(5,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);


