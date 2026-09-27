//Deserialize into struct, validate the workflow, produce ValidatedWorkflow, execute workflow,
//A task can be thought of as it's own configurable container with its own resources allocated to
//it. The steps are what actually gets executed in the container. It's possible for other containers
//to require another container's output in another step.
//In this case the task handler will try to satisfy those constraints.

use std::marker::PhantomData;

use serde::Deserialize;
use time::OffsetDateTime;

use std::collections::HashMap;

#[derive(Deserialize, Debug, Default)]
pub struct Task {
    #[serde(default)]
    pub metadata: Metadata,
    pub runtime: Runtime,
    pub dependencies: Dependencies,
    pub network: Network,
    pub filesystem: Filesystem,

    #[serde(default = "default_limits")]
    pub limits: Limits,
    pub environment: Environment,
    pub artifacts: Artifacts,
    pub cleanup: Cleanup,
    pub steps: Vec<Step>,
}

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub name: String,
    pub version: u32,
    pub date: OffsetDateTime,
}
impl Default for Metadata {
    fn default() -> Self {
        Self {
            name: String::default(),
            version: u32::default(),
            //Chose the first const I see, will replace later
            date: OffsetDateTime::UNIX_EPOCH,
        }
    }
}

#[derive(Deserialize, Debug, Default)]
pub struct Runtime {
    pub language: String,
    pub version: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Dependencies {
    pub file: String,
    pub sha256: String,
}

#[derive(Deserialize, Debug, Default)]
pub struct Network {
    pub enabled: bool,
    pub protocol: NetworkProtocol,
    //Might be the wrong type to represent a port?
    //Should hex or ints be used.
    pub ports: Vec<u32>,
}

#[derive(Default, Deserialize, Debug)]
pub enum NetworkProtocol {
    #[default]
    Tcp,
    Udp,
}

#[derive(Deserialize, Debug, Default)]
pub struct Filesystem {
    pub mode: String,
    pub read_only: Vec<String>,
}
pub enum FilesystemMode {}

#[derive(Deserialize, Debug, Default)]
pub struct Limits {
    pub timeout_seconds: u64,
    pub cpu_seconds: u64,
    pub memory_mb: u64,
    pub disk_mb: u64,
    pub processes: u32,
    pub output_mb: u64,
}
pub const fn default_limits() -> Limits {
    Limits {
        timeout_seconds: 10,
        cpu_seconds: 10,
        memory_mb: 10,
        disk_mb: 10,
        processes: 10,
        output_mb: 10,
    }
}
#[derive(Deserialize, Debug, Default)]
pub struct Environment {
    pub variables: HashMap<String, String>,
    //Secrets are provided by the host system and not exposed to the user.
    //This could be for something like an API.
    pub secrets: HashMap<String, String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Artifacts {
    pub include: Vec<String>,
}

#[derive(Deserialize, Debug, Default)]
pub struct Cleanup {
    pub workspace: bool,
    pub processes: bool,
}

#[derive(Deserialize, Debug, Default)]
pub struct Step {
    pub name: String,
    #[serde(rename = "type")]
    pub step_type: String,
    pub command: Option<Vec<String>>,
    pub timeout_seconds: Option<u64>,
    pub requires: Option<Vec<String>>,
}
//Spawn a container with the specified configs.
//Specify lfietime of the package
#[derive(Debug, Default, Clone)]
pub struct ExecutableTask {}

pub trait Executor {}

#[derive(Debug, Default, Clone)]
pub struct TaskQueue<S>
where
    S: Executor,
{
    //Since we want to avoid blocking of a thread and maximize usage of resources available to us we
    //should keep a queue of workers alloted.
    //This can be controlled with a .config or at compile time.
    pub workers: Vec<S>,
    pub tasks: Vec<ExecutableTask>,
}
//

struct TaskDistributor<S>
where
    S: Executor,
{
    __p: PhantomData<S>,
}
impl<S> TaskDistributor<S>
where
    S: Executor,
{
    fn verify_task(&mut self, task: Task) {
        //A ring buffer will be used to minimize excessively nested steps/tasks.
    }
}

impl<S> TaskQueue<S>
where
    S: Executor,
{
    pub fn new() -> Self {
        Self {
            workers: Vec::default(),
            tasks: Vec::default(),
        }
    }
    pub fn add_task() {}
    //Maybe leave it to tokio to execute tasks, but batching of similar tasks should be done first.
}

#[cfg(target_feature = "logging")]
pub struct TaskLog {
    map: HashMap<String, String>,
}

//Since arbitrary execution of unknown tasks would be dumb, the amount of executors are statically
//defined. Because of this we can have seperate executor queues for each type of executor.
//This allows us to not only avoid wasting memory due to how tagged unions work, but work is also
//properly distributed in a way that it can benefit from individual caching and optimizations of each pipeline.
