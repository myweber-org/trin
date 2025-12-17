
use rand_core::{OsRng, RngCore};

pub struct SecureRandom {
    rng: OsRng,
}

impl SecureRandom {
    pub fn new() -> Result<Self, rand_core::Error> {
        Ok(Self { rng: OsRng })
    }

    pub fn generate_bytes(&mut self, buffer: &mut [u8]) -> Result<(), rand_core::Error> {
        self.rng.fill_bytes(buffer);
        Ok(())
    }

    pub fn generate_u64(&mut self) -> Result<u64, rand_core::Error> {
        let mut bytes = [0u8; 8];
        self.rng.fill_bytes(&mut bytes);
        Ok(u64::from_le_bytes(bytes))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_bytes() {
        let mut rng = SecureRandom::new().unwrap();
        let mut buffer = [0u8; 32];
        rng.generate_bytes(&mut buffer).unwrap();
        
        let mut zero_count = 0;
        for &byte in &buffer {
            if byte == 0 {
                zero_count += 1;
            }
        }
        assert!(zero_count < buffer.len(), "Buffer should contain random data");
    }

    #[test]
    fn test_generate_u64() {
        let mut rng = SecureRandom::new().unwrap();
        let value1 = rng.generate_u64().unwrap();
        let value2 = rng.generate_u64().unwrap();
        assert_ne!(value1, value2, "Consecutive random numbers should differ");
    }
}