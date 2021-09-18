use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::{Deserialize, Serialize};
use std::fmt;
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Change {
    pub from: Status,
    pub to: Status,
    pub on: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Task {
    pub id: u64,
    pub status: Status,
    pub changes: Vec<Change>,
}

impl Task {
    pub fn detail(&self) -> Result<TaskDetail> {
        let contents =
            std::fs::read_to_string(self.target_path().wrap_err("computing target path")?)
                .wrap_err("reading task detail")?;
        let detail: TaskDetail = serde_yaml::from_str(&contents).wrap_err("parsing task detail")?;
        Ok(detail)
    }

    fn target_path(&self) -> Result<PathBuf> {
        let pm_dir = find_project_root()
            .map(|r| r.join("pm"))
            .wrap_err("computing pm dir")?;
        Ok(pm_dir.join("tasks").join(format!("{:03}.yml", self.id)))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Index {
    pub meta: Meta,
    pub tasks: Vec<Task>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskDetail {
    pub id: u64,
    pub summary: String,
    pub description: Option<String>,
}

impl TaskDetail {
    fn save(&self) -> Result<()> {
        let path = self.target_path().wrap_err("finding detail path")?;
        let body = serde_yaml::to_string(self).wrap_err("serializing task detail")?;
        std::fs::write(path, body).wrap_err("saving task detail")?;
        Ok(())
    }

    fn target_path(&self) -> Result<PathBuf> {
        let pm_dir = find_project_root()
            .map(|r| r.join("pm"))
            .wrap_err("computing pm dir")?;
        let tasks_dir = pm_dir.join("tasks");
        std::fs::create_dir_all(&tasks_dir).wrap_err("creating tasks dir")?;
        let filename = format!("{:03}.yml", self.id);
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

    pub fn save(&self) -> Result<()> {
        let path = find_index_path().wrap_err("finding index path")?;
        ensure_parent_dir(&path)
            .wrap_err_with(|| format!("ensuring parent dir for path {:?}", path))?;
        let body = serde_yaml::to_string(self).wrap_err("serializing index")?;
        std::fs::write(path, body).wrap_err("saving index")?;
        Ok(())
    }

    pub fn load() -> Result<Index> {
        let path = find_index_path().wrap_err("finding index path")?;
        let contents = std::fs::read_to_string(&path)
            .wrap_err_with(|| format!("reading config file {:?}", &path))?;
        let index: Index = serde_yaml::from_str(&contents).wrap_err("parsing index")?;
        Ok(index)
    }

    pub fn create_task(&mut self, entry: &str) -> Result<()> {
        let task = Task {
            id: self.next_id(),
            status: Status::Todo,
            changes: vec![Change {
                from: Status::None,
                to: Status::Todo,
                on: Utc::now(),
            }],
        };
        let detail = TaskDetail {
            id: task.id,
            summary: entry.into(),
            description: None,
        };
        self.tasks.push(task);
        self.save().wrap_err("saving")?;
        detail.save().wrap_err("saving task detail")?;

        Ok(())
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
}
