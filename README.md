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
pm/index.yml
pm/tasks/001-first-task.yml
pm/tasks/002-foo.yml
```

The index file contains global information about the project, including the title and a
basic description. It also contains the status of each task:


```yaml
meta:
  name: My first project
tasks:
- id: 1
  status: Doing
  changes:
  - from: Todo
    to: Doing
    on: 2021-01-01T00:00:00
- id: 2
  status: Done
  changes:
  - from: Todo
    to: Doing
    on: 2021-01-01T00:00:00
  - from: Doing
    to: Done
    on: 2021-02-01T00:00:00
```

Each task file contains information that's specific about the current task.

```yaml
title: Task title
description: |
This is the description
```

vim: tw=88:nowrap
