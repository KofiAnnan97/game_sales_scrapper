use aes_gcm::aead::{Aead, OsRng};
use aes_gcm::{AeadCore, Aes256Gcm, Key, KeyInit, Nonce};
use base64::prelude::*;
use errors::secrets::{DecryptError, SecretsError};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PasswordState {
    Hidden,
    Encrypted,
    PlainText,
}

#[derive(Debug)]
pub struct Password {
    value: String,
    state: PasswordState,
}

impl Password {
    pub fn new(key_str: &str, pwd: String) -> Result<Self, SecretsError> {
        let password = Password {
            value: encrypt(key_str, pwd)?,
            state: PasswordState::Encrypted,
        };
        Ok(password)
    }

    pub fn new_encrypted(pwd_encrypted: String) -> Self {
        Password {
            value: pwd_encrypted,
            state: PasswordState::Encrypted,
        }
    }

    pub fn get_state(&self) -> PasswordState {
        self.state
    }

    pub fn set_state(&mut self, new_state: PasswordState) {
        self.state = new_state;
    }

    pub fn get_value(&self, key_str: Option<&str>) -> Result<String, SecretsError> {
        match self.state {
            PasswordState::Hidden => Ok("\u{2219}".repeat(24)),
            PasswordState::Encrypted => {
                if let Some(key) = key_str {
                    Ok(decrypt(key, self.value.to_owned())?)
                } else {
                    Ok(self.value.to_owned())
                }
            }
            PasswordState::PlainText => {
                if let Some(key) = key_str {
                    Ok(decrypt(key, self.value.to_owned())?)
                } else {
                    Err(SecretsError::Message("Key could not be decrypted.".into()))
                }
            }
        }
    }
}

// Encryption and Decryption uses AES-256 GCM and Base64
fn encrypt(key_str: &str, plaintext: String) -> Result<String, SecretsError> {
    if key_str.len() != 32 {
        return Err(SecretsError::Message(
            "Encryption key must be exactly 32 bytes".into(),
        ));
    }
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let cipher = Aes256Gcm::new(key);
    let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
    let cipher_vec = cipher.encrypt(&nonce, plaintext.as_bytes())?;
    let mut encrypted_vec: Vec<u8> = nonce.to_vec();
    encrypted_vec.extend_from_slice(&cipher_vec);
    let mut encrypted = hex::encode(encrypted_vec);
    encrypted = BASE64_STANDARD.encode(encrypted.as_bytes());
    Ok(encrypted)
}

fn decrypt(key_str: &str, encrypted_text: String) -> Result<String, DecryptError> {
    if encrypted_text.is_empty() {
        return Err(DecryptError::Message("Encrypted key is empty".into()));
    }
    if key_str.is_empty() || key_str.len() != 32 {
        return Err(DecryptError::Message(
            "Decrypt key must not be empty and exactly 32 bytes".into(),
        ));
    }
    let key = Key::<Aes256Gcm>::from_slice(key_str.as_bytes());
    let base64_decode = BASE64_STANDARD.decode(encrypted_text.as_bytes())?;
    let decode_str = String::from_utf8(base64_decode)?;
    let encrypted_vec = hex::decode(decode_str)
        .map_err(|err| DecryptError::Message(format!("Invalid encrypted key format: {err}")))?;
    let (nonce_arr, ciphered_vec) = encrypted_vec.split_at(12);
    let nonce = Nonce::from_slice(nonce_arr);
    let cipher = Aes256Gcm::new(key);
    let plaintext_vec = cipher.decrypt(nonce, ciphered_vec)?;
    let key = String::from_utf8(plaintext_vec)?;
    Ok(key)
}
