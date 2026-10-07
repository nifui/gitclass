use std::error::Error;
pub enum Task {
    Workflow,
    Job,
}
pub trait TaskQueue {
    type Error: Error + Send + Sync + 'static;
    // Push a new task into the queue
    async fn enqueue(&mut self, task: Task) -> Result<String, Self::Error>;

    // Fetch the next available task (returns a unique task ID and the payload)
    async fn dequeue(&mut self) -> Result<Option<(String, Task)>, Self::Error>;

    // Mark a task as successfully processed
    async fn ack(&mut self, task_id: &str) -> Result<(), Self::Error>;

    // Optional: Return a task to the queue or move it to a dead-letter queue on failure
    async fn nack(&mut self, task_id: &str) -> Result<(), Self::Error>;
}
