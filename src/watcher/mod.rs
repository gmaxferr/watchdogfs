use crate::alerts::dispatch;
use crate::config::{Config, JobConfig};
use crate::integrity::{Baseline, calculate_checksum, generate_map};
use anyhow::{Context, Result};
use notify::{
    Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Result as NotifyResult,
    Watcher,
};
use serde_yaml;
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    sync::mpsc::{self, Receiver, Sender},
    thread,
    time::{Duration, Instant},
};

/// Start all named jobs. If `daemon == true`, also run a loop that watches `config.yaml`
/// for changes and dynamically adds/removes/reloads job threads at runtime.
///
/// This function returns immediately if `daemon == false`. If `daemon == true`, this
/// function blocks forever, restarting any job whose configuration struct changes,
/// and adding/removing jobs as needed.
pub fn start(daemon: bool) -> Result<()> {
    let config_path = "config.yaml";

    // Keep track of each running job: job_name -> (stop_sender, thread_handle)
    let mut job_handles: HashMap<String, (Sender<()>, thread::JoinHandle<()>)> = HashMap::new();

    // 1) Read the initial config, spawn one thread per job
    let mut last_modified = fs::metadata(config_path)?
        .modified()
        .context("getting initial config.yaml modified time")?;
    let mut current_cfg = load_config(config_path)?;

    for (job_name, job_cfg) in current_cfg.jobs.clone() {
        let baseline_map = load_or_generate_baseline(&job_name, &job_cfg)?;
        let (tx, rx) = mpsc::channel();
        let job_handle = spawn_job_thread(job_name.clone(), job_cfg.clone(), baseline_map, rx);
        job_handles.insert(job_name, (tx, job_handle));
    }

    // 2) If not a daemon, return now (threads will end when main exits)
    if !daemon {
        return Ok(());
    }

    // 3) If daemon, loop forever: check config.yaml every 2 seconds
    loop {
        thread::sleep(Duration::from_secs(2));

        // Check if config.yaml has been updated on disk
        let meta = match fs::metadata(config_path) {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to stat {}: {:?}", config_path, e);
                continue;
            }
        };
        let modified = match meta.modified() {
            Ok(m) => m,
            Err(e) => {
                eprintln!("Failed to get modified time for {}: {:?}", config_path, e);
                continue;
            }
        };

        // If it hasn't changed, do nothing
        if modified <= last_modified {
            continue;
        }
        last_modified = modified;

        // Reload the YAML
        let new_cfg = match load_config(config_path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to reload {}: {:?}", config_path, e);
                continue;
            }
        };

        // ===== 1) Remove jobs that no longer exist in new_cfg =====
        for existing_job in job_handles.keys().cloned().collect::<Vec<_>>() {
            if !new_cfg.jobs.contains_key(&existing_job) {
                if let Some((stop_tx, handle)) = job_handles.remove(&existing_job) {
                    let _ = stop_tx.send(());
                    let _ = handle.join();
                    println!("Stopped job '{}'", existing_job);
                }
            }
        }

        // ===== 2) Reload any job whose JobConfig struct changed (treat as remove+add) =====
        for job_name in current_cfg.jobs.keys() {
            if let (Some(old_cfg), Some(new_job_cfg)) =
                (current_cfg.jobs.get(job_name), new_cfg.jobs.get(job_name))
            {
                if old_cfg != new_job_cfg {
                    // Stop the old thread
                    if let Some((stop_tx, handle)) = job_handles.remove(job_name) {
                        let _ = stop_tx.send(());
                        let _ = handle.join();
                        println!("Reloaded job '{}' due to config change", job_name);
                    }
                    // Spawn a new thread with updated config
                    match load_or_generate_baseline(job_name, new_job_cfg) {
                        Ok(baseline_map) => {
                            let (tx, rx) = mpsc::channel();
                            let handle = spawn_job_thread(
                                job_name.clone(),
                                new_job_cfg.clone(),
                                baseline_map,
                                rx,
                            );
                            job_handles.insert(job_name.clone(), (tx, handle));
                        }
                        Err(e) => {
                            eprintln!(
                                "Failed to regenerate baseline for changed job '{}': {:?}",
                                job_name, e
                            );
                        }
                    }
                }
            }
        }

        // ===== 3) Add any new jobs in new_cfg =====
        for (job_name, job_cfg) in new_cfg.jobs.clone() {
            if !job_handles.contains_key(&job_name) {
                match load_or_generate_baseline(&job_name, &job_cfg) {
                    Ok(baseline_map) => {
                        let (tx, rx) = mpsc::channel();
                        let handle =
                            spawn_job_thread(job_name.clone(), job_cfg.clone(), baseline_map, rx);
                        job_handles.insert(job_name.clone(), (tx, handle));
                        println!("Started job '{}'", job_name);
                    }
                    Err(e) => {
                        eprintln!(
                            "Failed to generate baseline for new job '{}': {:?}",
                            job_name, e
                        );
                    }
                }
            }
        }

        // Update our snapshot
        current_cfg = new_cfg;
    }
}

/// Load + parse `config.yaml`.
fn load_config(path: &str) -> Result<Config> {
    let s = fs::read_to_string(path).with_context(|| format!("reading config file {:?}", path))?;
    let cfg: Config = serde_yaml::from_str(&s).context("parsing YAML config")?;
    Ok(cfg)
}

/// For a given job, either load its existing `baseline_<job_name>.json` or generate a fresh one.
fn load_or_generate_baseline(job_name: &str, job_cfg: &JobConfig) -> Result<Baseline> {
    let filename = format!("baseline_{}.json", job_name);
    if Path::new(&filename).exists() {
        // Load existing JSON
        let s = fs::read_to_string(&filename)?;
        let baseline: Baseline =
            serde_json::from_str(&s).with_context(|| format!("parsing {}", filename))?;
        Ok(baseline)
    } else {
        // Generate new baseline JSON from scratch
        let baseline_map = generate_map(&job_cfg.watch_paths)
            .with_context(|| format!("generating baseline for job '{}'", job_name))?;
        let json = serde_json::to_string_pretty(&baseline_map)
            .with_context(|| format!("serializing baseline for job '{}'", job_name))?;
        fs::write(&filename, &json).with_context(|| format!("writing {}", filename))?;
        Ok(baseline_map)
    }
}

/// Spawn a new thread to run exactly one job (either inotify‐based or polling‐based).
/// Returns the thread’s `JoinHandle<()>`. The `stop_rx` will be closed or receive a value
/// to signal this thread to stop.
fn spawn_job_thread(
    job_name: String,
    job_cfg: JobConfig,
    mut baseline_map: Baseline,
    stop_rx: Receiver<()>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        if job_cfg.watcher.mode.as_str() == "inotify" {
            if let Err(e) = run_inotify_job(
                job_name.clone(),
                job_cfg.clone(),
                &mut baseline_map,
                stop_rx,
            ) {
                eprintln!("Job '{}' inotify error: {:?}", job_name, e);
            }
        } else if job_cfg.watcher.mode.as_str() == "poll" {
            if let Err(e) = run_polling_job(
                job_name.clone(),
                job_cfg.clone(),
                &mut baseline_map,
                stop_rx,
            ) {
                eprintln!("Job '{}' polling error: {:?}", job_name, e);
            }
        } else {
            eprintln!(
                "Job '{}': unknown watcher mode '{}'",
                job_name, job_cfg.watcher.mode
            );
        }
    })
}

/// Run a single job in “inotify” mode. The thread registers all paths via notify,
/// then blocks on `stop_rx.recv()`. When `stop_rx` yields a value (or is closed), we drop
/// the watcher and return, allowing the thread to exit.
fn run_inotify_job(
    _job_name: String,
    job_cfg: JobConfig,
    baseline: &mut Baseline,
    stop_rx: Receiver<()>,
) -> Result<()> {
    let debounce = Duration::from_millis(job_cfg.watcher.debounce_ms.unwrap_or(500));
    let watch_paths = job_cfg.watch_paths.clone();

    // `last_seen` tracks debouncing per-file
    let mut last_seen: HashMap<String, Instant> = HashMap::new();
    let cfg_for_cb = job_cfg.clone();
    let mut baseline_for_cb = baseline.clone();

    let mut watcher: RecommendedWatcher = RecommendedWatcher::new(
        move |res: NotifyResult<Event>| {
            on_event_inotify(
                res,
                &cfg_for_cb,
                &mut baseline_for_cb,
                &mut last_seen,
                debounce,
            )
        },
        NotifyConfig::default(),
    )?;

    for path in &watch_paths {
        watcher.watch(path.as_ref(), RecursiveMode::Recursive)?;
    }

    // BLOCK here until we receive a stop signal
    let _ = stop_rx.recv();
    Ok(())
}

/// Event handler for inotify-based jobs. Calculates a new checksum, compares with the baseline,
/// and fires `dispatch(...)` if it changed. Uses `last_seen` + `debounce` to avoid duplicates.
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

            // Debounce: skip if we've seen it recently
            let prev = last_seen
                .get(&path_str)
                .cloned()
                .unwrap_or_else(|| now - debounce * 2);
            if now.duration_since(prev) < debounce {
                continue;
            }
            last_seen.insert(path_str.clone(), now);

            // Compute checksum and compare to baseline
            if let Ok(new_sum) = calculate_checksum(&path_str) {
                if let Some(old_sum) = baseline.insert(path_str.clone(), new_sum.clone()) {
                    if old_sum != new_sum {
                        dispatch(&cfg.alerts, &path_buf, old_sum, new_sum.clone());
                    }
                }
            }
        }
    }
}

/// Run a single job in “polling” mode. We keep all watched paths in one loop,
/// sleeping for `poll_interval` between iterations. We also implement debouncing
/// per-path (using a `last_seen` map). When `stop_rx` yields (or the channel is closed),
/// we break and the thread exits.
fn run_polling_job(
    job_name: String,
    job_cfg: JobConfig,
    baseline: &mut Baseline,
    stop_rx: Receiver<()>,
) -> Result<()> {
    let interval = Duration::from_secs(job_cfg.watcher.poll_interval.unwrap_or(5));
    let debounce = Duration::from_millis(job_cfg.watcher.debounce_ms.unwrap_or(500));

    // Track last time we fired for each path, to handle per-path debounce.
    let mut last_seen: HashMap<String, Instant> = HashMap::new();
    // For convenience, clone the list of paths here
    let watch_paths = job_cfg.watch_paths.clone();

    loop {
        // 1) Check for stop signal
        if stop_rx.try_recv().is_ok() {
            break;
        }

        // 2) Sleep for the polling interval
        thread::sleep(interval);

        // 3) For each watched path, see if it changed
        for path_str in &watch_paths {
            let now = Instant::now();
            // Debounce: skip if seen too recently
            let prev = last_seen
                .get(path_str)
                .cloned()
                .unwrap_or_else(|| now - debounce * 2);
            if now.duration_since(prev) < debounce {
                continue;
            }

            // Attempt to calculate a new checksum
            match calculate_checksum(path_str) {
                Ok(new_sum) => {
                    if let Some(old_sum) = baseline.get(path_str).cloned() {
                        if old_sum != new_sum {
                            // Fire the alert and update baseline + last_seen
                            let path_buf = PathBuf::from(path_str);
                            dispatch(&job_cfg.alerts, &path_buf, old_sum.clone(), new_sum.clone());
                            baseline.insert(path_str.clone(), new_sum.clone());
                            last_seen.insert(path_str.clone(), now);
                        }
                    }
                }
                Err(e) => {
                    eprintln!(
                        "Polling job '{}' failed checksum on {}: {:?}",
                        job_name, path_str, e
                    );
                }
            }
        }
    }

    Ok(())
}
