use eyre::{Result, WrapErr};
use std::collections::HashMap;
use structopt::StructOpt;

mod index;

#[derive(StructOpt)]
enum Opts {
    Init {
        #[structopt(short, long)]
        name: String,
    },
    Add {
        entry: Vec<String>,
    },
    Show,
    Move {
        task_id: u64,
        status: index::Status,
    },
    Delete {
        task_id: u64,
    },
}

struct Manager {}

impl Manager {
    fn init(&self, name: String) -> Result<()> {
        log::info!("init");

        let index = index::Index::new(name).wrap_err("loading configuration")?;
        index.save().wrap_err("saving index")?;
        Ok(())
    }

    fn add(&self, entry: Vec<String>) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        let entry_text = entry.join(" ");
        index.create_task(&entry_text).wrap_err("creating task")?;
        self.show().wrap_err("showing")?;
        Ok(())
    }

    fn show(&self) -> Result<()> {
        let index = index::Index::load().wrap_err("loading index")?;
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
                        println!("{:03}: {}", task.id, detail.summary);
                    }
                }
            }
            println!();
        }

        Ok(())
    }

    fn move_task(&self, task_id: u64, status: index::Status) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        index.move_task(task_id, status).wrap_err("moving task")?;
        self.show().wrap_err("showing")?;
        Ok(())
    }

    fn delete_task(&self, task_id: u64) -> Result<()> {
        let mut index = index::Index::load().wrap_err("loading index")?;
        index
            .delete_task(task_id)
            .wrap_err("deleting task from index")?;
        self.show().wrap_err("showing")?;
        Ok(())
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let args = Opts::from_args();

    let manager = Manager {};

    match args {
        Opts::Init { name } => manager.init(name).wrap_err("init")?,
        Opts::Add { entry } => manager.add(entry).wrap_err("add")?,
        Opts::Show => manager.show().wrap_err("show")?,
        Opts::Move { task_id, status } => manager.move_task(task_id, status).wrap_err("move")?,
        Opts::Delete { task_id } => manager.delete_task(task_id).wrap_err("deleting")?,
    }

    Ok(())
}
