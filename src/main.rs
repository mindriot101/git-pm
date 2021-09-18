use eyre::{Result, WrapErr};
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
    }

    Ok(())
}
