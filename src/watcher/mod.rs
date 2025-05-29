//! File monitoring abstraction.

use crate::alerts::dispatch;
use crate::config::Config;
use crate::integrity::{Baseline, calculate_checksum, generate_map};
use anyhow::{Result, bail};
use notify::Watcher;
use notify::{
    Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
};
use serde_yaml;
use std::{
    fs,
    path::PathBuf,
    thread::{self, park},
    time::{Duration, Instant},
};

/// Start the watcher. If `daemon == true`, blocks indefinitely.
pub fn start(daemon: bool) -> Result<()> {
    // 1. Load config.yaml
    let cfg_str = fs::read_to_string("config.yaml")?;
    let cfg: Config = serde_yaml::from_str(&cfg_str)?;

    // 2. Load baseline into a mutable map
    let baseline: Baseline = generate_map(&cfg.watch_paths)?;

    // 3. Dispatch to the chosen backend (pass ownership into each backend)
    match cfg.watcher.mode.as_str() {
        "inotify" => start_inotify(cfg.clone(), baseline, daemon)?,
        "poll" => start_polling(cfg.clone(), baseline, daemon)?,
        other => bail!("Unknown watcher mode: {}", other),
    }

    Ok(())
}

pub fn start_inotify(cfg: Config, baseline: Baseline, daemon: bool) -> Result<()> {
    // 1) Pre-calc debounce and copy the watch list
    let debounce = Duration::from_millis(cfg.watcher.debounce_ms.unwrap_or(500));
    let watch_paths = cfg.watch_paths.clone(); // so cfg can stay alive

    // 2) Clone only what the callback needs
    let callback_cfg = cfg.clone();
    let mut callback_baseline = baseline.clone();
    let mut last_seen = std::collections::HashMap::<String, Instant>::new();

    // 3) Build the watcher, capturing _only_ the clones above
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            on_event(
                res,
                &callback_cfg,
                &mut callback_baseline,
                &mut last_seen,
                debounce,
            )
        },
        NotifyConfig::default(),
    )?;

    // 4) Register each path (using the original cfg)
    for path in &watch_paths {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    }

    // 5) If daemon, block forever
    if daemon {
        park();
    }
    Ok(())
}

fn start_polling(cfg: Config, baseline: Baseline, daemon: bool) -> Result<()> {
    let interval = Duration::from_secs(cfg.watcher.poll_interval.unwrap_or(5));
    let debounce = Duration::from_millis(cfg.watcher.debounce_ms.unwrap_or(500));

    for watch_path in &cfg.watch_paths {
        let path = PathBuf::from(watch_path);
        let cfg_clone = cfg.clone();
        let mut baseline_clone = baseline.clone();

        thread::spawn(move || {
            let mut last_seen = Instant::now() - debounce;
            loop {
                thread::sleep(interval);
                if let Err(e) = on_poll(
                    &path,
                    &cfg_clone,
                    &mut baseline_clone,
                    &mut last_seen,
                    debounce,
                ) {
                    eprintln!("poll error: {:?}", e);
                }
            }
        });
    }

    if daemon {
        park();
    }
    Ok(())
}

fn on_event(
    res: NotifyResult<Event>,
    cfg: &Config,
    baseline: &mut Baseline,
    last_seen: &mut std::collections::HashMap<String, Instant>,
    debounce: Duration,
) {
    if let Ok(event) = res {
        for path_buf in event.paths {
            let path_str = path_buf.to_string_lossy().into_owned();
            let now = Instant::now();
            let prev = last_seen
                .get(&path_str)
                .cloned()
                .unwrap_or_else(|| now - debounce * 2);
            if now.duration_since(prev) < debounce {
                continue; // skip rapid duplicates
            }
            last_seen.insert(path_str.clone(), now);

            if let Ok(new_sum) = calculate_checksum(&path_str) {
                // Insert returns the old checksum if present
                let old_sum_opt = baseline.insert(path_str.clone(), new_sum.clone());
                if let Some(old_sum) = old_sum_opt {
                    if old_sum != new_sum {
                        dispatch(&cfg.alerts, &path_buf, old_sum, new_sum.clone());
                    }
                }
            }
        }
    }
}

fn on_poll(
    path: &PathBuf,
    cfg: &Config,
    baseline: &mut Baseline,
    last_seen: &mut Instant,
    debounce: Duration,
) -> Result<()> {
    let now = Instant::now();
    if now.duration_since(*last_seen) < debounce {
        return Ok(()); // still in debounce window
    }

    let path_str = path.to_string_lossy();
    let new_sum = calculate_checksum(&path_str)?;
    let old_sum_opt = baseline.get(&*path_str).cloned();
    if let Some(old_sum) = old_sum_opt {
        if old_sum != new_sum {
            dispatch(&cfg.alerts, path, old_sum, new_sum.clone());
            baseline.insert(path_str.to_string(), new_sum.clone());
            *last_seen = now;
        }
    }
    Ok(())
}
