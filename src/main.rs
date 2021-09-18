use eyre::{Result, WrapErr};
use structopt::StructOpt;

mod index;

#[derive(StructOpt)]
enum Opts {
    Init {
        #[structopt(short, long)]
        name: String,
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
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();
    let args = Opts::from_args();

    let manager = Manager {};

    match args {
        Opts::Init { name } => manager.init(name).wrap_err("init")?,
    }

    Ok(())
}
