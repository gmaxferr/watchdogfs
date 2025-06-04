# WatchdogFS  
*Filesystem Integrity Monitoring for Linux & Embedded*

[![CI](https://github.com/gmaxferr/watchdogfs/actions/workflows/ci.yml/badge.svg)](https://github.com/gmaxferr/watchdogfs/actions/workflows/ci.yml)
---

## 📋 Description

**WatchdogFS** is a lightweight, high-performance CLI tool written in Rust that continuously monitors specified files or directories for unauthorized changes. It computes and compares SHA-256 checksums against a stored baseline, and emits alerts via syslog, HTTP webhooks, or customizable local scripts. Designed for both server and embedded (BusyBox/Alpine) environments, WatchdogFS is memory-safe, cross-compile-friendly, and delivers real-time protection with minimal dependencies.

---

## ⭐ Features

### ✅ Already Implemented

- **Real-time Monitoring (inotify)**  
  Uses Linux’s `inotify` through the `notify` crate to watch file events and immediately respond to modifications.

- **Polling Fallback**  
  When `inotify` is unavailable (e.g. on some embedded kernels), WatchdogFS can poll file metadata at a configurable interval.

- **SHA-256 Baseline Generation & Verification**  
  - `watchdogfs init` writes a starter `config.yaml`.  
  - `watchdogfs baseline` computes SHA-256 checksums for each path in `watch_paths` and writes `baseline.json`.  
  - On subsequent runs, compares current checksums to `baseline.json`.

- **Alerting Subsystem**  
  - **Syslog** (via `syslog` crate)  
  - **HTTP Webhook** (blocking POST with `reqwest+rustls`)  
  - **Local Script Execution**  
  Alerts dispatch a JSON payload containing the file path, old checksum, and new checksum.

- **Self-Integrity Check**  
  Optionally verify that the running binary’s SHA-256 matches an externally stored digest (via `--self-integrity-path`).

- **Flexible YAML Configuration**  
  - `watch_paths: Vec<String>`  
  - `ignore_patterns: Vec<String>` (currently stubbed, future support)  
  - `alerts: { webhook_url, script_path, use_syslog }`  
  - `watcher: { mode: "inotify" | "poll", poll_interval: u64, debounce_ms: u64 }`  
  - `self_integrity_path: Option<String>`

- **Unit Tests & CI**  
  - Comprehensive unit tests for checksum, baseline, config, CLI parsing, alerts, and self-check.  
  - GitHub Actions CI for `fmt`, `clippy`, `cargo test`, and cross-compile (`x86_64-unknown-linux-musl`).

---

### ❌ Not Yet Implemented / Planned

- **Named Profiles / Multiple “Jobs”**  
  Support multiple named watch-sets within a single `config.yaml`.

- **Dynamic Configuration Reload**  
  Watch and re-load `config.yaml` at runtime without restarting the daemon.

- **Alert Payload Templating**  
  User-customizable templates (e.g. Liquid/Go-Template) for JSON output.

- **Metrics & Observability**  
  Expose Prometheus metrics (files monitored, events processed, alerts sent).

- **Cross-Platform Support**  
  Current focus is on Linux. macOS inotify support (via FSEvents) is not yet in scope.

- **Plugin Interface**  
  A Rust plugin trait or dynamic-loadable library hook for custom alert handlers.

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

# Compile the main binary:
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

# Edit config.yaml to add watch_paths, alerts, etc.
```

**Sample `config.yaml`:**

```yaml
watch_paths:
  - /etc/nginx/nginx.conf
  - /opt/firmware.bin

ignore_patterns: []  # (future support)

alerts:
  webhook_url: "https://example.com/notify"
  script_path: "/usr/local/bin/alert.sh"
  use_syslog: true

watcher:
  mode: inotify
  poll_interval: 5
  debounce_ms: 500

self_integrity_path: "/etc/watchdogfs/self.sha256"
```

---

### 4. Generate Baseline

```bash
./target/release/watchdogfs baseline
# => baseline.json is written in CWD, containing a map of <path, sha256>
```

---

### 5. Run Foreground or Daemon Mode

```bash
# Foreground (blocks in terminal)
./target/release/watchdogfs start 

# Daemon mode (blocks until killed, logs to stderr/syslog)
./target/release/watchdogfs start --daemon
```

- In daemon mode, WatchdogFS runs in the background (no TTY output).  
- Alerts are delivered per `config.yaml`.  

---

## ⚙️ Configuration Details

- **`watch_paths: Vec<String>`**  
  A list of files or directories to monitor. Directories are watched recursively.

- **`ignore_patterns: Vec<String>`**  
  (Reserved) Glob patterns to ignore specific sub-paths.

- **`alerts:`**  
  - `webhook_url: Option<String>`  
    If set, WatchdogFS POSTs JSON `{"path": "...","old":"...","new":"..."}` to the URL.  
  - `script_path: Option<String>`  
    If set, WatchdogFS runs that script (no arguments) on a change. Script’s exit status non-zero logs an error.  
  - `use_syslog: bool`  
    If `true`, WatchdogFS sends a syslog INFO message with the JSON payload.

- **`watcher:`**  
  - `mode: "inotify" | "poll"`  
    Use Linux inotify (default) or fallback to polling.  
  - `poll_interval: Option<u64>`  
    Seconds between stat( ) checks when `mode: poll`. Default is `5`.  
  - `debounce_ms: Option<u64>`  
    Milliseconds to ignore repeated events on the same file. Default is `500`.

- **`self_integrity_path: Option<String>`**  
  If provided, WatchdogFS reads this file (hex-encoded SHA256), computes the running binary’s SHA256, and aborts if they differ.

---

## 📦 Packaging & Distribution

- **Cross-Compile (Musl)**  
  ```bash
  cargo install cross
  cross build --release --target x86_64-unknown-linux-musl
  ```
  - Produces a statically linked binary at `target/x86_64-unknown-linux-musl/release/watchdogfs`.

---

## 🎯 Development & Contribution

### Local Workflow

1. **Clone & Build**  
   ```bash
   git clone https://github.com/yourusername/WatchdogFS.git
   cd WatchdogFS
   rustup default stable
   cargo build          # debug build
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
   cargo run -- init --config ./config.yaml
   cargo run -- baseline
   cargo run -- start
   ```

5. **Cross-Compile**  
   ```bash
   cargo install cross
   cross build --release --target x86_64-unknown-linux-musl
   ```

### Contribution Guidelines

- ✨ **Fork** the repository and **create a feature branch** (e.g. `feat/profiles-support`).  
- 🔍 **Write unit tests** or integration tests for any new logic.  
- 📝 **Update documentation** (README, examples, code comments) for any new behavior.  
- 📦 **Run `cargo fmt`, `cargo clippy`, and `cargo test`** locally before opening a PR.  
- 🔄 **Submit a Pull Request** against `main` with a clear description of changes.  
- 🚀 **CI will run** and ensure your PR passes formatting, linting, testing, and cross-build checks.  

---

## 🛠️ CI / CD

We use GitHub Actions to automate:

1. **Formatting**: `cargo fmt -- --check`  
2. **Linting**: `cargo clippy -- -D warnings`  
3. **Unit Tests**: `cargo test --all`  
4. **Cross-Compile**: `cross build --release --target x86_64-unknown-linux-musl`  

See `.github/workflows/ci.yml` for details.

---

## 📝 License

This project is licensed under the **MIT License**. See [LICENSE](LICENSE) for details.

---

## 🤝 Acknowledgements

- Built with [Rust](https://www.rust-lang.org/) for speed and safety.  
- File watching via [`notify`](https://crates.io/crates/notify) (inotify abstraction).  
- Checksum calculations via [`sha2`](https://crates.io/crates/sha2).  
- Alerts delivered with [`reqwest`](https://crates.io/crates/reqwest`) and [`syslog`](https://crates.io/crates/syslog`).  
- Logging powered by [`tracing`](https://crates.io/crates/tracing`).  
