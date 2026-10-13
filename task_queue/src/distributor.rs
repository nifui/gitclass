use std::sync::Arc;
use std::time::Duration;

use thiserror::Error;
use tokio::task::JoinSet;
use tokio::time::{MissedTickBehavior, interval, sleep};
use tokio_util::sync::CancellationToken;
use tracing::{error, info, warn};

use crate::error::QueueResult;
use crate::queue::TaskQueue;
use crate::types::{ClaimOptions, ClaimedTask, TaskFailure, WorkerId};

/// Error returned by task handlers.
#[derive(Debug, Error)]
pub enum TaskError {
    /// Retry according to the task's retry policy.
    #[error("transient failure: {0}")]
    Transient(String),

    /// Dead-letter immediately, without consuming more retries.
    #[error("permanent failure: {0}")]
    Permanent(String),
}

/// Execution context handed to handlers.
#[derive(Debug, Clone)]
pub struct TaskContext {
    pub id: crate::types::TaskId,

    /// 1-based attempt number.
    pub attempt: u32,
}

/// Statically dispatched task router.
///
/// Implement this trait in the application to decode the task payload
/// and route it to the appropriate concrete handler.
///
/// No trait objects are required.
pub trait TaskDispatcher: Send + Sync + 'static {
    fn handle(&self, claimed: &ClaimedTask) -> impl Future<Output = Result<(), TaskError>> + Send;
}

/// Distributor tuning.
#[derive(Debug, Clone)]
pub struct DistributorConfig {
    pub worker_id: WorkerId,

    /// Maximum tasks executed concurrently.
    pub concurrency: usize,

    /// Maximum tasks claimed per polling iteration.
    pub batch_size: usize,

    /// Lease duration.
    pub lease_duration: Duration,

    /// How often the lease is renewed.
    pub heartbeat_interval: Duration,

    /// Poll interval when the queue is idle.
    pub poll_interval: Duration,
}

impl Default for DistributorConfig {
    fn default() -> Self {
        Self {
            worker_id: WorkerId::generate(),
            concurrency: 8,
            batch_size: 8,
            lease_duration: Duration::from_secs(60),
            heartbeat_interval: Duration::from_secs(20),
            poll_interval: Duration::from_millis(250),
        }
    }
}

/// Task distributor using compile-time generic dispatch.
///
/// Q is the concrete queue implementation.
/// D is the concrete task dispatcher.
///
/// Neither dependency is stored as a trait object.
pub struct TaskDistributor<Q, D>
where
    Q: TaskQueue,
    D: TaskDispatcher,
{
    queue: Arc<Q>,
    dispatcher: Arc<D>,
    config: DistributorConfig,
}

impl<Q, D> TaskDistributor<Q, D>
where
    Q: TaskQueue,
    D: TaskDispatcher,
{
    pub const fn new(queue: Arc<Q>, dispatcher: Arc<D>, config: DistributorConfig) -> Self {
        Self {
            queue,
            dispatcher,
            config,
        }
    }

    /// Claims and executes tasks until shutdown is requested.
    ///
    /// Stops claiming new tasks on shutdown, then drains in-flight work.
    pub async fn run(self: Arc<Self>, shutdown: CancellationToken) -> QueueResult<()> {
        info!(
            worker = %self.config.worker_id,
            "distributor started"
        );

        let concurrency = self.config.concurrency.max(1);
        let batch_size = self.config.batch_size.max(1);
        let poll_interval = self.config.poll_interval.max(Duration::from_millis(1));

        let mut inflight: JoinSet<()> = JoinSet::new();

        loop {
            if shutdown.is_cancelled() {
                break;
            }

            // Wait when the concurrency limit has been reached.
            if inflight.len() >= concurrency {
                tokio::select! {
                    joined = inflight.join_next() => {
                        Self::report_join(joined);
                    }
                    _ = shutdown.cancelled() => break,
                }

                continue;
            }

            let available = concurrency - inflight.len();
            let wanted = batch_size.min(available);

            let claim_result = tokio::select! {
                result = self.queue.claim(ClaimOptions {
                    worker_id: self.config.worker_id.clone(),
                    lease_duration: self.config.lease_duration,
                    max_tasks: wanted,
                }) => result,

                _ = shutdown.cancelled() => break,
            };

            let claimed = match claim_result {
                Ok(tasks) => tasks,

                Err(err) => {
                    warn!(
                        error = %err,
                        "claim failed; backing off"
                    );

                    tokio::select! {
                        _ = sleep(poll_interval) => {}
                        _ = shutdown.cancelled() => break,
                    }

                    continue;
                }
            };

            if claimed.is_empty() {
                tokio::select! {
                    _ = sleep(poll_interval) => {}
                    _ = shutdown.cancelled() => break,
                }

                continue;
            }

            for task in claimed {
                let distributor = Arc::clone(&self);

                inflight.spawn(async move {
                    distributor.process(task).await;
                });
            }
        }

        // Drain all already-started work. Each task continues to
        // heartbeat its lease while its handler is running.
        while let Some(joined) = inflight.join_next().await {
            Self::report_join(Some(joined));
        }

        info!(
            worker = %self.config.worker_id,
            "distributor stopped"
        );

        Ok(())
    }

    fn report_join(joined: Option<Result<(), tokio::task::JoinError>>) {
        match joined {
            Some(Ok(())) => {}

            Some(Err(err)) if err.is_panic() => {
                error!(panic = %err, "task worker panicked");
            }

            Some(Err(err)) => {
                warn!(error = %err, "task worker aborted");
            }

            None => {}
        }
    }

    /// Executes a claimed task, renews its lease, and records its outcome.
    async fn process(self: Arc<Self>, task: ClaimedTask) {
        let task_id = task.id();

        let queue = Arc::clone(&self.queue);
        let lease = self.config.lease_duration;

        let heartbeat_period = self.config.heartbeat_interval.max(Duration::from_millis(1));

        let mut heartbeat = interval(heartbeat_period);
        heartbeat.set_missed_tick_behavior(MissedTickBehavior::Delay);

        // Skip interval's immediate first tick. The lease was already
        // established by claim(), so the first renewal should occur
        // after heartbeat_period.
        heartbeat.tick().await;

        let dispatcher = Arc::clone(&self.dispatcher);

        let outcome = tokio::select! {
            result = dispatcher.handle(&task) => Some(result),

            _ = async {
                loop {
                    heartbeat.tick().await;

                    if let Err(err) = queue
                        .extend_lease(&task, lease)
                        .await
                    {
                        warn!(
                            task_id = %task_id,
                            error = %err,
                            "lease extension failed"
                        );
                    }
                }
            } => None,
        };

        match outcome {
            Some(Ok(())) => {
                if let Err(err) = self.queue.ack(task).await {
                    error!(
                        task_id = %task_id,
                        error = %err,
                        "ack failed; task may be re-executed"
                    );
                }
            }

            Some(Err(TaskError::Transient(reason))) => {
                if let Err(err) = self.queue.fail(task, TaskFailure::transient(reason)).await {
                    error!(
                        task_id = %task_id,
                        error = %err,
                        "failed to record transient task failure"
                    );
                }
            }

            Some(Err(TaskError::Permanent(reason))) => {
                if let Err(err) = self.queue.fail(task, TaskFailure::permanent(reason)).await {
                    error!(
                        task_id = %task_id,
                        error = %err,
                        "failed to record permanent task failure"
                    );
                }
            }

            None => {
                // The heartbeat future is designed to run indefinitely.
                // This branch is retained defensively.
                if let Err(err) = self.queue.release(task, None).await {
                    error!(
                        task_id = %task_id,
                        error = %err,
                        "failed to release task"
                    );
                }
            }
        }
    }
}
