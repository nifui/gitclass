CREATE EXTENSION IF NOT EXISTS pgcrypto;

DO $$ BEGIN
    CREATE TYPE user_role AS ENUM ('ADMIN', 'TEACHER', 'STUDENT');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE assignment_status AS ENUM ('DRAFT', 'PUBLISHED', 'CLOSED', 'ARCHIVED');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE assignment_student_status AS ENUM (
        'ASSIGNED', 'STARTED', 'SUBMITTED', 'GRADED', 'LATE', 'EXEMPT'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE workflow_source_type AS ENUM ('LOCAL', 'GIT', 'URL', 'INLINE');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE workflow_run_status AS ENUM (
        'QUEUED', 'RUNNING', 'PASSED', 'FAILED', 'TIMEOUT', 'ERROR', 'CANCELLED'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE test_result_status AS ENUM ('PASSED', 'FAILED', 'ERROR', 'SKIPPED');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE submission_status AS ENUM (
        'PENDING', 'RUNNING', 'PASSED', 'FAILED', 'GRADED'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE grading_type AS ENUM ('AUTOMATIC', 'MANUAL', 'MIXED');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE job_status AS ENUM ('QUEUED', 'RUNNING', 'SUCCESS', 'FAILED');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE notification_type AS ENUM (
        'ASSIGNMENT_PUBLISHED',
        'ASSIGNMENT_DUE_SOON',
        'SUBMISSION_GRADED',
        'WORKFLOW_FAILED',
        'ASSIGNMENT_OVERDUE'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE audit_action AS ENUM (
        'TEACHER_CREATED_ASSIGNMENT',
        'TEACHER_CHANGED_DUE_DATE',
        'STUDENT_SUBMITTED',
        'TEACHER_CHANGED_GRADE',
        'ADMIN_ADDED_USER',
        'REPOSITORY_DELETED',
        'WORKFLOW_EXECUTED'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE audit_entity_type AS ENUM (
        'USER', 'CLASS', 'ASSIGNMENT', 'REPOSITORY',
        'SUBMISSION', 'WORKFLOW', 'GRADE'
    );
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;

DO $$ BEGIN
    CREATE TYPE log_stream AS ENUM ('STDOUT', 'STDERR');
EXCEPTION WHEN duplicate_object THEN NULL;
END $$;
