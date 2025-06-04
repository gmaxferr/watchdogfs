use anyhow::{Context, Result, bail};
use libloading::{Library, Symbol};
use std::ffi::CString;
use std::fmt;

/// Represents a dynamically loaded plugin. We keep the `Library` handle alive so
/// that `run_alert` symbol stays valid for as long as this struct exists.
pub struct Plugin {
    _lib: Library,
    run_alert_fn: Symbol<'static, unsafe extern "C" fn(*const std::os::raw::c_char) -> i32>,
}

// Provide a minimal `Debug` implementation so that `expect_err` in tests compiles.
// We don't print internals, just indicate the struct type.
impl fmt::Debug for Plugin {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Plugin").finish()
    }
}

impl Plugin {
    /// Load a shared library at `path` and look up the `run_alert` symbol.
    pub fn load(path: &str) -> Result<Self> {
        // SAFETY: We trust the user‐specified path to point to a valid shared library compiled
        // against the same `run_alert` ABI. libloading will return an error if it can't load or
        // the symbol isn't found.
        let lib = unsafe { Library::new(path) }
            .with_context(|| format!("failed to load plugin library `{}`", path))?;

        // Once the library is loaded, grab a static reference to the `run_alert` function symbol.
        // We require it to have the exact C ABI: `fn(*const c_char) -> i32`.
        let run_alert_fn: Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> i32> =
            unsafe { lib.get(b"run_alert\0") }
                .with_context(|| format!("symbol `run_alert` not found in `{}`", path))?;

        // Cast from `Symbol<'_, _>` to `Symbol<'static, _>` so that it can be stored.
        #[allow(clippy::missing_transmute_annotations)]
        let run_alert_fn: Symbol<
            'static,
            unsafe extern "C" fn(*const std::os::raw::c_char) -> i32,
        > = unsafe {
            std::mem::transmute::<
                Symbol<unsafe extern "C" fn(*const std::os::raw::c_char) -> i32>,
                Symbol<'static, unsafe extern "C" fn(*const std::os::raw::c_char) -> i32>,
            >(run_alert_fn)
        };

        Ok(Plugin {
            _lib: lib,
            run_alert_fn,
        })
    }

    /// Execute the plugin’s `run_alert` function, passing `payload` as a C‐string.
    /// Returns `Ok(())` if `run_alert` returns zero, otherwise `Err`.
    pub fn execute(&self, payload: &str) -> Result<()> {
        let c_payload = CString::new(payload).context("failed to convert payload to C‐string")?;
        let ret_code = unsafe { (self.run_alert_fn)(c_payload.as_ptr()) };

        if ret_code != 0 {
            bail!("plugin returned non‐zero exit code: {}", ret_code);
        }
        Ok(())
    }
}

/// Convenience wrapper. Load, execute once, then drop.
pub fn execute_plugin(path: &str, payload: &str) -> Result<()> {
    let plugin = Plugin::load(path)?;
    plugin.execute(payload)
}

#[cfg(test)]
mod tests {
    use super::Plugin;
    use std::io::Write;
    use tempfile::NamedTempFile;

    /// Note: In real usage, a plugin is a compiled shared‐object (.so).
    /// For testing, we can create a tiny C source that returns zero. But here we
    /// can at least verify that loading a non‐existent file errors out.
    #[test]
    fn load_missing_plugin_fails() {
        let err = Plugin::load("/does/not/exist.so").expect_err("should fail loading");
        assert!(
            err.to_string().contains("failed to load plugin library"),
            "unexpected error: {}",
            err
        );
    }

    /// If a file exists but doesn’t define `run_alert`, we should get either a
    /// "failed to load plugin library" (non‐shared‐object) or "symbol `run_alert` not found" error.
    #[test]
    fn missing_symbol_errors() -> anyhow::Result<()> {
        // Create a dummy file (not a real .so). The loader may fail early.
        let mut tmp = NamedTempFile::new()?;
        writeln!(tmp, "not a shared object").unwrap();
        let path = tmp.path().to_str().unwrap();
        let err = Plugin::load(path).expect_err("should fail symbol lookup or load");
        let msg = err.to_string();
        assert!(
            msg.contains("symbol `run_alert`") || msg.contains("failed to load plugin library"),
            "unexpected error: {}",
            msg
        );
        Ok(())
    }
}
