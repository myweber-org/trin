
use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};

pub struct HashVerifier;

impl HashVerifier {
    pub fn calculate_file_hash(file_path: &str) -> Result<String> {
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
        
        Ok(format!("{:x}", hasher.finalize()))
    }
    
    pub fn verify_file_integrity(file_path: &str, expected_hash: &str) -> Result<bool> {
        let calculated_hash = Self::calculate_file_hash(file_path)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }
    
    pub fn compare_files(file1_path: &str, file2_path: &str) -> Result<bool> {
        let hash1 = Self::calculate_file_hash(file1_path)?;
        let hash2 = Self::calculate_file_hash(file2_path)?;
        Ok(hash1 == hash2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;
    
    #[test]
    fn test_identical_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        let test_data = b"Test data for hash verification";
        file1.write_all(test_data)?;
        file2.write_all(test_data)?;
        
        let result = HashVerifier::compare_files(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        )?;
        
        assert!(result);
        Ok(())
    }
    
    #[test]
    fn test_different_files() -> Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"Data 1")?;
        file2.write_all(b"Data 2")?;
        
        let result = HashVerifier::compare_files(
            file1.path().to_str().unwrap(),
            file2.path().to_str().unwrap()
        )?;
        
        assert!(!result);
        Ok(())
    }
}