
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 8192];

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

    pub fn verify_file_integrity<P: AsRef<Path>>(
        file_path: P,
        expected_hash: &str
    ) -> Result<bool, Error> {
        let calculated_hash = Self::calculate_sha256(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn compare_files<P: AsRef<Path>>(
        file1: P,
        file2: P
    ) -> Result<bool, Error> {
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
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        assert!(FileHashVerifier::compare_files(
            file1.path(),
            file2.path()
        )?);
        Ok(())
    }

    #[test]
    fn test_different_files() -> Result<(), Error> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Data 1")?;
        file2.write_all(b"Data 2")?;
        
        assert!(!FileHashVerifier::compare_files(
            file1.path(),
            file2.path()
        )?);
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<(), Error> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"Verification test")?;
        
        let hash = FileHashVerifier::calculate_sha256(temp_file.path())?;
        assert!(FileHashVerifier::verify_file_integrity(
            temp_file.path(),
            &hash
        )?);
        
        assert!(!FileHashVerifier::verify_file_integrity(
            temp_file.path(),
            "invalidhash123"
        )?);
        Ok(())
    }
}