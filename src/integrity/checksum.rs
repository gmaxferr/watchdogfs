// SHA256 checksum logic
use sha2::{Sha256, Digest};
use std::fs;
use anyhow::Result;

pub fn calculate_checksum(path: &str) -> Result<String> {
    let data = fs::read(path)?;
    let mut hasher = Sha256::new();
    hasher.update(&data);
    Ok(format!("{:x}", hasher.finalize()))
}


#[cfg(test)]
mod tests {
    use super::calculate_checksum;
    use tempfile::NamedTempFile;
    use std::io::Write;
    use std::fs;

    #[test]
    fn checksum_empty_file() {
        let tmp = NamedTempFile::new().unwrap();
        // ensure file exists on disk
        fs::write(tmp.path(), &[]).unwrap();
        let sum = calculate_checksum(tmp.path().to_str().unwrap()).unwrap();
        // SHA256 of empty content
        assert_eq!(
            sum,
            "e3b0c44298fc1c149afbf4c8996fb924\
             27ae41e4649b934ca495991b7852b855"
        );
    }

    #[test]
    fn checksum_hello_world() {
        let mut tmp = NamedTempFile::new().unwrap();
        write!(tmp, "hello world").unwrap();
        let sum = calculate_checksum(tmp.path().to_str().unwrap()).unwrap();
        assert_eq!(
            sum,
            "b94d27b9934d3e08a52e52d7da7dabfa\
             c484efe37a5380ee9088f7ace2efcde9"
        );
    }
}