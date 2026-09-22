-- Provider-side organizations/groups/workspaces.
CREATE TABLE external_organizations (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    provider TEXT NOT NULL,
    provider_org_id TEXT NOT NULL,
    name TEXT NOT NULL,
    slug TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (provider, provider_org_id)
);

CREATE INDEX external_organizations_provider_idx
    ON external_organizations(provider);


