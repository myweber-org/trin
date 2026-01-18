use std::fs::File;
use std::io::{Read, Result};
use sha2::{Sha256, Digest};
use blake3::Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHasher;

impl FileHasher {
    pub fn calculate_hash(file_path: &str, algorithm: HashAlgorithm) -> Result<String> {
        let mut file = File::open(file_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let hash = match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Hasher::new();
                hasher.update(&buffer);
                hasher.finalize().to_string()
            }
        };

        Ok(hash)
    }

    pub fn verify_integrity(file_path: &str, expected_hash: &str, algorithm: HashAlgorithm) -> Result<bool> {
        let calculated_hash = Self::calculate_hash(file_path, algorithm)?;
        Ok(calculated_hash == expected_hash)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_hash_calculation() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"test data for hashing")?;
        
        let hash = FileHasher::calculate_hash(temp_file.path().to_str().unwrap(), HashAlgorithm::Sha256)?;
        assert_eq!(hash.len(), 64);
        Ok(())
    }

    #[test]
    fn test_blake3_hash_calculation() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"another test payload")?;
        
        let hash = FileHasher::calculate_hash(temp_file.path().to_str().unwrap(), HashAlgorithm::Blake3)?;
        assert!(!hash.is_empty());
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        temp_file.write_all(b"verification test")?;
        
        let hash = FileHasher::calculate_hash(temp_file.path().to_str().unwrap(), HashAlgorithm::Sha256)?;
        let is_valid = FileHasher::verify_integrity(
            temp_file.path().to_str().unwrap(),
            &hash,
            HashAlgorithm::Sha256
        )?;
        
        assert!(is_valid);
        Ok(())
    }
}