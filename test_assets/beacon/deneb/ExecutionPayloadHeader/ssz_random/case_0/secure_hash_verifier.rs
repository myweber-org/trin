
use sha2::{Digest, Sha256};
use std::fs::File;
use std::io::{Read, Error};
use std::path::Path;

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_sha256<P: AsRef<Path>>(path: P) -> Result<String, Error> {
        let mut file = File::open(path)?;
        let mut hasher = Sha256::new();
        let mut buffer = [0; 4096];

        loop {
            let bytes_read = file.read(&mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            hasher.update(&buffer[..bytes_read]);
        }

        Ok(format!("{:x}", hasher.finalize()))
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
        let mut file = NamedTempFile::new()?;
        file.write_all(b"Test content")?;
        
        let hash = FileHashVerifier::calculate_sha256(file.path())?;
        let expected_hash = "a8a2f6ebe286697c527eb35a58b5539532e9b3ae3b64d4eb0a46fb657b41562c";
        
        assert!(FileHashVerifier::verify_file_integrity(
            file.path(),
            expected_hash
        )?);
        Ok(())
    }
}