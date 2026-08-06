use std::panic;
use std::env;
use std::path::{PathBuf};

use properties::env_vars::EnvVar;
use properties::{self, env_vars::{self, get_decrypt_key}};
use files::general;
use constants::operations::properties::*;
use crate::utils::{file_operations, tmp_setup};

const TMP_DIR_TITLE: &str = "env_vars";

fn delete_decrypt_key(){
    let path_buf : PathBuf = [CONFIG_DIR.to_string(), DECRYPT_FILENAME.to_string()].iter().collect();
    general::delete_file(path_buf.display().to_string());
}

#[test]
fn check_environment_variables() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let steam_api_key_val = "INITIAL";
    let recipient_email_val = "recipient@example.com";
    let smtp_host_val = "smtp.initial.com";
    let smtp_port_val: u16 = 587;
    let smtp_email_val = "user@initial.com";
    let smtp_username_val = "initial_user";
    let smtp_password_val = "initial_pwd";
    let env_data = file_operations::create_env_str(steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &_tmp_env.temp_dir);
    general::write_file(&_tmp_env.temp_dir, ENV_FILENAME, &env_data);
    
    unsafe {
        env::remove_var(STEAM_API_KEY_ENV);
        env::remove_var(RECIPIENT_EMAIL_ENV);
        env::remove_var(SMTP_HOST_ENV);
        env::remove_var(SMTP_PORT_ENV);
        env::remove_var(SMTP_EMAIL_ENV);
        env::remove_var(SMTP_USERNAME_ENV);
        env::remove_var(SMTP_PASSWORD_ENV);
        env::remove_var(PROJECT_PATH_ENV);
        env::remove_var(TEST_PATH_ENV);
    }
    env::set_current_dir(&_tmp_env.temp_dir).unwrap();

    let vars = env_vars::get_variables();
    println!("Test Environment Variables: {:?}", vars);
    if vars.is_empty(){ assert!(false, "No environment variables found."); }
    else {
        let api_key = vars.get(STEAM_API_KEY_ENV).and_then(EnvVar::as_password);
        let recipient = vars.get(RECIPIENT_EMAIL_ENV).and_then(EnvVar::as_str);
        let host = vars.get(SMTP_HOST_ENV).and_then(EnvVar::as_str);
        let port = vars.get(SMTP_PORT_ENV).and_then(EnvVar::as_u16);
        let email = vars.get(SMTP_EMAIL_ENV).and_then(EnvVar::as_str);
        let user = vars.get(SMTP_USERNAME_ENV).and_then(EnvVar::as_str);
        let pass = vars.get(SMTP_PASSWORD_ENV).and_then(EnvVar::as_password);
        let project_path = vars.get(PROJECT_PATH_ENV).and_then(EnvVar::as_str);
        let test_path = vars.get(TEST_PATH_ENV).and_then(EnvVar::as_str);

        let pwd_decode = panic::catch_unwind(|| {
            let decrypt_key = get_decrypt_key(properties::get_project_path());
            assert_eq!(steam_api_key_val, api_key.unwrap().get_value(Some(&decrypt_key)), "Environment variable should be {} not {}", STEAM_API_KEY_ENV, api_key.unwrap().get_value(Some(&decrypt_key)));
            assert_eq!(smtp_password_val, pass.unwrap().get_value(Some(&decrypt_key)), "Environment variable should be {} not {}", SMTP_PASSWORD_ENV, pass.unwrap().get_value(Some(&decrypt_key)));
        });
        match pwd_decode {
            Ok(_) => println!("Decryption successful"),
            Err(_) => delete_decrypt_key()
        }
        
        assert_eq!(recipient_email_val, recipient.unwrap(), "Environment variable should be {} not {}", recipient_email_val, recipient.unwrap());
        assert_eq!(smtp_host_val, host.unwrap(), "Environment variable should be {} not {}", smtp_host_val, host.unwrap());
        assert_eq!(smtp_port_val, port.unwrap(), "{} should be 0 not {}", smtp_port_val, port.unwrap());
        assert_eq!(smtp_email_val, email.unwrap(), "Environment variable should be {} not {}", smtp_email_val, email.unwrap());
        assert_eq!(smtp_username_val, user.unwrap(), "Environment variable should be {} not {}", smtp_username_val, user.unwrap());
        assert_eq!(_tmp_env.temp_dir.display().to_string(), project_path.unwrap(), "Environment variable should be {} not {}", _tmp_env.temp_dir.display().to_string(), project_path.unwrap());
        let expected_test_path = _tmp_env.temp_dir.join(DEFAULT_TEST_DIR);
        assert_eq!(expected_test_path.display().to_string(), test_path.unwrap(), "Environment variable should be {} not {}", expected_test_path.display().to_string(), test_path.unwrap());

        _tmp_env.tear_down();
    }
}

#[test]
fn no_variables(){
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    let steam_api_key_val = "";
    let recipient_email_val = "";
    let smtp_host_val = "";
    let smtp_port_val = 0;
    let smtp_email_val = "";
    let smtp_username_val = "";
    let smtp_password_val = "";
    let env_data = file_operations::create_env_str(steam_api_key_val, recipient_email_val, smtp_host_val, smtp_port_val, smtp_email_val, smtp_username_val, smtp_password_val, &_tmp_env.temp_dir);
    general::write_file(&_tmp_env.temp_dir, ENV_FILENAME, &env_data);

    unsafe {
        env::remove_var(STEAM_API_KEY_ENV);
        env::remove_var(RECIPIENT_EMAIL_ENV);
        env::remove_var(SMTP_HOST_ENV);
        env::remove_var(SMTP_PORT_ENV);
        env::remove_var(SMTP_EMAIL_ENV);
        env::remove_var(SMTP_USERNAME_ENV);
        env::remove_var(SMTP_PASSWORD_ENV);
        env::remove_var(PROJECT_PATH_ENV);
        env::remove_var(TEST_PATH_ENV);
    }
    env::set_current_dir(&_tmp_env.temp_dir).unwrap();

    let vars = env_vars::get_variables();
    println!("Test Environment Variables: {:?}", vars);

    let api_key = vars.get(STEAM_API_KEY_ENV).and_then(EnvVar::as_password);
        let recipient = vars.get(RECIPIENT_EMAIL_ENV).and_then(EnvVar::as_str);
        let host = vars.get(SMTP_HOST_ENV).and_then(EnvVar::as_str);
        let port = vars.get(SMTP_PORT_ENV).and_then(EnvVar::as_u16);
        let email = vars.get(SMTP_EMAIL_ENV).and_then(EnvVar::as_str);
        let user = vars.get(SMTP_USERNAME_ENV).and_then(EnvVar::as_str);
        let pass = vars.get(SMTP_PASSWORD_ENV).and_then(EnvVar::as_password);
        let project_path = vars.get(PROJECT_PATH_ENV).and_then(EnvVar::as_str);
        let test_path = vars.get(TEST_PATH_ENV).and_then(EnvVar::as_str);

        let pwd_decode = panic::catch_unwind(|| {
            let decrypt_key = get_decrypt_key(properties::get_project_path());
            assert_eq!(steam_api_key_val, api_key.unwrap().get_value(Some(&decrypt_key)), "Environment variable should be {} not {}", STEAM_API_KEY_ENV, api_key.unwrap().get_value(Some(&decrypt_key)));
            assert_eq!(smtp_password_val, pass.unwrap().get_value(Some(&decrypt_key)), "Environment variable should be {} not {}", SMTP_PASSWORD_ENV, pass.unwrap().get_value(Some(&decrypt_key)));
        });
        match pwd_decode {
            Ok(_) => println!("Decryption successful"),
            Err(_) => delete_decrypt_key()
        }

    assert_eq!(recipient_email_val, recipient.unwrap(), "Environment variable should be {} not {}", recipient_email_val, recipient.unwrap());
    assert_eq!(smtp_host_val, host.unwrap(), "Environment variable should be {} not {}", smtp_host_val, host.unwrap());
    assert_eq!(0, port.unwrap(), "{} should be 0 not {}", 0, port.unwrap());
    assert_eq!(smtp_email_val, email.unwrap(), "Environment variable should be {} not {}", smtp_email_val, email.unwrap());
    assert_eq!(smtp_username_val, user.unwrap(), "Environment variable should be {} not {}", smtp_username_val, user.unwrap());
    assert_eq!(_tmp_env.temp_dir.display().to_string(), project_path.unwrap(), "Environment variable should be {} not {}", _tmp_env.temp_dir.display().to_string(), project_path.unwrap());
    let expected_test_path = _tmp_env.temp_dir.join(DEFAULT_TEST_DIR);
    assert_eq!(expected_test_path.display().to_string(), test_path.unwrap(), "Environment variable should be {} not {}", expected_test_path.display().to_string(), test_path.unwrap());

    _tmp_env.tear_down();
}

#[test]
fn decrypt_key_created() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());

    let decrypt_file: PathBuf = [_tmp_env.temp_dir.display().to_string(), CONFIG_DIR.to_string(), DECRYPT_FILENAME.to_string()].iter().collect();

    assert!(!decrypt_file.exists(), "Decrypt key should not exist before creation");
    let initial_key = get_decrypt_key(_tmp_env.temp_dir.display().to_string());
    assert_eq!(32, initial_key.len(), "Decrypt key should be 32 characters long");
    assert!(decrypt_file.is_file(), "Decrypt key file should be created");

    let curr_key = get_decrypt_key(_tmp_env.temp_dir.display().to_string());
    assert_eq!(initial_key, curr_key, "Decrypt key should be reused from the same file");

    _tmp_env.tear_down();
}

#[test]
fn read_custom_env_file() {
    let _tmp_env = tmp_setup::setup_tmp_environment(TMP_DIR_TITLE, Vec::new());
    env::set_current_dir(&_tmp_env.temp_dir).unwrap();

    let env_filename = "custom.env";
    let env_var_name = "CUSTOM_VAR";
    let env_var_value = "CUSTOM_VALUE";
    let env_data = format!("{}=\"{}\"\n", env_var_name, env_var_value);
    general::write_file(&_tmp_env.temp_dir, env_filename, &env_data);

    env_vars::load_dotenv(Some(env_filename));
    assert_eq!(env::var(env_var_name).unwrap(), env_var_value, "Environment variable should be {} not {}", env_var_value, env::var(env_var_name).unwrap());

    _tmp_env.tear_down();
    unsafe {
        env::remove_var(env_var_name);
    }
}