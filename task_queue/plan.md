SQL Schema: 

task_id
- Which task is being executed

attempt_number
- Which execution attempt this is

claim_token
- Identifies this specific claim

worker_id
- Worker that owns the claim

lease_expires_at
- When the claim expires

status
- Current state of the attempt

The Task queue should really only act as a generic message broker that allows users to specify whether Redis or Postgres is used. 
It should provide a Task trait that can be implemented on any type that can be schelduled and executed. 
The scheduler method should return some sort of stats that allow us to apply a criteria for the task to be schelduled. 
Maybe use a Stats struct that must be returned and then allow users to pick an algorithm for ranking tasks to be executed. 
The execute method should allow taking in the parameter state of type T which allows users to supply their own state.
