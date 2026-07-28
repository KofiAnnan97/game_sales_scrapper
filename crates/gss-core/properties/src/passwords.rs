use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use aes_gcm::aead::{Aead, OsRng};
use base64::prelude::*;
use core::panic;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordState {
    Hidden,
    Encrypted,
    PlainText,
}

#[derive(Debug)]
pub struct Password{
    value: String,
    state: PasswordState,
}

impl Password{
    pub fn new(key_str: &str, pwd: String) -> Self{
        Password { value: encrypt(key_str, pwd), state: PasswordState::Encrypted }
    }

    pub fn new_encrypted(pwd_encrypted: String) -> Self {
        Password { value: pwd_encrypted, state: PasswordState::Encrypted }
    }

    pub fn get_state(&self) -> PasswordState {
        self.state
    }

    pub fn set_state(&mut self, new_state: PasswordState) {
        self.state = new_state;
    }

    pub fn get_value(&self, key_str: Option<&str>) -> String {
        match self.state {
            PasswordState::Hidden => "\u{00B7}".repeat(16),
            PasswordState::Encrypted => {
                if key_str.is_some() {
                    decrypt(key_str.unwrap(), self.value.to_owned())
                } else {
                    self.value.to_owned()
                }
            },
            PasswordState::PlainText => {
                if key_str.is_some() {
                    decrypt(key_str.unwrap(), self.value.to_owned())
                } else {
                    String::new()
                }
            }
        }
    }
}

// Encryption and Decryption uses AES-256 GCM and Base64
fn encrypt(key_str: &str, plaintext: String) -> String{
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

fn decrypt(key_str: &str, encrypted_text: String) -> String {
    if encrypted_text.is_empty() { return String::new() }
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