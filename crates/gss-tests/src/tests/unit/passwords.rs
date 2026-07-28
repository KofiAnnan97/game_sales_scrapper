use rand::distr::{Alphanumeric, SampleString};

use properties::passwords::Password;

static PLAINTEXT_PASSWORD : &str = "ASecretMessasge";
static KEY_STR : &str = "n54ltlcd7k81vefwsgxnihn5dlkjm2ri";
static ENCRYPTED_PASSWORD : &str = "MzEwYzA2NzU1OTBjMmIxYjFhNWQ1NmJhODA4MmE0NWZlYjgzNTA2MTM2ZTBlZjczNWExYjc5NmRmYTNjYjU2N2RhZjYwODBmNDAxZTRiZWFhNjMwZmU=";

#[test]
fn encryption_test(){
    let encrypted = Password::new(KEY_STR, PLAINTEXT_PASSWORD.to_string());
    assert_eq!(PLAINTEXT_PASSWORD, encrypted.get_value(Some(KEY_STR)), "Passwords was not encrypted correctly");
}

#[test]
fn decryption_test_successful(){
    let pass = Password::new_encrypted(ENCRYPTED_PASSWORD.to_string());
    let decrypt = pass.get_value(Some(KEY_STR));
    assert_eq!(PLAINTEXT_PASSWORD, decrypt, "Passwords was not decrypted correctly");
}

#[test]
#[should_panic]
fn decryption_test_fails(){
    let key_str = Alphanumeric.sample_string(&mut rand::rng(), 32);
    let pass: Password = Password::new_encrypted(ENCRYPTED_PASSWORD.to_string());
    assert_ne!(PLAINTEXT_PASSWORD, pass.get_value(Some(&key_str)), "This test should panic during decryption");
}
