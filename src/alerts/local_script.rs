// Local script execution
use std::process::Command;
use anyhow::Result;

pub fn execute_script(script_path: &str) -> Result<()> {
    Command::new(script_path).status()?;
    Ok(())
}
