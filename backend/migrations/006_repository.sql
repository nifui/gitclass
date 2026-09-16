
CREATE TABLE IF NOT EXISTS repository (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    git_repo_id BIGINT NOT NULL UNIQUE,
    organization_id UUID REFERENCES git_organization(id) ON DELETE SET NULL,
    owner TEXT NOT NULL,
    name TEXT NOT NULL,
    full_name TEXT NOT NULL UNIQUE,
    url TEXT NOT NULL,
    default_branch TEXT NOT NULL DEFAULT 'main',
    is_private BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    archived_at TIMESTAMPTZ
);

CREATE INDEX IF NOT EXISTS idx_repository_org
    ON repository(organization_id);
