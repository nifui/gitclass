use std::collections::HashMap;

#[derive(Debug, Default, Clone)]
pub struct Task<'a> {
    pub dependencies: &'a str,
    pub name: &'a str,
    ///Has the task been resolved? Meaning it's depedencies have been resolved?
    pub resolved: bool,
    pub command: String,
    pub environment: Vec<(String, String)>,
}

#[derive(Debug, Default, Clone)]
pub struct TaskDirectory<'a> {
    pub task_map: HashMap<&'a str, Task<'a>>,
}
//Must convert from the instructions to actual tasks to be executed.
impl<'a> TaskDirectory<'a> {
    fn new() -> Self {
        Self::default()
    }
    fn add_task() -> Option<()> {
        Some(())
    }
}
