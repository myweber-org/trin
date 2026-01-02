
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_sha256(file_path: &Path) -> Result<String, Error> {
        let mut file = File::open(file_path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        let result = hasher.finalize();
        Ok(format!("{:x}", result))
    }

    pub fn verify_integrity(file_path: &Path, expected_hash: &str) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files(file1: &Path, file2: &Path) -> Result<bool, Error> {
        let hash1 = Self::calculate_sha256(file1)?;
        let hash2 = Self::calculate_sha256(file2)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_identical_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;

        file1.write_all(b"test content")?;
        file2.write_all(b"test content")?;

        assert!(HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;

        file1.write_all(b"content one")?;
        file2.write_all(b"content two")?;

        assert!(!HashVerifier::compare_files(file1.path(), file2.path())?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<(), Error> {
        let mut file = NamedTempFile::new()?;
        file.write_all(b"verification test")?;

        let hash = HashVerifier::calculate_sha256(file.path())?;
        assert!(HashVerifier::verify_integrity(file.path(), &hash)?);
        Ok(())
    }
}