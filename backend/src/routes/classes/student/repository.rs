use sqlx::{Executor, Postgres};
use uuid::Uuid;

pub async fn get_student_info<'e, E>(executor: E) -> ()
where
    E: Executor<'e, Database = Postgres>,
{
}
// Returns info
///Returns info about the assignments that a student has. Limited only to the current class.
pub async fn get_student_assignments<'e, E>(executor: E, student_id: Uuid) -> ()
where
    E: Executor<'e, Database = Postgres>,
{
}
/// Returns info on the specific assignment belonging to a student within the class.
pub async fn get_student_assignment<'e, E>(executor: E, stdudent_id: Uuid) -> ()
where
    E: Executor<'e, Database = Postgres>,
{
}

pub async fn remove_student<'e, E>(executor: E, student_id: Uuid) -> ()
where
    E: Executor<'e, Database = Postgres>,
{
}
