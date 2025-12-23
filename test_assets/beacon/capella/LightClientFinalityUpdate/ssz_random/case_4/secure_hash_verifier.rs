
use std::fs::File;
use std::io::{Read, self};
use std::path::Path;
use sha2::{Sha256, Digest};

pub struct HashVerifier {
    algorithm: HashAlgorithm,
}

pub enum HashAlgorithm {
    SHA256,
}

impl HashVerifier {
    pub fn new(algorithm: HashAlgorithm) -> Self {
        HashVerifier { algorithm }
    }

    pub fn compute_file_hash(&self, file_path: &Path) -> io::Result<String> {
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

    pub fn verify_file_integrity(
        &self,
        file_path: &Path,
        expected_hash: &str
    ) -> io::Result<bool> {
        let computed_hash = self.compute_file_hash(file_path)?;
        Ok(computed_hash == expected_hash.to_lowercase())
    }

    pub fn compute_string_hash(&self, data: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(data.as_bytes());
        let result = hasher.finalize();
        format!("{:x}", result)
    }
}

pub fn verify_files_match(file1: &Path, file2: &Path) -> io::Result<bool> {
    let verifier = HashVerifier::new(HashAlgorithm::SHA256);
    let hash1 = verifier.compute_file_hash(file1)?;
    let hash2 = verifier.compute_file_hash(file2)?;
    Ok(hash1 == hash2)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn test_string_hash_consistency() {
        let verifier = HashVerifier::new(HashAlgorithm::SHA256);
        let data = "test data for hashing";
        let hash1 = verifier.compute_string_hash(data);
        let hash2 = verifier.compute_string_hash(data);
        assert_eq!(hash1, hash2);
        assert_eq!(hash1.len(), 64);
    }

    #[test]
    fn test_file_hash_verification() -> io::Result<()> {
        let mut temp_file = NamedTempFile::new()?;
        let content = b"file content to verify";
        temp_file.write_all(content)?;

        let verifier = HashVerifier::new(HashAlgorithm::SHA256);
        let computed_hash = verifier.compute_file_hash(temp_file.path())?;
        
        let is_valid = verifier.verify_file_integrity(
            temp_file.path(),
            &computed_hash
        )?;
        
        assert!(is_valid);
        Ok(())
    }

    #[test]
    fn test_file_comparison() -> io::Result<()> {
        let mut file1 = NamedTempFile::new()?;
        let mut file2 = NamedTempFile::new()?;
        
        file1.write_all(b"identical content")?;
        file2.write_all(b"identical content")?;
        
        let result = verify_files_match(file1.path(), file2.path())?;
        assert!(result);
        
        file2.write_all(b"different")?;
        let result = verify_files_match(file1.path(), file2.path())?;
        assert!(!result);
        
        Ok(())
    }
}