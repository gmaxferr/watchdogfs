// Baseline generation & validation
use crate::integrity::calculate_checksum;
use anyhow::Result;
use std::collections::HashMap;

pub type Baseline = HashMap<String, String>;

pub fn generate(paths: &[String]) -> Result<Baseline> {
    let mut baseline = Baseline::new();
    for path in paths {
        let checksum = calculate_checksum(path)?;
        baseline.insert(path.clone(), checksum);
    }
    Ok(baseline)
}

#[cfg(test)]
mod tests {
    use super::generate;
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn baseline_two_files() {
        let dir = tempdir().unwrap();

        let a = dir.path().join("a.txt");
        let mut fa = File::create(&a).unwrap();
        write!(fa, "foo").unwrap();

        let b = dir.path().join("b.txt");
        let mut fb = File::create(&b).unwrap();
        write!(fb, "bar").unwrap();

        let paths = vec![
            a.to_str().unwrap().to_string(),
            b.to_str().unwrap().to_string(),
        ];
        let baseline = generate(&paths).unwrap();
        assert_eq!(baseline.len(), 2);
        // Ensure keys match
        assert!(baseline.contains_key(&paths[0]));
        assert!(baseline.contains_key(&paths[1]));
    }
}