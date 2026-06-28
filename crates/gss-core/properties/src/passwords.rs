use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use base64::prelude::*;

// Encryption and Decryption uses AES-256 GCM and Base64

pub fn encrypt(key_str: &str, plaintext: String) -> String{
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher_vec = cipher.encrypt(&nonce, plaintext.as_bytes()).expect("Failed to encrypt");
    let mut encrypted_vec: Vec<u8> = nonce.to_vec();
    encrypted_vec.extend_from_slice(&cipher_vec);
    let mut encrypted = hex::encode(encrypted_vec);
    encrypted = BASE64_STANDARD.encode(encrypted.as_bytes());
    encrypted
}

pub fn decrypt(key_str: &str, encrypted_text: String) -> String {
    if key_str == "" || key_str.len() != 32 { panic!("Decrypt key is empty or incorrectly formatted") }
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let base64_decode = BASE64_STANDARD.decode(encrypted_text.as_bytes()).expect("Failed to base64 decode");
    let decode_str = String::from_utf8(base64_decode).expect("Failed base64 decode");
    let encrypted_vec = hex::decode(decode_str).expect("Failed to decode hex string");
    let (nonce_arr, ciphered_vec) = encrypted_vec.split_at(12);
    let nonce = Nonce::from_slice(nonce_arr);
    let cipher = Aes256Gcm::new(key);
    let plaintext_vec = cipher.decrypt(nonce, ciphered_vec).expect("Failed to decrypt");
    let decrypted = String::from_utf8(plaintext_vec).expect("Failed to decode password");
    decrypted
}