## 📌 **WatchdogFS – Compact Project Summary**

### 🚩 **Overview**

WatchdogFS is a high-performance, lightweight filesystem integrity monitoring CLI tool designed for Linux and embedded environments. It ensures critical file integrity by detecting unexpected modifications, comparing files against pre-approved SHA256 baselines, and providing instant alerts, while running efficiently with minimal dependencies.

---

### 🎯 **Goals & Expected Results**

* **Rapid detection** of unauthorized filesystem changes.
* **Minimal binary size** (<1.5MB static), high speed, and low resource usage.
* **Compatibility** from Ubuntu servers down to minimal embedded systems (BusyBox, Alpine).
* Clear, structured logging & alert mechanisms for seamless integration into existing infrastructures.

---

### 🔑 **Main Features (v1.0)**

* **Real-time monitoring** via inotify (Linux native FS events).
* **Polling fallback mode** for compatibility (no inotify available).
* **Baseline checksum validation** (SHA256), with automated baseline generation.
* **Alerting mechanisms**: local logs, syslog, webhook integration (HTTP POST), custom local script execution.
* **Self-integrity checks** to protect against binary modification.
* Flexible configuration profiles (YAML) with glob patterns for file/directory selection.
* Execution modes: manual CLI, headless/daemon background mode.

---

### 🗓 **Milestones & Deliverables (6-month Roadmap)**

* **Month 1:** CLI core (init, verify, check), YAML config handling, baseline checksums.
* **Month 2:** Runtime watcher (inotify), structured logging (JSON/syslog), debounce logic.
* **Month 3:** Polling fallback mode, embedded Linux static build (cross-compile with musl libc).
* **Month 4:** Webhook alerts, local command execution, binary self-integrity checks.
* **Month 5:** Advanced configurability (profiles, globs), headless mode, automated baseline updates.
* **Month 6:** Comprehensive testing, benchmarking, packaging (.deb, .rpm, static bin), public v1.0 release, documentation.

---

### ⚠️ **Challenges & Solutions**

* **File-system race conditions**: Managed via debounce, retries, and multiple checks.
* **Lack of inotify**: Efficient polling fallback (stat() + metadata-based intervals).
* **False positives on legitimate file changes**: Configurable maintenance window/silent mode.
* **Log file growth**: Auto-rotation, compression, deduplication, configurable TTL.
* **Permissions issues**: Capabilities tuning (CAP_DAC_READ_SEARCH) and privilege documentation.

---

### 🚧 **Inherent Limitations**

* Indirect filesystem changes (mount --bind, tmpfs) require periodic polling checks.
* Systems with severely limited kernel support may have performance penalties (pure polling).

---

### 🛠️ **Recommended Tech Stack**

* **Language**: Rust (optimized performance, memory-safe, small binary)
* **CLI parser**: clap
* **File watching**: notify crate (inotify abstraction)
* **Hashing**: sha2
* **Logging**: tracing crate with JSON appender
* **Configuration**: YAML (serde_yaml)
* **Packaging**: Cross-compilation via musl libc (static binaries)
* **Targets**: Ubuntu, Alpine, BusyBox-based embedded systems (x86\_64, ARM64)

---

### ⚡️ **Typical Usage Example**

shell
watchdogfs init /etc/nginx/nginx.conf /opt/firmware.bin
watchdogfs baseline generate
watchdogfs start --daemon


---

### 🎖️ **Final Result**

By v1.0, WatchdogFS will provide a compact, robust, and highly efficient integrity monitoring solution, suitable for diverse Linux environments—from servers to IoT devices—meeting compliance requirements and enhancing overall security posture.