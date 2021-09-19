use eyre::{Result, WrapErr};
use std::collections::HashMap;
use std::process;
use structopt::StructOpt;

mod error;
mod index;

#[derive(StructOpt)]
enum Opts {
    Init {
        #[structopt(short, long)]
        name: String,
        #[structopt(short, long)]
        force: bool,
    },
    Add {
        entry: Vec<String>,
    },
    Show {
        task_id: Option<u64>,
    },
    Move {
        task_id: u64,
        status: index::Status,
    },
    Delete {
        task_id: u64,
    },
    Edit {
        task_id: u64,
    },
    Start {
        task_id: u64,
    },
    Finish {
        task_id: u64,
    },
}

struct Manager {}

impl Manager {
    fn init(&self, name: String, force: bool) -> Result<()> {
        let index = index::Index::new(name).wrap_err("loading configuration")?;
        match index.save(force) {
            Ok(_) => {}
            Err(e) => {
                if e.is::<crate::error::PmError>() {
                    match e.downcast::<crate::error::PmError>() {
                        Ok(crate::error::PmError::IndexExists) => {
                            eprintln!("index already exists, not overwriting");
                            std::process::exit(1);
                        }
                        Err(e) => return Err(e.into()),
                    }
                }
            }
        }
        Ok(())
    }

    fn add(&self, entry: Vec<String>) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        index.create_task(&entry).wrap_err("creating task")?;
        self.show(None).wrap_err("showing")?;
        Ok(())
    }

    fn show(&self, task_id: Option<u64>) -> Result<()> {
        let index = index::Index::load().wrap_err("loading index")?;
        if let Some(id) = task_id {
            let task = index.get_task(id).expect("could not find task in index");
            let detail = task.detail().wrap_err("fetching task detail")?;

            let summary = detail.summary.trim();
            println!("{}", summary);
            // print a heading line
            for _ in 0..summary.len() {
                print!("-");
            }
            println!();
            // TODO: nice formatting and colours
            println!("{}", detail.description.trim());
        } else {
            let mut store: HashMap<index::Status, Vec<&index::Task>> = HashMap::new();

            for task in &index.tasks {
                let e = store.entry(task.status).or_insert(Vec::new());
                e.push(task);
            }

            let to_print_statuses = &[
                index::Status::Todo,
                index::Status::Doing,
                index::Status::Done,
            ];

            for status in to_print_statuses {
                println!("----------");
                println!("{}", status);

                match store.get_mut(status) {
                    None => println!("... no tasks found"),
                    Some(ts) => {
                        ts.sort_by_key(|task| task.id);
                        for task in ts {
                            let detail = task.detail().wrap_err_with(|| {
                                format!("reading task detail for task {}", task.id)
                            })?;
                            if !detail.tags.is_empty() {
                                let tags_entry = {
                                    let tags =
                                        detail.tags.iter().map(|t| t.as_str()).collect::<Vec<_>>();
                                    tags.join(" ")
                                };
                                println!("{:03}: {}\t\t:{}:", task.id, detail.summary, tags_entry);
                            } else {
                                println!("{:03}: {}", task.id, detail.summary);
                            }
                        }
                    }
                }
                println!();
            }
        }

        Ok(())
    }

    fn move_task(&self, task_id: u64, status: index::Status) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        index.move_task(task_id, status).wrap_err("moving task")?;
        self.show(None).wrap_err("showing")?;
        Ok(())
    }

    fn delete_task(&self, task_id: u64) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        index
            .delete_task(task_id)
            .wrap_err("deleting task from index")?;
        self.show(None).wrap_err("showing")?;
        Ok(())
    }

    fn edit_task(&self, task_id: u64) -> Result<()> {
        let index = index::Index::load().wrap_err("loading index")?;
        let detail_path = index
            .detail_path(task_id)
            .wrap_err("fetching detail path")?;

        let editor = std::env::var("EDITOR").unwrap_or("vim".to_string());
        let mut child = process::Command::new(editor)
            .args(&[detail_path])
            .spawn()
            .wrap_err("spawning editor")?;
        let status = child.wait().wrap_err("getting command exit status")?;
        if !status.success() {
            return Err(eyre::eyre!(
                "editor command exited with status {}",
                status.code().expect("fetching error code")
            ));
        }
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let args = Opts::from_args();

    let manager = Manager {};

    match args {
        Opts::Init { name, force } => manager.init(name, force).wrap_err("init")?,
        Opts::Add { entry } => manager.add(entry).wrap_err("add")?,
        Opts::Show { task_id } => manager.show(task_id).wrap_err("show")?,
        Opts::Move { task_id, status } => manager.move_task(task_id, status).wrap_err("move")?,
        Opts::Delete { task_id } => manager.delete_task(task_id).wrap_err("deleting")?,
        Opts::Edit { task_id } => manager.edit_task(task_id).wrap_err("editing")?,
        Opts::Start { task_id } => manager
            .move_task(task_id, index::Status::Doing)
            .wrap_err("starting task")?,
        Opts::Finish { task_id } => manager
            .move_task(task_id, index::Status::Done)
            .wrap_err("finishing task")?,
    }

    Ok(())
}
