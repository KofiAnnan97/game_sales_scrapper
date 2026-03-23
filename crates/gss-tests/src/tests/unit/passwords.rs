use rand::distr::{Alphanumeric, SampleString};

use properties::passwords;

static PLAINTEXT_PASSWORD : &str = "ASecretMessasge";
static KEY_STR : &str = "n54ltlcd7k81vefwsgxnihn5dlkjm2ri";
static ENCRYPTED_PASSWORD : &str = "MzEwYzA2NzU1OTBjMmIxYjFhNWQ1NmJhODA4MmE0NWZlYjgzNTA2MTM2ZTBlZjczNWExYjc5NmRmYTNjYjU2N2RhZjYwODBmNDAxZTRiZWFhNjMwZmU=";

#[test]
fn encryption_test(){
    let encrypted = passwords::encrypt(KEY_STR, PLAINTEXT_PASSWORD.to_string());
    assert_eq!(PLAINTEXT_PASSWORD, passwords::decrypt(KEY_STR, encrypted), "Passwords was not encrypted correctly");
}

#[test]
fn decryption_test_successful(){
    let decrypt = passwords::decrypt(KEY_STR, ENCRYPTED_PASSWORD.to_string());
    assert_eq!(PLAINTEXT_PASSWORD, decrypt, "Passwords was not decrypted correctly");
}

#[test]
#[should_panic]
fn decryption_test_fails(){
    let key_str = Alphanumeric.sample_string(&mut rand::rng(), 32);
    assert_ne!(PLAINTEXT_PASSWORD, passwords::decrypt(&key_str, ENCRYPTED_PASSWORD.to_string()), "This test should panic during decryption");
}
