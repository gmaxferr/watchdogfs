// Linux inotify implementation
use notify::{RecommendedWatcher, RecursiveMode, Watcher, Config, Result};
use std::path::Path;

pub fn watch<P: AsRef<Path>>(path: P) -> Result<()> {
    let mut watcher = RecommendedWatcher::new(
        |res| match res {
            Ok(event) => println!("File changed: {:?}", event),
            Err(e) => eprintln!("watch error: {:?}", e),
        },
        Config::default(), // Added missing argument
    )?;

    watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    Ok(())
}
