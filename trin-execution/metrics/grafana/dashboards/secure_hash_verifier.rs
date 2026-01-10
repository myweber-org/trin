use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use sha2::{Sha256, Digest};
use blake3::Hasher;

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

pub struct FileHashVerifier;

impl FileHashVerifier {
    pub fn calculate_hash<P: AsRef<Path>>(
        path: P,
        algorithm: HashAlgorithm,
    ) -> io::Result<String> {
        let mut file = File::open(path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        match algorithm {
            HashAlgorithm::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&buffer);
                Ok(format!("{:x}", hasher.finalize()))
            }
            HashAlgorithm::Blake3 => {
                let mut hasher = Hasher::new();
                hasher.update(&buffer);
                Ok(hasher.finalize().to_hex().to_string())
            }
        }
    }

    pub fn verify_hash<P: AsRef<Path>>(
        path: P,
        expected_hash: &str,
        algorithm: HashAlgorithm,
    ) -> io::Result<bool> {
        let calculated = Self::calculate_hash(path, algorithm)?;
        Ok(calculated == expected_hash.to_lowercase())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_sha256_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Test content for hashing").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Sha256
        ).unwrap();
        
        assert!(FileHashVerifier::verify_hash(
            temp_file.path(),
            &hash,
            HashAlgorithm::Sha256
        ).unwrap());
    }

    #[test]
    fn test_blake3_verification() {
        let mut temp_file = NamedTempFile::new().unwrap();
        writeln!(temp_file, "Another test content").unwrap();
        
        let hash = FileHashVerifier::calculate_hash(
            temp_file.path(),
            HashAlgorithm::Blake3
        ).unwrap();
        
        assert!(FileHashVerifier::verify_hash(
            temp_file.path(),
            &hash,
            HashAlgorithm::Blake3
        ).unwrap());
    }
}