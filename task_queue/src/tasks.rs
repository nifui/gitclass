use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub enum QueueError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ClaimToken(Uuid);

impl ClaimToken {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TaskId(pub Uuid);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct WorkflowRunId(pub Uuid);
pub trait Task: Send + Sync {
    fn id(&self) -> TaskId;

    fn workflow_run_id(&self) -> Option<WorkflowRunId>;

    fn dependencies(&self) -> &[TaskId];

    fn priority(&self) -> i32;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskSpec {
    Job(JobSpec),
}
#[derive(Debug, Clone)]
pub struct ScheduledTask {
    pub id: TaskId,
    pub workflow_run_id: Option<WorkflowRunId>,
    pub dependencies: Vec<TaskId>,
    pub priority: i32,
    pub spec: TaskSpec,
}
pub trait TaskQueue: Send + Sync {
    async fn enqueue(&self, task: NewTask) -> Result<TaskId, QueueError>;

    async fn claim(&self, worker: WorkerId) -> Result<Option<ClaimedTask>, QueueError>;

    async fn renew_lease(&self, claim: &ClaimToken) -> Result<(), QueueError>;
    async fn complete(&self, claim: ClaimToken, result: TaskResult) -> Result<(), QueueError>;

    async fn fail(&self, claim: ClaimToken, error: TaskError) -> Result<(), QueueError>;
}
pub struct PostgresTaskQueue {
    pool: sqlx::PgPool,
}
#[cfg(feature = "redis")]
pub struct RedisTaskQueue {
    postgres: PostgresTaskQueue,
    redis: redis::Client,
}
