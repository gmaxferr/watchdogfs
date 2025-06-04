# WatchdogFS  
*Filesystem Integrity Monitoring for Linux & Embedded*

[![CI](https://github.com/gmaxferr/watchdogfs/actions/workflows/ci.yml/badge.svg)](https://github.com/gmaxferr/watchdogfs/actions/workflows/ci.yml)
---

## 📋 Description

**WatchdogFS** is a lightweight, high-performance CLI tool written in Rust (latest stable) that continuously monitors specified files or directories for unauthorized changes. It computes and compares SHA-256 checksums against a stored baseline and emits alerts via syslog, HTTP webhooks, customizable local scripts, or dynamically loaded plugins. Designed for both server and embedded (BusyBox/Alpine) environments, WatchdogFS is memory-safe, cross-compile-friendly, and delivers real-time protection with minimal dependencies. The “daemon” mode now supports dynamic configuration reload and multiple named watch-sets (“jobs”) within a single `config.yaml`.

---

## ⭐ Features

### ✅ Already Implemented

- **Named Profiles / Multiple “Jobs”**  
  Instead of a single list of `watch_paths`, you can now define multiple named “jobs” in `config.yaml`. Each job has its own `watch_paths`, `ignore_patterns`, `alerts`, and `watcher` settings.  
  ```yaml
  jobs:
    web_config:
      watch_paths:
        - /etc/nginx/nginx.conf
        - /etc/nginx/sites-enabled/
      ignore_patterns: []  # (reserved)
      alerts:
        webhook_url: "https://example.com/webhook"
        script_path: "/usr/local/bin/nginx-alert.sh"
        use_syslog: true
        plugin_path: "/usr/lib/watchdogfs/nginx_plugin.so"
        payload_template: |
          {
            "job": "{{job_name}}",
            "path": "{{path}}",
            "old": "{{old}}",
            "new": "{{new}}"
          }
      watcher:
        mode: inotify
        poll_interval: 5
        debounce_ms: 500

    firmware_files:
      watch_paths:
        - /opt/firmware.bin
      ignore_patterns: []
      alerts:
        webhook_url: "https://example.com/firmware-alert"
        use_syslog: true
      watcher:
        mode: poll
        poll_interval: 10
        debounce_ms: 250

  self_integrity_path: "/etc/watchdogfs/self.sha256"
  ```
  > Each named job spawns its own watcher thread. Baseline files are generated per job (e.g. `baseline_web_config.json`, `baseline_firmware_files.json`). Alerts from each job include the job name in the payload when rendering a template.

- **Dynamic Configuration Reload**  
  When running in daemon mode (`watchdogfs start --daemon`), WatchdogFS watches `config.yaml` for changes and automatically adds, removes, or reloads jobs at runtime—no restart needed.  
  - If a job is removed from `config.yaml`, its thread is stopped and its baseline file remains on disk.  
  - If a job’s configuration changes (any field under that job), the old thread is stopped and a new thread is spawned with the updated settings (including generating or reloading its baseline).  
  - If a new job is added under `jobs:`, a new watcher thread is spawned immediately.  
  This uses file‐system metadata polling (every 2 seconds) to detect changes to `config.yaml`’s modification time and reload the entire YAML.

- **Alert Payload Templating**  
  Instead of the fixed JSON `{"path":"…","old":"…","new":"…"}`, you can provide a Liquid‐style template string under `alerts.payload_template`. Available variables inside the template:  
  - `job_name` (string)  
  - `path` (string)  
  - `old` (string, previous checksum)  
  - `new` (string, updated checksum)  
  Example template in `config.yaml`:  
  ```yaml
  payload_template: |
    {
      "job":"{{job_name}}",
      "file":"{{path}}",
      "before":"{{old}}",
      "after":"{{new}}",
      "timestamp":"{{ now | date: "%Y-%m-%dT%H:%M:%SZ" }}"
    }
  ```  
  If templating fails (parse or render error), WatchdogFS falls back to the default JSON payload.

- **Plugin Interface**  
  You can now specify `plugin_path: <path-to-shared-lib>` under `alerts:` in each job. At runtime, WatchdogFS will load the shared library (via `libloading`) and look for a C‐ABI function named `run_alert(const char* payload) -> i32`. If the plugin returns non-zero, an error is logged; otherwise, it’s treated as successful. Example in `config.yaml`:  
  ```yaml
  alerts:
    plugin_path: "/usr/lib/watchdogfs/custom_alert.so"
  ```  
  A plugin receives the rendered JSON payload (after templating, if any) as a C‐string. It can perform arbitrary actions—e.g. write to a database, forward to another system, etc.

- **Real-time Monitoring (inotify)**  
  Uses Linux’s `inotify` through the `notify` crate (latest version) to watch file events and immediately respond to modifications.

- **Polling Fallback**  
  When `inotify` is unavailable (e.g. on some embedded kernels), WatchdogFS can poll file metadata at a configurable interval (`watcher.mode = "poll"`). Each job uses its own `poll_interval` (in seconds).

- **SHA-256 Baseline Generation & Verification**  
  - `watchdogfs init` writes a starter `config.yaml` with no jobs.  
  - `watchdogfs baseline` (or `integrity::generate_baseline()`) computes SHA-256 checksums for each path in every job’s `watch_paths` and writes a `baseline_<job_name>.json`.  
  - On subsequent runs, it loads those baseline files and compares checksums.  
  - Generating a baseline for a new job happens on first invocation of `baseline` or when starting a new job thread.

- **Alerting Subsystem**  
  - **Syslog** (via the `syslog` crate)  
  - **HTTP Webhook** (blocking POST with `reqwest + rustls`)  
  - **Local Script Execution** (no arguments; any non-zero exit code is logged as an error)  
  - **Plugin** (dynamic‐loadable `.so` with `run_alert` symbol)  
  Each alert channel receives the same JSON payload (templated or default).

- **Self-Integrity Check**  
  Optionally verify that the running binary’s SHA-256 matches an externally stored digest (via `--self-integrity-path` or `self_integrity_path:` in `config.yaml`). If verification fails, WatchdogFS aborts.

- **Flexible YAML Configuration**  
  - **Top-Level**  
    ```yaml
    jobs: HashMap<String, JobConfig>
    self_integrity_path: Option<String>
    ```  
  - **`JobConfig`**  
    ```rust
    pub struct JobConfig {
      pub watch_paths: Vec<String>,
      pub ignore_patterns: Vec<String>,         # reserved
      pub alerts: AlertsConfig,
      pub watcher: WatcherConfig,
    }
    ```  
  - **`AlertsConfig`**  
    ```rust
    pub struct AlertsConfig {
      pub webhook_url: Option<String>,
      pub script_path: Option<String>,
      pub use_syslog: bool,
      pub plugin_path: Option<String>,
      pub payload_template: Option<String>,
    }
    ```  
  - **`WatcherConfig`**  
    ```rust
    pub struct WatcherConfig {
      pub mode: String,            # "inotify" or "poll"
      pub poll_interval: Option<u64>,  # seconds
      pub debounce_ms: Option<u64>,    # milliseconds
    }
    ```

- **Unit Tests & CI**  
  - Comprehensive unit tests for checksum, baseline, config, CLI parsing, alerts, plugin interface, and self-check.  
  - GitHub Actions CI for `fmt` (Rust 1.72+), `clippy`, `cargo test`, and cross-compile (`x86_64-unknown-linux-musl`).

---

### ❌ Not Yet Implemented / Planned

- **Metrics & Observability**  
  Expose Prometheus metrics (files monitored, events processed, alerts sent).

- **Cross-Platform Support**  
  Current focus is on Linux. macOS inotify support (via FSEvents) is not yet in scope.

- **Packaging**  
  - Official Debian/Alpine packages (currently manual).  
  - Docker image (scratch/Musl) definition.  
  - Homebrew formula.

---

## 🚀 Quick Start / Usage

### 1. Install Rust (if not already)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup default stable
```

Ensure `cargo`, `rustc`, and `rustfmt` (from `rustup`) are in your `PATH`.

---

### 2. Clone & Compile

```bash
git clone https://github.com/yourusername/WatchdogFS.git
cd WatchdogFS

# Compile the main binary (latest toolchain)
cargo build --release

# (Optional) Cross‐compile a static Linux binary (requires `cross`)
cargo install cross
cross build --release --target x86_64-unknown-linux-musl
# resulting binary in target/x86_64-unknown-linux-musl/release/watchdogfs
```

---

### 3. Initialize Configuration

```bash
# Create a starter config.yaml in the current directory
./target/release/watchdogfs init --config ./config.yaml

# Edit config.yaml to define your jobs, watch_paths, alerts, etc.
```

**Sample `config.yaml`:**

```yaml
jobs:
  web_config:
    watch_paths:
      - /etc/nginx/nginx.conf
      - /etc/nginx/sites-enabled/
    ignore_patterns: []  # (reserved for future use)
    alerts:
      webhook_url: "https://example.com/webhook"
      script_path: "/usr/local/bin/nginx-alert.sh"
      use_syslog: true
      plugin_path: "/usr/lib/watchdogfs/nginx_plugin.so"
      payload_template: |
        {
          "job":"{{job_name}}",
          "path":"{{path}}",
          "old":"{{old}}",
          "new":"{{new}}"
        }
    watcher:
      mode: inotify
      poll_interval: 5
      debounce_ms: 500

  firmware_files:
    watch_paths:
      - /opt/firmware.bin
    ignore_patterns: []
    alerts:
      webhook_url: "https://example.com/firmware-alert"
      use_syslog: true
    watcher:
      mode: poll
      poll_interval: 10
      debounce_ms: 250

# Top-level self_integrity_path is optional; applies to the binary itself.
self_integrity_path: "/etc/watchdogfs/self.sha256"
```

---

### 4. Generate Baseline

```bash
./target/release/watchdogfs baseline
# => baseline_web_config.json, baseline_firmware_files.json, etc.
```

This will generate `baseline_<job_name>.json` for each defined job in `config.yaml`.

---

### 5. Run Foreground or Daemon Mode

```bash
# Foreground (blocks in terminal; no dynamic reload)
./target/release/watchdogfs start

# Daemon mode (background watcher + config reload every 2 seconds)
./target/release/watchdogfs start --daemon
```

- In **daemon mode**, WatchdogFS:
  1. Loads `config.yaml` (and spawns one thread per job).  
  2. Watches `config.yaml` for modifications and automatically adds/removes/reloads jobs when it changes.  
  3. Keeps each job running until the process is killed or the job is removed.

- Alerts are delivered according to each job’s `alerts` settings.

---

## ⚙️ Configuration Details

- **Top-Level `config.yaml` Fields**  
  ```yaml
  jobs: HashMap<String, JobConfig>
  self_integrity_path: Option<String>
  ```

- **`jobs: { <job_name>: JobConfig, … }`**  
  Each key is a unique job name (alphanumeric, underscores). The value is a `JobConfig`:

  - **`watch_paths: Vec<String>`**  
    A list of files or directories to monitor. Directories are watched recursively (inotify) or via polling (stat).  

  - **`ignore_patterns: Vec<String>`**  
    (Reserved) Glob patterns to ignore matching sub-paths under the watched paths.

  - **`alerts: AlertsConfig`**  
    ```yaml
    webhook_url: Option<String>     # e.g. "https://example.com/notify"
    script_path: Option<String>     # e.g. "/usr/local/bin/alert.sh"
    use_syslog: bool                # true ⇒ send JSON payload as a syslog INFO message
    plugin_path: Option<String>     # e.g. "/usr/lib/watchdogfs/custom_alert.so"
    payload_template: Option<String> # Liquid template (multiline string)
    ```
    - If `payload_template` is set, WatchdogFS attempts to parse and render it with the variables `{ job_name, path, old, new }`. On parse/render error, it falls back to the default JSON (`{"path":"…","old":"…","new":"…"}`).  
    - If `plugin_path` is set, WatchdogFS will attempt to load the shared library and call its `run_alert(const char* payload) -> int` symbol. A return value of zero is treated as success; any non-zero or load failure logs an error.

  - **`watcher: WatcherConfig`**  
    ```yaml
    mode: "inotify" | "poll"
    poll_interval: Option<u64>  # seconds, fallback if inotify unavailable or mode="poll"
    debounce_ms: Option<u64>    # milliseconds between handling duplicate events on the same path
    ```
    - `mode = "inotify"` (default) uses Linux inotify via the `notify` crate.  
    - `mode = "poll"` uses a periodic `stat()` loop, checking each path every `poll_interval` seconds.  
    - `debounce_ms` defaults to `500` ms if omitted.

  - **Example `JobConfig` in YAML**  
    ```yaml
    web_config:
      watch_paths:
        - /etc/nginx/nginx.conf
      ignore_patterns: []
      alerts:
        webhook_url: "https://example.com/notify"
        use_syslog: true
        payload_template: |
          {"job":"{{job_name}}","path":"{{path}}","old":"{{old}}","new":"{{new}}"}
      watcher:
        mode: inotify
        poll_interval: 5
        debounce_ms: 500
    ```

- **`self_integrity_path: Option<String>` (Top-Level)**  
  If provided, WatchdogFS reads this file (hex-encoded SHA256), computes the running binary’s SHA256, and aborts if they differ. Useful for ensuring the binary itself has not been tampered with.

---

## 📦 Packaging & Distribution

- **Cross-Compile (Musl)**  
  ```bash
  cargo install cross
  cross build --release --target x86_64-unknown-linux-musl
  ```
  - Produces a statically linked binary at `target/x86_64-unknown-linux-musl/release/watchdogfs` (latest Rust 1.72+).

- **Debian/Alpine**  
  Packaging is manual—for official `.deb` or `.apk`, fill in the corresponding package metadata under `debian/` or `alpine/`.  

- **Docker**  
  A `Dockerfile` definition (scratch + Musl) can be added under `docker/` for minimal containers.  

- **Homebrew (macOS)**  
  A Homebrew formula is not yet published; contributions welcome.

---

## 🎯 Development & Contribution

### Local Workflow

1. **Clone & Build**  
   ```bash
   git clone https://github.com/yourusername/WatchdogFS.git
   cd WatchdogFS
   rustup default stable
   cargo build          # Debug build
   ```

2. **Run Unit Tests**  
   ```bash
   cargo test
   ```

3. **Lint & Format**  
   ```bash
   cargo fmt -- --check
   cargo clippy -- -D warnings
   ```

4. **Run Locally**  
   ```bash
   # Create config.yaml (empty jobs)
   cargo run -- init --config ./config.yaml

   # Edit config.yaml: add jobs, paths, alerts, etc.
   # Generate baselines for all jobs
   cargo run -- baseline

   # Start in foreground
   cargo run -- start

   # Start as daemon (dynamic reload)
   cargo run -- start --daemon
   ```

5. **Cross-Compile**  
   ```bash
   cargo install cross
   cross build --release --target x86_64-unknown-linux-musl
   ```

### Contribution Guidelines

- ✨ **Fork** the repository and **create a feature branch** (e.g. `feat/profiles-support`).  
- 🔍 **Write unit tests** (and integration tests) for any new logic, especially around multi‐job config, templating, and plugin interface.  
- 📝 **Update documentation** (README, examples, code comments) for any new behavior.  
- 📦 **Run `cargo fmt`, `cargo clippy`, and `cargo test`** locally before opening a PR.  
- 🔄 **Submit a Pull Request** against `main` with a clear description of changes and how it affects existing functionality.  
- 🚀 **CI will run** and ensure your PR passes formatting, linting, testing, and cross-build checks.

---

## 🛠️ CI / CD

We use GitHub Actions to automate:

1. **Formatting**: `cargo fmt -- --check`  
2. **Linting**: `cargo clippy -- -D warnings`  
3. **Unit Tests**: `cargo test --all`  
4. **Cross-Compile**: `cross build --release --target x86_64-unknown-linux-musl`  

See [`.github/workflows/ci.yml`](.github/workflows/ci.yml) for details.  

---

## 📝 License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

## 🤝 Acknowledgements

- Built with [Rust](https://www.rust-lang.org/) (latest stable) for speed and safety.  
- File watching via [`notify`](https://crates.io/crates/notify) (inotify fallback, polling).  
- Checksum calculations via [`sha2`](https://crates.io/crates/sha2`).  
- Alerts delivered with [`reqwest`](https://crates.io/crates/reqwest`) + [`syslog`](https://crates.io/crates/syslog`).  
- Dynamic library loading via [`libloading`](https://crates.io/crates/libloading`).  
- Templating powered by [`liquid`](https://crates.io/crates/liquid`).  
- Logging powered by [`tracing`](https://crates.io/crates/tracing`).  
