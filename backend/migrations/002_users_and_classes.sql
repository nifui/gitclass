CREATE TABLE IF NOT EXISTS github_organization (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_org_id BIGINT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE IF NOT EXISTS class (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name TEXT NOT NULL,
    course_code TEXT,
    term TEXT,
    organization_id UUID REFERENCES github_organization(id) ON DELETE SET NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS app_user (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_user_id BIGINT UNIQUE,
    username TEXT NOT NULL,
    display_name TEXT,
    email TEXT,
    role user_role NOT NULL DEFAULT 'STUDENT',
    grade NUMERIC(5,2),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_login_at TIMESTAMPTZ
);

CREATE TABLE IF NOT EXISTS class_member (
    class_id UUID NOT NULL REFERENCES class(id) ON DELETE CASCADE,
    user_id UUID NOT NULL REFERENCES app_user(id) ON DELETE CASCADE,
    role user_role NOT NULL DEFAULT 'STUDENT',
    joined_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    left_at TIMESTAMPTZ,
    PRIMARY KEY (class_id, user_id)
);

CREATE INDEX IF NOT EXISTS idx_class_member_user
    ON class_member(user_id);

CREATE INDEX IF NOT EXISTS idx_class_member_active
    ON class_member(class_id, user_id)
    WHERE left_at IS NULL;

CREATE UNIQUE INDEX IF NOT EXISTS idx_app_user_username_ci
    ON app_user (lower(username));
