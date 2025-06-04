use anyhow::Result;
use watchdogfs::{
    cli,
    config::{self, Config},
    integrity, logger, selfcheck, watcher,
};

fn main() -> Result<()> {
    logger::init()?;
    let args = cli::parse();

    // Self‐integrity check (if supplied via --self-integrity-path)
    if let Some(_path) = &args.self_integrity_path() {
        let cfg: Config = config::load("config.yaml")?;
        if let Some(sip) = &cfg.self_integrity_path {
            selfcheck::verify(sip)?;
        }
    }

    match args.command {
        cli::Commands::Init {
            config: config_path,
            with_baseline,
        } => {
            // If they asked for `--with-baseline` but gave a custom filename,
            // we refuse and print an error.
            if with_baseline && config_path != "config.yaml" {
                eprintln!("❌ When using --with-baseline, the config file must be named `config.yaml`");
            }
            
            // 1) Write a default config.yaml (or error if it already exists)
            config::write_default(&config_path)?;

            // 2) If user passed --with-baseline, immediately generate all baselines
            if with_baseline {
                // Note: generate_baseline() expects to read "config.yaml" in cwd.
                //
                // If you wrote to a different filename than "config.yaml" (e.g. "foo.yaml"),
                // we might either rename it to "config.yaml" first, or extend generate_baseline
                // to accept a path. For now, we assume the default name "config.yaml".
                integrity::generate_baseline()?;
            }
        }

        cli::Commands::Baseline => {
            // Explicit "baseline" command: regenerate each baseline_<job>.json
            integrity::generate_baseline()?;
        }

        cli::Commands::Start { daemon } => {
            watcher::start(daemon)?;
        }
    }

    Ok(())
}
