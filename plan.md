-- Notes 
Rate limit of around 1000 to 5000 
Shouldn't exceed it with regular usage but good to keep in mind when optimizing requests to the REST api. 
Utilize multithreading to execute workflows/tests for code. 
Requires some idea of the environment and the layout of the code. 
Language must support testing in some way. 
Language workflow implementation left to admin/users.
Automated jobs for cleaning up the classroom (Or leave to users if deletions that happen are infrequent)

-- Idea
Github classroom clone built on top of GitHub.

-- Tables

Refresh Tokens: 
    user_id: F_KEY(User)
    token_hash: STRING
    expires_at: TIMESTAMP 
    created_at: TIMESTAMP

User: 
    created_at: TIMESTAMP 
    updated_at: TIMESTAMP
    id: UNIQUE KEY
    username/name: STRING, 
    perms: ENUM(ADMIN | STUDENT) || FLAGS(If customization is needed)
    class_id: ID,
    grade: NUMBER
    

Assignments: 
    repo: F_KEY(Repository)
    assignment_id: UNIQUE KEY
    assigned: TIMESTAMP
    due_date: TIMESTAMP
    total_points: NUMBER

Repository: 
    repository_id: UNIQUE KEY
    link: STRING(url/path)
    workflows: F_KEY(Workflows)
    -- some metadata(Unsure yet)
    

Workflows: 
    workflow_id: UNIQUE KEY
    path: STRING(LOCAL/URL)
    assignment_id: UNIQUE KEY

Junction Tables: 
    Assignment Student Metadata: 
        date_completed
        student_id: F_KEY TO USER
        assignment_id: F_KEY TO USER
        submission_date: TIMESTAMP
        recived_points: NUMBER

-- Backend features
Protected routes backed by API access to the Github Organization that this is built on.
Admins have access to the entire database(Maybe add manager roles incase more members are needed to properly manage a classroom)
If a seperate classroom is needed spin up another database or something. 

-- Optional 
Build on top of self github 


For executing jobs like removing stale data, turnining in assignments, a task queue should be employed. 
Cron jobs work for single one off things but they lack proper synchronization.
