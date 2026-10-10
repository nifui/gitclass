-- Required as the referenced key for the composite grade -> submission FK.
-- (id is already unique, but PostgreSQL requires a unique key matching the
-- exact referenced column list.)
ALTER TABLE submissions
    ADD CONSTRAINT submissions_assignment_student_id_uq
    UNIQUE (assignment_id, student_id, id);

-- A grade may refer to a submission only when its assignment and student
-- match that submission. Replace the existing single-column FK so deletion
-- only nulls submission_id, leaving assignment_id and student_id intact.
ALTER TABLE grades
    DROP CONSTRAINT IF EXISTS grades_submission_id_fkey;

ALTER TABLE grades
    ADD CONSTRAINT grades_submission_consistency_fk
    FOREIGN KEY (assignment_id, student_id, submission_id)
    REFERENCES submissions (assignment_id, student_id, id)
    ON DELETE SET NULL (submission_id);

-- A submission must belong to a student assigned to that assignment.
ALTER TABLE submissions
    ADD CONSTRAINT submissions_assignment_student_fk
    FOREIGN KEY (assignment_id, student_id)
    REFERENCES assignment_students (assignment_id, student_id);

ALTER TABLE rubrics
    ADD CONSTRAINT rubrics_id_assignment_uq
    UNIQUE (id, assignment_id);

-- Populate assignment_id from each rubric before enforcing NOT NULL.
ALTER TABLE rubric_grades
    ADD COLUMN assignment_id UUID;

UPDATE rubric_grades AS rg
SET assignment_id = r.assignment_id
FROM rubrics AS r
WHERE r.id = rg.rubric_id;

-- Abort clearly if any rubric grade could not be associated with a rubric.
DO $$
BEGIN
    IF EXISTS (
        SELECT 1
        FROM rubric_grades
        WHERE assignment_id IS NULL
    ) THEN
        RAISE EXCEPTION
            'Cannot migrate rubric_grades: one or more rows have no matching rubric';
    END IF;
END;
$$;

ALTER TABLE rubric_grades
    ALTER COLUMN assignment_id SET NOT NULL;

ALTER TABLE rubric_grades
    ADD CONSTRAINT rubric_grades_rubric_assignment_fk
    FOREIGN KEY (rubric_id, assignment_id)
    REFERENCES rubrics (id, assignment_id)
    ON DELETE CASCADE;

-- A rubric grade can only exist for a student assigned to the rubric's
-- assignment. Cascade deletion of the assignment-student row to its grades.
ALTER TABLE rubric_grades
    ADD CONSTRAINT rubric_grades_assignment_student_fk
    FOREIGN KEY (assignment_id, student_id)
    REFERENCES assignment_students (assignment_id, student_id)
    ON DELETE CASCADE;


ALTER TABLE external_organizations
    ADD CONSTRAINT external_organizations_provider_id_uq
    UNIQUE (provider, id);

-- Replace the existing organization_id FK. The composite FK enforces that
-- the repository's provider matches the organization's provider. On
-- organization deletion, only organization_id is nulled.
ALTER TABLE repositories
    DROP CONSTRAINT IF EXISTS repositories_organization_id_fkey;

ALTER TABLE repositories
    ADD CONSTRAINT repositories_organization_provider_fk
    FOREIGN KEY (provider, organization_id)
    REFERENCES external_organizations (provider, id)
    ON DELETE SET NULL (organization_id);

-- A submission's repository must be associated with its assignment.
ALTER TABLE submissions
    ADD CONSTRAINT submissions_assignment_repository_fk
    FOREIGN KEY (assignment_id, repository_id)
    REFERENCES assignment_repositories (assignment_id, repository_id);


ALTER TABLE workflow_runs
    ADD CONSTRAINT workflow_runs_state_timestamps_ck
    CHECK (
        (
            status = 'QUEUED'
            AND started_at IS NULL
            AND finished_at IS NULL
        )
        OR
        (
            status = 'RUNNING'
            AND started_at IS NOT NULL
            AND finished_at IS NULL
        )
        OR
        (
            status IN (
                'PASSED', 'FAILED', 'TIMEOUT', 'ERROR', 'CANCELLED'
            )
            AND started_at IS NOT NULL
            AND finished_at IS NOT NULL
        )
    );

COMMIT;
