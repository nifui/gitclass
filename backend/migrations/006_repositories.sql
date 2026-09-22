-- A repository is an application entity, but its canonical identity belongs
-- to an external Git provider.
CREATE TABLE repositories (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    provider_repo_id TEXT NOT NULL,
    organization_id UUID
        REFERENCES external_organizations(id) ON DELETE SET NULL,
    name TEXT NOT NULL,
    full_name TEXT,
    clone_url TEXT,
    web_url TEXT,
    default_branch TEXT,
    is_private BOOLEAN NOT NULL DEFAULT TRUE,
    archived_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_repo_id)
);

CREATE INDEX repositories_organization_idx
    ON repositories(organization_id);

CREATE INDEX repositories_provider_idx
    ON repositories(provider);
