use rand::Rng;
use std::fs::{File, OpenOptions};
use std::io::{Write, Error};
use std::path::Path;

const KEY_SIZE: usize = 32;
const KEY_FILE: &str = "secret.key";

pub fn generate_secure_key() -> Result<[u8; KEY_SIZE], Error> {
    let mut rng = rand::thread_rng();
    let mut key = [0u8; KEY_SIZE];
    rng.fill(&mut key);
    Ok(key)
}

pub fn save_key_to_file(key: &[u8], path: &str) -> Result<(), Error> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(Path::new(path))?;
    file.write_all(key)?;
    Ok(())
}

pub fn load_key_from_file(path: &str) -> Result<[u8; KEY_SIZE], Error> {
    let mut file = File::open(Path::new(path))?;
    let mut buffer = [0u8; KEY_SIZE];
    file.read_exact(&mut buffer)?;
    Ok(buffer)
}

pub fn generate_and_store_key() -> Result<(), Error> {
    let key = generate_secure_key()?;
    save_key_to_file(&key, KEY_FILE)?;
    println!("Key generated and saved to {}", KEY_FILE);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    #[test]
    fn test_key_generation() {
        let key1 = generate_secure_key().unwrap();
        let key2 = generate_secure_key().unwrap();
        assert_ne!(key1, key2);
        assert_eq!(key1.len(), KEY_SIZE);
    }

    #[test]
    fn test_key_storage() {
        let test_file = "test.key";
        let key = generate_secure_key().unwrap();
        
        save_key_to_file(&key, test_file).unwrap();
        let loaded_key = load_key_from_file(test_file).unwrap();
        
        assert_eq!(key, loaded_key);
        fs::remove_file(test_file).unwrap();
    }
}