use chrono::{DateTime, Utc};
use eyre::{Result, WrapErr};
use serde::Serialize;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize)]
pub struct Meta {
    pub name: String,
}

#[derive(Debug, Serialize)]
pub enum Status {
    Todo,
    Doing,
    Done,
}

#[derive(Debug, Serialize)]
pub struct Change {
    pub from: Status,
    pub to: Status,
    pub on: DateTime<Utc>,
}

#[derive(Debug, Serialize)]
pub struct Task {
    pub id: u64,
    pub status: Status,
    pub changes: Vec<Change>,
}

#[derive(Debug, Serialize)]
pub struct Index {
    pub meta: Meta,
    pub tasks: Vec<Task>,
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
        todo!()
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
