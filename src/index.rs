use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::fmt;
use std::io::Write;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Status {
    None,
    Todo,
    Doing,
    Done,
}

impl fmt::Display for Status {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Status::None => write!(f, "None"),
            Status::Todo => write!(f, "Todo"),
            Status::Doing => write!(f, "Doing"),
            Status::Done => write!(f, "Done"),
        }
    }
}

impl std::str::FromStr for Status {
    type Err = eyre::Report;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "todo" => Ok(Status::Todo),
            "doing" => Ok(Status::Doing),
            "done" => Ok(Status::Done),
            other => Err(eyre::eyre!("invalid status {}", other)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Change {
    pub from: Status,
    pub to: Status,
    pub on: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub status: Status,
    pub changes: Vec<Change>,
    pub priority: Option<u64>,
}

impl Task {
    pub fn detail(&self) -> Result<TaskDetail> {
        let contents =
            std::fs::read_to_string(self.target_path().wrap_err("computing target path")?)
                .wrap_err("reading task detail")?;
        let mut parts = contents.splitn(3, "---");
        let _ = parts.next().unwrap();
        let header: TaskDetailHeader =
            serde_yaml::from_str(parts.next().unwrap()).wrap_err("parsing task detail")?;
        let description = parts.next().unwrap();
        Ok(TaskDetail {
            id: header.id,
            summary: header.summary,
            tags: header.tags,
            description: description.to_string(),
        })
    }

    fn target_path(&self) -> Result<PathBuf> {
        let pm_dir = find_project_root()
            .map(|r| r.join("pm"))
            .wrap_err("computing pm dir")?;
        Ok(pm_dir.join("tasks").join(format!("{:03}.md", self.id)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub meta: Meta,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
struct TaskDetailHeader {
    id: u64,
    summary: String,
    tags: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: u64,
    pub summary: String,
    pub description: String,
    pub tags: Vec<String>,
}

impl TaskDetail {
    fn new(task_id: u64, entry: &[String]) -> TaskDetail {
        let summary_entries = entry
            .iter()
            .filter(|w| !(w.starts_with(':') && w.ends_with(':')))
            .map(|w| w.as_str())
            .collect::<Vec<_>>();
        let summary = summary_entries.join(" ");
        let tags = entry
            .iter()
            .filter_map(|e| {
                if e.starts_with(':') && e.ends_with(':') {
                    Some(e.chars().skip(1).take_while(|c| *c != ':').collect())
                } else {
                    None
                }
            })
            .collect();
        TaskDetail {
            id: task_id,
            summary,
            description: "".to_string(),
            tags,
        }
    }

    fn save(&self) -> Result<()> {
        let path = self.target_path().wrap_err("finding detail path")?;
        let header = self.header();
        let header = serde_yaml::to_string(&header).wrap_err("serializing task detail")?;
        let mut f = std::fs::File::create(path).wrap_err("creating file")?;
        writeln!(&mut f, "{}", header)?;
        writeln!(&mut f, "---")?;
        writeln!(&mut f, "{}", self.description.trim())?;
        Ok(())
    }

    fn header(&self) -> TaskDetailHeader {
        TaskDetailHeader {
            id: self.id,
            summary: self.summary.clone(),
            tags: self.tags.clone(),
        }
    }

    fn target_path(&self) -> Result<PathBuf> {
        let pm_dir = find_project_root()
            .map(|r| r.join("pm"))
            .wrap_err("computing pm dir")?;
        let tasks_dir = pm_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).wrap_err("creating tasks dir")?;
        let filename = format!("{:03}.md", self.id);
        Ok(tasks_dir.join(filename))
    }
}

impl Index {
    pub fn new(name: impl Into<String>) -> Result<Index> {
        Ok(Index {
            meta: Meta { name: name.into() },
            tasks: Vec::new(),
        })
    }

    pub fn save(&self, force: bool) -> Result<()> {
        let path = find_index_path().wrap_err("finding index path")?;
        if path.is_file() && !force {
            return Err(crate::error::PmError::IndexExists.into());
        }
        ensure_parent_dir(&path)
            .wrap_err_with(|| format!("ensuring parent dir for path {:?}", path))?;
        let body = serde_yaml::to_string(self).wrap_err("serializing index")?;
        std::fs::write(path, body).wrap_err("writing index")?;
        Ok(())
    }

    pub fn load() -> Result<Index> {
        let path = find_index_path().wrap_err("finding index path")?;
        let contents = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("reading config file {:?}", &path))?;
        let index: Index = serde_yaml::from_str(&contents).wrap_err("parsing index")?;
        Ok(index)
    }

    pub fn create_task(&mut self, entry: &[String]) -> Result<()> {
        let task = Task {
            id: self.next_id(),
            status: Status::Todo,
            changes: vec![Change {
                from: Status::None,
                to: Status::Todo,
                on: Utc::now(),
            }],
            priority: None,
        };

        let detail = TaskDetail::new(task.id, entry);

        self.tasks.push(task);
        // TODO(srw): handle the case of one file not saving and rolling back
        self.save(true).wrap_err("saving")?;
        detail.save().wrap_err("saving task detail")?;

        Ok(())
    }

    pub fn get_task(&self, task_id: u64) -> Option<&Task> {
        for task in &self.tasks {
            if task.id == task_id {
                return Some(task);
            }
        }

        None
    }

    pub fn move_task(&mut self, task_id: u64, new_status: Status) -> Result<()> {
        let mut found = false;
        for task in self.tasks.iter_mut() {
            if task.id == task_id {
                found = true;

                if task.status == new_status {
                    // do not update
                    break;
                }

                let change = Change {
                    from: task.status,
                    to: new_status,
                    on: Utc::now(),
                };
                task.changes.push(change);
                task.status = new_status;
                break;
            }
        }

        if !found {
            return Err(eyre::eyre!("could not find task {}", task_id));
        }

        self.save(true).wrap_err("saving")?;
        Ok(())
    }

    pub fn delete_task(&mut self, task_id: u64) -> Result<()> {
        let detail_path = self.detail_path(task_id).wrap_err("getting detail path")?;
        std::fs::remove_file(&detail_path)
            .wrap_err_with(|| format!("deleting file {:?}", &detail_path))?;
        if let Some(idx) = self.tasks.iter().position(|t| t.id == task_id) {
            self.tasks.remove(idx);
        }
        self.save(true).wrap_err("saving")?;
        Ok(())
    }

    pub fn detail_path(&self, task_id: u64) -> Result<PathBuf> {
        let pm_dir = find_project_root()
            .map(|r| r.join("pm"))
            .wrap_err("computing pm dir")?;
        Ok(pm_dir.join("tasks").join(format!("{:03}.md", task_id)))
    }

    pub fn sorted_tasks_with_status(&self, status: Status) -> Option<Vec<Task>> {
        let mut tasks: Vec<_> = self
            .tasks
            .iter()
            .cloned()
            .filter(|t| t.status == status)
            .collect();
        if tasks.is_empty() {
            return None;
        }

        tasks.sort_by(|a, b| match (a.priority, b.priority) {
            (Some(pa), Some(pb)) => pa.cmp(&pb),
            (Some(_), None) => std::cmp::Ordering::Greater,
            (None, Some(_)) => std::cmp::Ordering::Less,
            (None, None) => a.id.cmp(&b.id),
        });

        Some(tasks)
    }

    fn next_id(&self) -> u64 {
        self.tasks.iter().map(|t| t.id).max().unwrap_or(0) + 1
    }
}

fn find_index_path() -> Result<PathBuf> {
    let project_root = find_project_root().wrap_err("finding project root")?;
    Ok(project_root.join("pm").join("index.yml"))
}

fn find_project_root() -> Result<PathBuf> {
    let mut cwd = std::env::current_dir().wrap_err("getting current dir")?;
    loop {
        if cwd == Path::new("/") {
            return Err(eyre::eyre!("could not find root path for git repository"));
        }
        if cwd.join(".git").is_dir() {
            return Ok(cwd);
        }
        cwd = cwd.join("..").canonicalize()?;
    }
}

fn ensure_parent_dir(p: &Path) -> Result<()> {
    // unwrap is safe because we construct the final two path components
    let parent_dir = p.parent().unwrap();
    std::fs::create_dir_all(parent_dir)
        .wrap_err_with(|| format!("creating directory {:?}", parent_dir))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_index() {
        let text = r#"
meta:
  name: My first project
tasks:
- id: 1
  status: Doing
  changes:
  - from: Todo
    to: Doing
    on: 2021-01-01T00:00:00+00:00
- id: 2
  status: Done
  changes:
  - from: Todo
    to: Doing
    on: 2021-01-01T00:00:00+00:00
  - from: Doing
    to: Done
    on: 2021-02-01T00:00:00+00:00
"#;

        let parsed: Index = serde_yaml::from_str(text).unwrap();
        assert_eq!(parsed.meta.name, "My first project");
    }

    #[test]
    fn parse_entry_for_task_detail_no_tags() {
        let entry = vec!["A".to_string(), "basic".to_string(), "title".to_string()];
        let task_detail = TaskDetail::new(0, &entry);

        assert_eq!(task_detail.summary, "A basic title".to_string());
        assert_eq!(task_detail.tags, Vec::<String>::new());
    }

    #[test]
    fn parse_entry_for_task_detail_with_tags() {
        let entry = vec![
            "A".to_string(),
            "basic".to_string(),
            ":tag:".to_string(),
            "title".to_string(),
        ];
        let task_detail = TaskDetail::new(0, &entry);

        assert_eq!(task_detail.summary, "A basic title".to_string());
        assert_eq!(task_detail.tags, vec!["tag".to_string()]);
    }

    #[test]
    fn task_sorting_without_priorities() {
        let tasks = vec![
            Task {
                id: 1,
                status: Status::Done,
                changes: vec![],
                priority: None,
            },
            Task {
                id: 2,
                status: Status::Done,
                changes: vec![],
                priority: None,
            },
        ];

        let index = Index {
            meta: Meta {
                name: "Foo".to_string(),
            },
            tasks,
        };
        let retrieved_tasks = index.sorted_tasks_with_status(Status::Done).unwrap();
        let ids: Vec<_> = retrieved_tasks.iter().map(|t| t.id).collect();
        assert_eq!(ids, &[1, 2]);
    }

    #[test]
    fn task_sorting_with_priorities() {
        let tasks = vec![
            Task {
                id: 1,
                status: Status::Done,
                changes: vec![],
                priority: Some(100),
            },
            Task {
                id: 2,
                status: Status::Done,
                changes: vec![],
                priority: None,
            },
        ];

        let index = Index {
            meta: Meta {
                name: "Foo".to_string(),
            },
            tasks,
        };
        let retrieved_tasks = index.sorted_tasks_with_status(Status::Done).unwrap();
        let ids: Vec<_> = retrieved_tasks.iter().map(|t| t.id).collect();
        assert_eq!(ids, &[2, 1]);
    }
}
