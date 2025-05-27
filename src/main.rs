use anyhow::Result;
use watchdogfs::{cli, integrity, logger, watcher};

fn main() -> Result<()> {
    logger::init()?;
    let args = cli::parse();

    match args.command {
        cli::Commands::Init { files } => integrity::init(files)?,
        cli::Commands::Baseline => {
            integrity::generate_baseline()?;
        }
        cli::Commands::Start { daemon } => watcher::start(daemon)?,
    }

    Ok(())
}
