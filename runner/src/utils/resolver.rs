use crate::utils::tasks::Task;

pub enum TaskError {}

pub fn resolve_task(task: &Task) -> Result<(), ()> {
    //Validate the Task prior to execution.
    //
    Ok(())
}
//This will only validate resources and not the steps.
//If one of the steps does not execute properly this is handled by the executor.
//Since we allow the users to specify a runtime, we should ask them to validate that the images
//being installed are valid and can be used for the specified case.
pub fn validate_task(task: &Task) -> Result<(), TaskError> {
    Ok(())
}
pub fn execute_task() {}
