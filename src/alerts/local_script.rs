// Local script execution
use std::process::Command;
use anyhow::Result;

/// Execute a local script (no arguments).  
/// Fails if the script returns a non-zero exit code.
pub fn execute_script(script_path: &str) -> Result<()> {
    let status = Command::new(script_path).status()?;
    if !status.success() {
        anyhow::bail!(
            "script {} exited with code {:?}",
            script_path,
            status.code()
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::execute_script;
    use anyhow::Result;

    #[test]
    fn execute_script_true_succeeds() -> Result<()> {
        // `true` should exist on Unix and exit with code 0
        execute_script("true")?;
        Ok(())
    }

    #[test]
    fn execute_script_false_fails() {
        // `false` exists and returns non-zero
        let err = execute_script("false").expect_err("false should exit non-zero");
        let msg = format!("{}", err);
        assert!(
            msg.contains("exited with code"),
            "unexpected error message: {}",
            msg
        );
    }

    #[test]
    fn execute_nonexistent_script_fails() {
        let err = execute_script("/path/does/not/exist").expect_err("should error");
        assert!(
            err.to_string().contains("No such file or directory"),
            "unexpected error kind: {}",
            err
        );
    }
}