use sha2::{Sha256, Sha512, Digest};
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;

pub enum HashAlgorithm {
    SHA256,
    SHA512,
}

pub struct HashGenerator;

impl HashGenerator {
    pub fn hash_file<P: AsRef<Path>>(path: P, algorithm: HashAlgorithm) -> Result<String, std::io::Error> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        let hash = match algorithm {
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::SHA512 => {
                let mut hasher = Sha512::new();
                hasher.update(&buffer);
                format!("{:x}", hasher.finalize())
            }
        };

        Ok(hash)
    }

    pub fn hash_string(input: &str, algorithm: HashAlgorithm) -> String {
        let bytes = input.as_bytes();
        
        match algorithm {
            HashAlgorithm::SHA256 => {
                let mut hasher = Sha256::new();
                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            }
            HashAlgorithm::SHA512 => {
                let mut hasher = Sha512::new();
                hasher.update(bytes);
                format!("{:x}", hasher.finalize())
            }
        }
    }

    pub fn verify_file_hash<P: AsRef<Path>>(path: P, expected_hash: &str, algorithm: HashAlgorithm) -> Result<bool, std::io::Error> {
        let calculated_hash = Self::hash_file(path, algorithm)?;
        Ok(calculated_hash == expected_hash.to_lowercase())
    }

    pub fn save_hash_to_file<P: AsRef<Path>>(path: P, hash: &str) -> Result<(), std::io::Error> {
        let mut file = File::create(path)?;
        file.write_all(hash.as_bytes())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hashing() {
        let test_string = "Hello, World!";
        
        let sha256_hash = HashGenerator::hash_string(test_string, HashAlgorithm::SHA256);
        let sha512_hash = HashGenerator::hash_string(test_string, HashAlgorithm::SHA512);
        
        assert_eq!(sha256_hash.len(), 64);
        assert_eq!(sha512_hash.len(), 128);
        assert_ne!(sha256_hash, sha512_hash);
    }

    #[test]
    fn test_file_hashing() -> Result<(), std::io::Error> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Test content for hashing")?;
        
        let path = temp_file.path();
        let sha256_hash = HashGenerator::hash_file(path, HashAlgorithm::SHA256)?;
        let sha512_hash = HashGenerator::hash_file(path, HashAlgorithm::SHA512)?;
        
        assert!(!sha256_hash.is_empty());
        assert!(!sha512_hash.is_empty());
        assert_ne!(sha256_hash, sha512_hash);
        
        Ok(())
    }

    #[test]
    fn test_hash_verification() -> Result<(), std::io::Error> {
        let mut temp_file = NamedTempFile::new()?;
        writeln!(temp_file, "Verification test")?;
        
        let path = temp_file.path();
        let hash = HashGenerator::hash_file(path, HashAlgorithm::SHA256)?;
        
        let is_valid = HashGenerator::verify_file_hash(path, &hash, HashAlgorithm::SHA256)?;
        assert!(is_valid);
        
        let invalid_check = HashGenerator::verify_file_hash(path, "invalidhash123", HashAlgorithm::SHA256)?;
        assert!(!invalid_check);
        
        Ok(())
    }
}