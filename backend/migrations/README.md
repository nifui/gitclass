# Git Classroom replacement — PostgreSQL migrations

## Migration order

Run the files in lexical order:

1. `001_extensions_and_types.sql`
2. `002_users_and_external_identities.sql`
3. `003_external_organizations_and_repositories.sql`
4. `004_classes_and_memberships.sql`
5. `005_assignments_and_repositories.sql`
6. `006_submissions_and_grading.sql`
7. `007_workflows_and_execution.sql`
8. `008_jobs_notifications_audit.sql`
9. `009_auth_and_triggers.sql`

These are forward migrations and are intended to run once, in order, using the
migration mechanism of the application (Flyway, Liquibase, Prisma, Drizzle,
Goose, Rails migrations, etc.).

## Design decisions

### Provider neutrality

Git providers are integration details rather than domain tables.

- `external_identities` maps an application user to a provider identity.
- `provider_connections` stores provider credentials separately from identity data.
- `external_organizations` represents provider-side organizations/groups.
- `repositories` stores provider + provider repository ID rather than assuming
  a GitHub numeric ID.

Do not add provider-specific columns such as `github_user_id` or
`gitlab_project_id` to domain tables.

### User roles

`users.system_role` only represents application-wide administration.

`class_members.role` represents the user's role within a class. This avoids
making "teacher" or "student" a global property of a person.

### Repository metadata

`provider_repo_id` is the stable external identity. URLs, branch names,
organization membership, and display names are cached metadata and can be
refreshed from the provider API.

### Submission model

A submission records the repository and exact commit SHA used for grading.
`is_late` was removed because lateness is derivable from `submitted_at` and
the assignment due date.

### Grading

`grades` is intentionally not unique on `(assignment_id, student_id)`.
This permits grade history/revisions. The application can determine the current
grade by its own policy, for example latest grade.

### Workflow execution

Workflows are application-level execution definitions. A workflow run belongs
to a submission, and test results/logs belong to that run.

### Removed from the original schema

The original file contained duplicate definitions of many tables, references
to both `app_user` and `app_users`, references to both `repository` and
`repositories`, and several `DROP TABLE repositories` operations. Those are
not migration-safe and have been removed.

The original `git_user_id`, `git_org_id`, and `git_repo_id` fields were also
removed because they hard-code a single provider and use provider-specific
numeric identifiers.

The old `repository.owner`, `full_name UNIQUE`, and mandatory `url` constraints
were removed. Those assumptions do not hold uniformly across Git providers.

`assignment_student.late` was removed because it duplicates information that
can be calculated from the assignment due date and submission timestamp.

## One application-level assumption

The encrypted token columns contain ciphertext produced by the application or
a secrets service. The database schema does not pretend that storing plaintext
OAuth tokens is safe.

The `provider` column is deliberately free-form. If the application wants a
controlled provider registry, add a `providers` table later rather than using
a PostgreSQL enum; that allows providers to be added without a schema change.
