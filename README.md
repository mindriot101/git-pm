# Git project management

*A kanban board tracked by your git repository.*

## Usage

### Start tracking project tasks

`git pm init`

This creates a `pm` directory which contains the project task state.

### Adding tasks
#### Add a new task to the backlog

`git pm add "Do something"`

#### Add a task with a label

`git pm add "Do something :high-priority:`

### Showing the current status

`git pm`

```
Todo
----
1. Do something :high-priority:

Doing
-----

Done
----
```

## Implementation

The state is all contained in a `pm` directory at the top level of the current git
repository.

### State

```
pm/index.toml
pm/tasks/001-first-task.toml
pm/tasks/002-foo.toml
```

The index file contains global information about the project, including the title and a
basic description. It also contains the status of each task:


```toml
[meta]
name = "My first project"

[[tasks]]
id = 1
status = "doing"

[[tasks.changes]]
from = "todo"
to = "doing"
on = "2021-01-01T00:00:00"

[[tasks]]
id = 2
status = "done"

[[tasks.changes]]
from = "todo"
to = "doing"
on = "2021-01-01T00:00:00"

[[tasks.changes]]
from = "doing"
to = "done"
on = "2021-02-01T00:00:00"
```

Each task file contains information that's specific about the current task.

```toml
title = "Task title"
description = """
This is the description
"""
```

vim: tw=88:nowrap
