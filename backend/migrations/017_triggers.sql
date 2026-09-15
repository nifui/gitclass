CREATE OR REPLACE FUNCTION set_updated_at()
RETURNS TRIGGER
LANGUAGE plpgsql
AS $$
BEGIN
    NEW.updated_at = now();
    RETURN NEW;
END;
$$;

DROP TRIGGER IF EXISTS trg_class_updated_at ON class;
CREATE TRIGGER trg_class_updated_at
BEFORE UPDATE ON class
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_app_user_updated_at ON app_user;
CREATE TRIGGER trg_app_user_updated_at
BEFORE UPDATE ON app_user
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_repository_updated_at ON repository;
CREATE TRIGGER trg_repository_updated_at
BEFORE UPDATE ON repository
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_assignment_updated_at ON assignment;
CREATE TRIGGER trg_assignment_updated_at
BEFORE UPDATE ON assignment
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_assignment_student_updated_at ON assignment_student;
CREATE TRIGGER trg_assignment_student_updated_at
BEFORE UPDATE ON assignment_student
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_workflow_updated_at ON workflow;
CREATE TRIGGER trg_workflow_updated_at
BEFORE UPDATE ON workflow
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_grade_updated_at ON grade;
CREATE TRIGGER trg_grade_updated_at
BEFORE UPDATE ON grade
FOR EACH ROW EXECUTE FUNCTION set_updated_at();

DROP TRIGGER IF EXISTS trg_rubric_grade_updated_at ON rubric_grade;
CREATE TRIGGER trg_rubric_grade_updated_at
BEFORE UPDATE ON rubric_grade
FOR EACH ROW EXECUTE FUNCTION set_updated_at();
