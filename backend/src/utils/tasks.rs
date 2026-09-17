//For the general structure we should define a parser.

use std::marker::PhantomData;

#[derive(Debug, Default, Clone)]
pub struct ExecutableTask {
    pub dependencies: Vec<String>,
}

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
impl<S> TaskDistributor<S> where S: Executor {}

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

//Since arbitrary execution of unknown tasks would be dumb, the amount of executors are statically
//defined. Because of this we can have seperate executor queues for each type of executor.
//This allows us to not only avoid wasting memory due to how tagged unions work, but work is also
//properly distributed in a way that it can benefit from individual caching and optimizations of each pipeline.
