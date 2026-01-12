use aes::cipher::{block_padding::Pkcs7, BlockDecryptMut, BlockEncryptMut, KeyIvInit};
use hex;
use rand::Rng;
use std::fs;
use std::io::{Read, Write};

type Aes256CbcEnc = cbc::Encryptor<aes::Aes256>;
type Aes256CbcDec = cbc::Decryptor<aes::Aes256>;

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut plaintext = Vec::new();
    file.read_to_end(&mut plaintext).map_err(|e| e.to_string())?;

    let salt: [u8; 16] = rand::thread_rng().gen();
    let key_iv = derive_key_iv(password.as_bytes(), &salt);

    let ciphertext = Aes256CbcEnc::new(&key_iv.0.into(), &key_iv.1.into())
        .encrypt_padded_vec_mut::<Pkcs7>(&plaintext);

    let mut output = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output.write_all(&salt).map_err(|e| e.to_string())?;
    output.write_all(&ciphertext).map_err(|e| e.to_string())?;

    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str) -> Result<(), String> {
    let mut file = fs::File::open(input_path).map_err(|e| e.to_string())?;
    let mut data = Vec::new();
    file.read_to_end(&mut data).map_err(|e| e.to_string())?;

    if data.len() < 16 {
        return Err("Invalid encrypted file".to_string());
    }

    let (salt, ciphertext) = data.split_at(16);
    let key_iv = derive_key_iv(password.as_bytes(), salt);

    let plaintext = Aes256CbcDec::new(&key_iv.0.into(), &key_iv.1.into())
        .decrypt_padded_vec_mut::<Pkcs7>(ciphertext)
        .map_err(|e| e.to_string())?;

    let mut output = fs::File::create(output_path).map_err(|e| e.to_string())?;
    output.write_all(&plaintext).map_err(|e| e.to_string())?;

    Ok(())
}

fn derive_key_iv(password: &[u8], salt: &[u8]) -> ([u8; 32], [u8; 16]) {
    let mut key = [0u8; 32];
    let mut iv = [0u8; 16];
    let mut combined = [0u8; 48];

    let mut hasher = blake3::Hasher::new();
    hasher.update(password);
    hasher.update(salt);
    let hash = hasher.finalize();

    combined.copy_from_slice(&hash.as_bytes()[..48]);
    key.copy_from_slice(&combined[..32]);
    iv.copy_from_slice(&combined[32..48]);

    (key, iv)
}

pub fn generate_random_key() -> String {
    let key: [u8; 32] = rand::thread_rng().gen();
    hex::encode(key)
}