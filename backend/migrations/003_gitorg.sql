CREATE TABLE IF NOT EXISTS github_organization (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    github_org_id BIGINT NOT NULL UNIQUE,
    name TEXT NOT NULL,
    url TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);


