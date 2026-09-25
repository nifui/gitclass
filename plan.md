-- Notes

Rate limit of around 1000 to 5000 Shouldn't exceed it with regular usage but
good to keep in mind when optimizing requests to the REST api. Utilize
multithreading to execute workflows/tests for code. Requires some idea of the
environment and the layout of the code. Language must support testing in some
way. Language workflow implementation left to admin/users. Automated jobs for
cleaning up the classroom (Or leave to users if deletions that happen are
infrequent)

-- Backend features

Protected routes backed by API access to the Github Organization that this is
built on. Admins have access to the entire database(Maybe add manager roles
incase more members are needed to properly manage a classroom) If a seperate
classroom is needed spin up another database or something.

-- Optional

Build on top of self github

For executing jobs like removing stale data, turnining in assignments, a task
queue should be employed. Cron jobs work for single one off things but they lack
proper synchronization.

For workflows, we can prevent a majority of security issues by restricting
packages as those tend to be a source of vulnerabilities out of our control.
Since the issue is not the workflow itself executing malicious code through
direct commands but rather code supplied by the user being tested, that should
be vetted more heavily. The admin can declare the package file. When the code is
being tested or worked on by the workflow, it checks to ensure the package file
is the same. While this is a bit more tedious it avoids the annoyance and
resource consumption of spinning up a VM. Since this is going to be used in a
classroom setting, this is an acceptable comrpomise.

TOML vs YAML TOML seems to be a much more strict and therefore safer language,
but YAML is much more raw in it's capabilities hence why GitHub uses it. Using
TOML doesn't seem to be that much of a feature loss to me so far, but if needed,
the switch to YAML shouldn't be too bad.

Elaborating on the package locking, we can even use this for caching/mirror of
packages.

Students could also remotely test their code at home if needed. Only issue with
this is rarely would this ever happen as the code grows more complex and testing
is more subjective than objective. For this shifting to an idea of a
reproducible execution platform might be a more favorable option. Basic
requirements might be:

- Can be compiled.
- Actually run the program.
- Just test the program.

We could break the pipeline up into stages. While we wait for a human to grade
the subjective part, we can handle the objective parts like does a file exist,
does the program run and last, does it exceed a certain limit,

These might be out of the depth of the project, but some form of ML could be
implemented to handle some parts of the subjective grading aspect especially for
web apps.

## Thoughts
Should we allow individual steps to be defined in their own file? 
I think no due to the fact that some steps inherently rely on certain limits being established for safe execution.
Logs are most likely going to be the most important part of this whole thing as it allows errors to be visually seen. 
Log levels - (CRITICAL, WARNING, DEBUG, INFO)
Might fuse certain levels or add more dependeing on overall usage. 
Timestampled and with callbacks detailing where. 
Should be configurable via conditional compilation
If conditional compilation is a bad thing, we can just runtime switch statements.

I think we should seperate steps from the actual container configuration, 
Containers can be reused but steps are more versatile. 
Instead of implicit defaults or explicit user defined configuration options, we 
can expose an option that allows the configuration to be left to the schelduler.

Gonna use Redis in case we want this backend to be distributed later on. 
Moka would work for lightweight implementation thats non distributed.
Session table might be missing something. 



