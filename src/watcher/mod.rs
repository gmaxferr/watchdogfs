//! File monitoring abstraction (multi‐job / named profiles).
use crate::alerts::dispatch;
use crate::config::{Config, JobConfig};
use crate::integrity::{Baseline, calculate_checksum, generate_map};
use anyhow::Result;
use notify::{
    Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
    Watcher,
};
use serde_json;
use serde_yaml;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    thread,
    thread::park,
    time::{Duration, Instant},
};

/// Start all named jobs. If `daemon == true`, blocks indefinitely.
pub fn start(daemon: bool) -> Result<()> {
    // 1) Load config.yaml (again, inside watcher)
    let cfg_str = fs::read_to_string("config.yaml")?;
    let cfg: Config = serde_yaml::from_str(&cfg_str)?;

    // 2) For each job, load or generate its baseline, then spin up its watcher
    let mut handles = Vec::new();
    for (job_name, job_cfg) in cfg.jobs.clone() {
        // Determine baseline file for this job
        let baseline_filename = format!("baseline_{}.json", job_name);
        let baseline: Baseline = if Path::new(&baseline_filename).exists() {
            // Load existing baseline_<job_name>.json
            let s = fs::read_to_string(&baseline_filename)?;
            serde_json::from_str(&s)?
        } else {
            // Generate new baseline and write it
            let bmap = generate_map(&job_cfg.watch_paths)?;
            let json = serde_json::to_string_pretty(&bmap)?;
            fs::write(&baseline_filename, json)?;
            bmap
        };

        // Clone for thread
        let job_cfg_clone = job_cfg.clone();
        let job_name_clone = job_name.clone();
        let mut baseline_clone = baseline.clone();

        // Spawn a thread per job
        let handle = thread::spawn(move || {
            if job_cfg_clone.watcher.mode.as_str() == "inotify" {
                if let Err(e) = start_inotify_job(
                    job_name_clone.clone(),
                    job_cfg_clone.clone(),
                    &mut baseline_clone,
                ) {
                    eprintln!("Job '{}' inotify start error: {:?}", job_name_clone, e);
                }
            } else if job_cfg_clone.watcher.mode.as_str() == "poll" {
                if let Err(e) = start_polling_job(
                    job_name_clone.clone(),
                    job_cfg_clone.clone(),
                    &mut baseline_clone,
                ) {
                    eprintln!("Job '{}' polling start error: {:?}", job_name_clone, e);
                }
            } else {
                eprintln!(
                    "Job '{}': unknown watcher mode '{}'",
                    job_name_clone, job_cfg_clone.watcher.mode
                );
            }
        });
        handles.push(handle);
    }

    // 3) If daemon, park the main thread so watchers stay alive
    if daemon {
        park();
    }

    // If not daemon, return and let threads run briefly (they’ll die once main exits,
    // so in practice you must run with --daemon to keep them alive).
    Ok(())
}

/// Starts a single job’s inotify watcher (does NOT park here).
fn start_inotify_job(_job_name: String, job_cfg: JobConfig, baseline: &mut Baseline) -> Result<()> {
    let debounce = Duration::from_millis(job_cfg.watcher.debounce_ms.unwrap_or(500));
    let watch_paths = job_cfg.watch_paths.clone();

    // `last_seen` tracks the last time we triggered on a given path to handle debounce
    let mut last_seen: HashMap<String, Instant> = HashMap::new();

    // Build the inotify watcher with a callback capturing job_cfg and baseline
    let callback_cfg = job_cfg.clone();
    let mut callback_baseline = baseline.clone();
    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            on_event_inotify(
                res,
                &callback_cfg,
                &mut callback_baseline,
                &mut last_seen,
                debounce,
            )
        },
        NotifyConfig::default(),
    )?;

    // Register each path for this job
    for path in &watch_paths {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    }

    Ok(())
}

/// Inotify event handler for one job
fn on_event_inotify(
    res: NotifyResult<Event>,
    cfg: &JobConfig,
    baseline: &mut Baseline,
    last_seen: &mut HashMap<String, Instant>,
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
                continue; // skip duplicates within debounce window
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

/// Starts a single job’s polling watcher (does NOT park here).
pub fn start_polling_job(
    job_name: String,
    job_cfg: JobConfig,
    baseline: &mut Baseline,
) -> Result<()> {
    let interval = Duration::from_secs(job_cfg.watcher.poll_interval.unwrap_or(5));
    let debounce = Duration::from_millis(job_cfg.watcher.debounce_ms.unwrap_or(500));

    for watch_path in &job_cfg.watch_paths {
        let path = PathBuf::from(watch_path);
        let cfg_clone = job_cfg.clone();
        let mut baseline_clone = baseline.clone();
        let path_clone = path.clone();
        // CLONE job_name FOR THIS ITERATION:
        let job_name_clone = job_name.clone();

        thread::spawn(move || {
            let mut last_seen_time = Instant::now() - debounce;
            loop {
                thread::sleep(interval);
                let now = Instant::now();
                if now.duration_since(last_seen_time) < debounce {
                    continue;
                }

                let path_str = path_clone.to_string_lossy();
                match calculate_checksum(&path_str) {
                    Ok(new_sum) => {
                        if let Some(old_sum) = baseline_clone.get(&*path_str).cloned() {
                            if old_sum != new_sum {
                                dispatch(
                                    &cfg_clone.alerts,
                                    &path_clone,
                                    old_sum.clone(),
                                    new_sum.clone(),
                                );
                                baseline_clone.insert(path_str.to_string(), new_sum.clone());
                                last_seen_time = now;
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!(
                            "Polling job '{}' failed checksum on {}: {:?}",
                            job_name_clone,
                            path_clone.display(),
                            e
                        );
                    }
                }
            }
        });
    }

    Ok(())
}
