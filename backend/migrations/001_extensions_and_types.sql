CREATE EXTENSION IF NOT EXISTS pgcrypto;

CREATE TYPE system_role AS ENUM (
    'USER',
    'ADMIN'
);

CREATE TYPE class_member_role AS ENUM (
    'TEACHER',
    'STUDENT'
);

CREATE TYPE assignment_status AS ENUM (
    'DRAFT',
    'PUBLISHED',
    'CLOSED',
    'ARCHIVED'
);

CREATE TYPE assignment_student_status AS ENUM (
    'ASSIGNED',
    'STARTED',
    'SUBMITTED',
    'GRADED',
    'EXEMPT'
);

CREATE TYPE grading_type AS ENUM (
    'AUTOMATIC',
    'MANUAL',
    'MIXED'
);

CREATE TYPE submission_status AS ENUM (
    'PENDING',
    'RUNNING',
    'PASSED',
    'FAILED',
    'GRADED'
);

CREATE TYPE workflow_run_status AS ENUM (
    'QUEUED',
    'RUNNING',
    'PASSED',
    'FAILED',
    'TIMEOUT',
    'ERROR',
    'CANCELLED'
);

CREATE TYPE test_result_status AS ENUM (
    'PASSED',
    'FAILED',
    'ERROR',
    'SKIPPED'
);

CREATE TYPE workflow_source_type AS ENUM (
    'LOCAL',
    'GIT',
    'URL',
    'INLINE'
);

CREATE TYPE job_status AS ENUM (
    'QUEUED',
    'RUNNING',
    'SUCCESS',
    'FAILED'
);

CREATE TYPE notification_type AS ENUM (
    'ASSIGNMENT_PUBLISHED',
    'ASSIGNMENT_DUE_SOON',
    'SUBMISSION_GRADED',
    'WORKFLOW_FAILED',
    'ASSIGNMENT_OVERDUE'
);

CREATE TYPE audit_action AS ENUM (
    'USER_CREATED',
    'CLASS_CREATED',
    'ASSIGNMENT_CREATED',
    'ASSIGNMENT_PUBLISHED',
    'DUE_DATE_CHANGED',
    'STUDENT_SUBMITTED',
    'GRADE_CHANGED',
    'REPOSITORY_DELETED',
    'WORKFLOW_EXECUTED'
);

CREATE TYPE audit_entity_type AS ENUM (
    'USER',
    'CLASS',
    'ASSIGNMENT',
    'REPOSITORY',
    'SUBMISSION',
    'WORKFLOW',
    'GRADE'
);

CREATE TYPE log_stream AS ENUM (
    'STDOUT',
    'STDERR'
);
