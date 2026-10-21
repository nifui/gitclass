CREATE TABLE class_assignments (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    class_id UUID NOT NULL REFERENCES classes(id) ON DELETE CASCADE,
    assignment_id UUID NOT NULL REFERENCES assignments(id) ON DELETE CASCADE,
    assigned_at TIMESTAMPTZ,
    due_at TIMESTAMPTZ,
    status assignment_status NOT NULL DEFAULT 'DRAFT',
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),

    UNIQUE (class_id, assignment_id),
    CHECK (due_at IS NULL OR assigned_at IS NULL OR due_at >= assigned_at)
);

CREATE INDEX class_assignments_class_idx ON class_assignments(class_id);
CREATE INDEX class_assignments_assignment_idx ON class_assignments(assignment_id);
CREATE INDEX class_assignments_due_idx ON class_assignments(due_at);

ALTER TABLE assignments
    DROP CONSTRAINT IF EXISTS assignments_class_id_fkey,
    DROP COLUMN class_id,
    DROP COLUMN assigned_at,
    DROP COLUMN due_at;

-- Drop obsolete indexes from the assignments table
DROP INDEX IF EXISTS assignments_class_idx;
DROP INDEX IF EXISTS assignments_due_idx;


-- 3. Update assignment_students to reference class_assignments (class instances)
-- Student submissions and statuses belong to the specific class instance of an assignment.
ALTER TABLE assignment_students
    DROP CONSTRAINT IF EXISTS assignment_students_assignment_id_fkey;

   -- Rename foreign key column to reflect the new relationship
ALTER TABLE assignment_students
    RENAME COLUMN assignment_id TO class_assignment_id;

ALTER TABLE assignment_students
    ADD CONSTRAINT assignment_students_class_assignment_id_fkey
    FOREIGN KEY (class_assignment_id) REFERENCES class_assignments(id) ON DELETE CASCADE;

-- Recreate indexes for assignment_students
DROP INDEX IF EXISTS assignment_students_status_idx;
CREATE INDEX assignment_students_status_idx
    ON assignment_students(class_assignment_id, status);

-- 4. Update assignment_repositories (Optional / Recommended)
-- If starter repositories are class-instance specific rather than global template defaults, 
-- update assignment_repositories to point to class_assignments as well.
ALTER TABLE assignment_repositories
    DROP CONSTRAINT IF EXISTS assignment_repositories_assignment_id_fkey;

ALTER TABLE assignment_repositories
    RENAME COLUMN assignment_id TO class_assignment_id;

ALTER TABLE assignment_repositories
    ADD CONSTRAINT assignment_repositories_class_assignment_id_fkey
    FOREIGN KEY (class_assignment_id) REFERENCES class_assignments(id) ON DELETE CASCADE;

DROP INDEX IF EXISTS assignment_repositories_assignment_student_idx;
CREATE INDEX assignment_repositories_class_assignment_student_idx
    ON assignment_repositories(class_assignment_id, student_id);
