CREATE TABLE refresh_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    token_hash TEXT NOT NULL UNIQUE,
    expires_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    revoked_at TIMESTAMPTZ,
    last_used_at TIMESTAMPTZ,
    device_name TEXT
);

CREATE INDEX refresh_tokens_user_idx
    ON refresh_tokens(user_id);

CREATE INDEX refresh_tokens_expiry_idx
    ON refresh_tokens(expires_at);

CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

CREATE TRIGGER users_updated_at
BEFORE UPDATE ON users
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER external_identities_updated_at
BEFORE UPDATE ON external_identities
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER provider_connections_updated_at
BEFORE UPDATE ON provider_connections
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER external_organizations_updated_at
BEFORE UPDATE ON external_organizations
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER repositories_updated_at
BEFORE UPDATE ON repositories
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER classes_updated_at
BEFORE UPDATE ON classes
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER assignments_updated_at
BEFORE UPDATE ON assignments
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER assignment_students_updated_at
BEFORE UPDATE ON assignment_students
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER workflows_updated_at
BEFORE UPDATE ON workflows
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER grades_updated_at
BEFORE UPDATE ON grades
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

CREATE TRIGGER rubric_grades_updated_at
BEFORE UPDATE ON rubric_grades
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
