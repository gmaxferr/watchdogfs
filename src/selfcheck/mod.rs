use anyhow::{Context, Result};
use sha2::{Digest, Sha256};
use std::{env, fs};

/// Compute SHA256 over the current executable, compare to the contents of `path`.
/// The file must contain the hex-encoded expected SHA256.
pub fn verify(self_integrity_path: &str) -> Result<()> {
    // 1) Read expected hex digest
    let expected_hex = fs::read_to_string(self_integrity_path)
        .with_context(|| format!("reading self-integrity file `{}`", self_integrity_path))?
        .trim()
        .to_string();

    // 2) Locate our own binary
    let exe_path = env::current_exe().context("getting path to current executable")?;

    let data = fs::read(&exe_path)
        .with_context(|| format!("reading executable `{}`", exe_path.display()))?;

    // 3) Compute SHA256
    let mut hasher = Sha256::new();
    hasher.update(&data);
    let actual_hex = format!("{:x}", hasher.finalize());

    // 4) Compare
    if expected_hex != actual_hex {
        anyhow::bail!(
            "self-integrity failure: expected {} but got {}",
            expected_hex,
            actual_hex
        );
    }

    println!("✅ Self-integrity verified for {}", exe_path.display());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::verify;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn missing_integrity_file_errors() {
        let err = verify("does_not_exist.sha").unwrap_err();
        assert!(
            err.to_string().contains("reading self-integrity file"),
            "unexpected message: {}",
            err
        );
    }

    #[test]
    fn mismatched_hash_errors() {
        // create a fake integrity file with wrong digest
        let mut tmp = NamedTempFile::new().unwrap();
        writeln!(tmp, "abcdef123456").unwrap();
        let path = tmp.path().to_str().unwrap();
        let err = verify(path).unwrap_err();
        assert!(
            err.to_string().contains("self-integrity failure"),
            "unexpected message: {}",
            err
        );
    }
}
