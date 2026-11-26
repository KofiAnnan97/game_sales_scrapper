use serde_json::{Value, json};
use std::fs::{metadata, read_to_string};
use std::path::{Path, PathBuf};

use files::general;
pub mod env_vars;
pub mod passwords;
use crate::env_vars::{EnvVar, get_decrypt_key};
use crate::passwords::Password;
use constants::operations::logging::LOG_DIR;
use constants::operations::properties::{
    CONFIG_DIR, DATA_DIR, DEFAULT_TEST_DIR, ENV_FILENAME, PROJECT_PATH_ENV, PROP_PROJECT_PATH,
    PROP_RECIPIENT_EMAIL, PROP_SLIDING_STEAM_APPID, PROP_SMTP_EMAIL, PROP_SMTP_HOST,
    PROP_SMTP_PASSWORD, PROP_SMTP_PORT, PROP_SMTP_USERNAME, PROP_STEAM_API_KEY, PROP_TEST_MODE,
    PROP_TEST_PATH, PROPERTIES_FILENAME, RECIPIENT_EMAIL_ENV, SMTP_EMAIL_ENV, SMTP_HOST_ENV,
    SMTP_PASSWORD_ENV, SMTP_PORT_ENV, SMTP_USERNAME_ENV, STEAM_API_KEY_ENV, TEST_PATH_ENV,
};
use errors::input::FileOperationError;

// Retrieve paths
pub fn get_properties_path() -> Result<String, FileOperationError> {
    let mut project_path = env_vars::get_project_path();
    if project_path.is_empty() {
        project_path = std::env::current_dir()?.display().to_string();
    }
    let mut path_buf: PathBuf = [&project_path, CONFIG_DIR].iter().collect();
    let data_path = path_buf.display().to_string();
    general::create_dir(&data_path)?;
    path_buf = [data_path, PROPERTIES_FILENAME.to_string()]
        .iter()
        .collect();
    let properties_path = path_buf.display().to_string();
    let path_str = general::get_path(&properties_path)?;
    match metadata(&path_str) {
        Ok(md) => {
            if md.len() == 0 {
                let vars_result = env_vars::get_variables();
                let mut generic_test_path = String::new();
                match vars_result {
                    Ok(vars) => {
                        if vars.is_empty() {
                            let path_buf: PathBuf =
                                [&project_path, DEFAULT_TEST_DIR].iter().collect();
                            generic_test_path = path_buf.display().to_string();
                            general::create_dir(&generic_test_path)?;
                        }
                        let properties = json!({
                            PROP_STEAM_API_KEY : if let Some(EnvVar::Secret(steam_key)) = vars.get(STEAM_API_KEY_ENV) && let Ok(data) = steam_key.get_value(None){ data } else { String::new() },
                            PROP_RECIPIENT_EMAIL: if let Some(EnvVar::Text(recipient)) = vars.get(RECIPIENT_EMAIL_ENV) { &recipient } else { "" },
                            PROP_SMTP_HOST: if let Some(EnvVar::Text(host)) = vars.get(SMTP_HOST_ENV) { &host } else { "" },
                            PROP_SMTP_PORT: if let Some(EnvVar::Number(port)) = vars.get(SMTP_PORT_ENV) { *port } else { 0u16 },
                            PROP_SMTP_EMAIL: if let Some(EnvVar::Text(email)) = vars.get(SMTP_EMAIL_ENV) { &email } else { "" },
                            PROP_SMTP_USERNAME: if let Some(EnvVar::Text(user)) = vars.get(SMTP_USERNAME_ENV) { &user } else { "" },
                            PROP_SMTP_PASSWORD: if let Some(EnvVar::Secret(smtp_pwd)) = vars.get(SMTP_PASSWORD_ENV) && let Ok(data) = smtp_pwd.get_value(None) { data } else { String::new() },
                            PROP_PROJECT_PATH: &project_path,
                            PROP_TEST_PATH: if let Some(EnvVar::Text(test_path)) = vars.get(TEST_PATH_ENV) { &test_path} else { &generic_test_path },
                            PROP_SLIDING_STEAM_APPID: 0,
                            PROP_TEST_MODE: 0
                        });

                        match serde_json::to_string_pretty(&properties) {
                            Ok(properties_str) => {
                                general::write_to_file(
                                    properties_path.to_string(),
                                    properties_str,
                                )?;
                            }
                            Err(_) => println!("Initial properties could not be created."),
                        }
                    }
                    Err(e) => eprintln!("CONFIG_ERROR: {}", e),
                }
            }
        }
        Err(e) => return Err(FileOperationError::IO(e)),
    }
    Ok(path_str)
}

pub fn get_data_path() -> Result<String, FileOperationError> {
    let mut data_path = if is_testing_enabled()? {
        get_test_path()?
    } else {
        get_project_path()?
    };
    let path: PathBuf = [&data_path, DATA_DIR].iter().collect();
    data_path = path.display().to_string();
    general::create_dir(&data_path)?;
    Ok(data_path)
}

pub fn get_config_path() -> Result<String, FileOperationError> {
    let mut config_path = if is_testing_enabled()? {
        get_test_path()?
    } else {
        get_project_path()?
    };
    let path: PathBuf = [&config_path, CONFIG_DIR].iter().collect();
    config_path = path.display().to_string();
    general::create_dir(&config_path)?;
    Ok(config_path)
}

pub fn get_log_path() -> Result<String, FileOperationError> {
    let path: PathBuf = [&get_project_path()?, LOG_DIR].iter().collect();
    let log_path = path.display().to_string();
    general::create_dir(&log_path)?;
    Ok(log_path)
}

// Properties Functions

pub fn update_properties_from_env() -> Result<(), FileOperationError> {
    if let Err(e) = env_vars::load_dotenv(Some(ENV_FILENAME)) {
        return Err(FileOperationError::Message(format!(
            "Could not load {}: {}",
            ENV_FILENAME, e
        )));
    }

    let properties_path = get_properties_path()?;
    let prev_steam_key = get_steam_api_key(false)?;
    let prev_recipient = get_recipient()?;
    let prev_host = get_smtp_host()?;
    let prev_port = get_smtp_port()?;
    let prev_email = get_smtp_email()?;
    let prev_user = get_smtp_user()?;
    let prev_pwd = get_smtp_pwd(false)?;
    let prev_project_path = get_project_path()?;
    let prev_test_path = get_test_path()?;

    let vars = match env_vars::get_variables() {
        Ok(data) if !data.is_empty() => data,
        Err(e) => {
            return Err(FileOperationError::Message(format!(
                "Could not load environment variables: {}",
                e
            )));
        }
        _ => {
            return Err(FileOperationError::Message(format!(
                "Cannot find environment variables. Missing file: {}.",
                ENV_FILENAME
            )));
        }
    };

    if let Some((name, message)) = vars.iter().find_map(|(name, value)| match value {
        EnvVar::Invalid(message) => Some((name, message)),
        _ => None,
    }) {
        return Err(FileOperationError::Message(format!(
            "Invalid environment variable {}: {}",
            name, message
        )));
    }

    if !vars.is_empty() {
        let curr_steam_key = vars.get(STEAM_API_KEY_ENV).and_then(EnvVar::as_password);
        let curr_recipient = vars.get(RECIPIENT_EMAIL_ENV).and_then(EnvVar::as_str);
        let curr_host = vars.get(SMTP_HOST_ENV).and_then(EnvVar::as_str);
        let curr_port = vars.get(SMTP_PORT_ENV).and_then(EnvVar::as_u16);
        let curr_email = vars.get(SMTP_EMAIL_ENV).and_then(EnvVar::as_str);
        let curr_user = vars.get(SMTP_USERNAME_ENV).and_then(EnvVar::as_str);
        let curr_pwd = vars.get(SMTP_PASSWORD_ENV).and_then(EnvVar::as_password);
        let curr_project_path = vars.get(PROJECT_PATH_ENV).and_then(EnvVar::as_str);
        let curr_test_path = vars.get(TEST_PATH_ENV).and_then(EnvVar::as_str);

        let key_str = env_vars::get_decrypt_key(env_vars::get_project_path())?;
        let steam_api_key_updated = if let Some(key) = curr_steam_key
            && let Ok(data) = key.get_value(Some(&key_str))
        {
            prev_steam_key != data
        } else {
            false
        };
        let smtp_pwd_updated = if let Some(key) = curr_pwd
            && let Ok(data) = key.get_value(Some(&key_str))
        {
            prev_pwd != data
        } else {
            false
        };

        let properties_changed = steam_api_key_updated
            || smtp_pwd_updated
            || (curr_recipient.is_some() && prev_recipient != curr_recipient.unwrap())
            || (curr_host.is_some() && prev_host != curr_host.unwrap())
            || (curr_port.is_some() && prev_port != curr_port.unwrap())
            || (curr_email.is_some() && prev_email != curr_email.unwrap())
            || (curr_user.is_some() && prev_user != curr_user.unwrap())
            || (curr_project_path.is_some()
                && Path::new(curr_project_path.unwrap()).is_dir()
                && prev_project_path != curr_project_path.unwrap())
            || (curr_test_path.is_some()
                && Path::new(curr_test_path.unwrap()).is_dir()
                && prev_test_path != curr_test_path.unwrap());

        if properties_changed {
            let properties = json!({
                PROP_STEAM_API_KEY : if let Some(key) = curr_steam_key {
                    match key.get_value(None) {
                        Ok(pass)
                            if steam_api_key_updated => pass,
                        _ => get_string_var(PROP_STEAM_API_KEY)?,
                    }
                } else {
                    get_string_var(PROP_STEAM_API_KEY)?
                },
                PROP_RECIPIENT_EMAIL: curr_recipient.unwrap_or(&prev_recipient),
                PROP_SMTP_HOST: curr_host.unwrap_or(&prev_host),
                PROP_SMTP_PORT: curr_port.unwrap_or(prev_port),
                PROP_SMTP_EMAIL: curr_email.unwrap_or(&prev_email),
                PROP_SMTP_USERNAME: curr_user.unwrap_or(&prev_user),
                PROP_SMTP_PASSWORD: if let Some(pass) = curr_pwd {
                    match pass.get_value(None) {
                        Ok(data)
                            if smtp_pwd_updated => data,
                        _ => get_string_var(PROP_SMTP_PASSWORD)?,
                    }
                } else {
                    get_string_var(PROP_SMTP_PASSWORD)?
                },
                PROP_PROJECT_PATH: if let Some(path) = curr_project_path && Path::new(path).is_dir(){
                    path
                } else {
                    &prev_project_path
                },
                PROP_TEST_PATH: if let Some(path) = curr_test_path && Path::new(path).is_dir() {
                    path
                } else {
                    &prev_test_path
                },
                PROP_SLIDING_STEAM_APPID: get_sliding_steam_appid()?,
                PROP_TEST_MODE: get_test_mode()?,
            });
            match serde_json::to_string_pretty(&properties) {
                Ok(properties_str) => {
                    if let Err(e) = general::write_to_file(properties_path, properties_str) {
                        return Err(FileOperationError::Message(format!(
                            "Properties could not be set using env file: {}",
                            e
                        )));
                    }
                }
                Err(_) => {
                    return Err(FileOperationError::Message(
                        "Properties could not be set using env file.".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}

pub fn load_properties() -> Result<Value, FileOperationError> {
    let properties_path = get_properties_path()?;
    let data = read_to_string(properties_path)?;
    let body: Value = serde_json::from_str(&data)?;
    Ok(body)
}

// Getters

fn get_secret_var(var_name: &str) -> Result<Password, FileOperationError> {
    match load_properties() {
        Ok(properties) => match properties.get(var_name) {
            Some(result) => match serde_json::to_string(result) {
                Ok(val) => {
                    let trimmed_val = &val[1..val.len() - 1];
                    Ok(Password::new_encrypted(trimmed_val.to_string()))
                }
                Err(e) => Err(FileOperationError::Message(format!(
                    "Could not extract string from property, \"{}\", {}",
                    var_name, e
                ))),
            },
            None => Err(FileOperationError::Message(format!(
                "Warning: Property \"{}\" is empty/does not exist.",
                var_name
            ))),
        },
        Err(_) => Err(FileOperationError::Message(
            "Failed to load secret from properties file".into(),
        )),
    }
}

fn get_string_var(var_name: &str) -> Result<String, FileOperationError> {
    match load_properties() {
        Ok(properties) => match properties.get(var_name) {
            Some(result) => match result.as_str() {
                Some(str) => Ok(str.to_string()),
                None => Err(FileOperationError::Message(format!(
                    "'{}' did not return a string value",
                    var_name
                ))),
            },
            None => Err(FileOperationError::Message(format!(
                "Warning: Property \"{}\" is empty/does not exist.",
                var_name
            ))),
        },
        Err(_) => Err(FileOperationError::Message(format!(
            "Failed to load '{}' from properties file.",
            var_name
        ))),
    }
}

fn get_integer_var(var_name: &str) -> Result<i64, FileOperationError> {
    //let default_int: i64 = 0;
    match load_properties() {
        Ok(properties) => match properties.get(var_name) {
            Some(var) => match var.as_i64() {
                Some(digits) => Ok(digits),
                None => Err(FileOperationError::Message(format!(
                    "'{}' did not return an integer value",
                    var_name
                ))),
            },
            None => Err(FileOperationError::Message(format!(
                "Failed to parse \"{}\". Please check that this is an integer.",
                var_name
            ))), // default_int
        },
        Err(_) => Err(FileOperationError::Message(format!(
            "Failed to load '{}' from properties file.",
            var_name
        ))),
    }
}

pub fn get_steam_api_key(hidden: bool) -> Result<String, FileOperationError> {
    let steam_api_key = get_secret_var(PROP_STEAM_API_KEY);

    if let Ok(mut key) = steam_api_key {
        if hidden {
            key.set_state(passwords::PasswordState::Hidden);
        } else {
            key.set_state(passwords::PasswordState::PlainText);
        }

        let key_str = get_decrypt_key(get_project_path()?)?;
        key.get_value(Some(&key_str))
            .map_err(|e| FileOperationError::Message(format!("Could not retrieve secrete. {}", e)))
    } else {
        Ok(String::new())
    }
}

pub fn get_recipient() -> Result<String, FileOperationError> {
    get_string_var(PROP_RECIPIENT_EMAIL)
}

pub fn get_smtp_email() -> Result<String, FileOperationError> {
    get_string_var(PROP_SMTP_EMAIL)
}

pub fn get_smtp_host() -> Result<String, FileOperationError> {
    get_string_var(PROP_SMTP_HOST)
}

pub fn get_smtp_port() -> Result<u16, FileOperationError> {
    let val = get_integer_var(PROP_SMTP_PORT)?;
    Ok(val as u16)
}

pub fn get_smtp_user() -> Result<String, FileOperationError> {
    get_string_var(PROP_SMTP_USERNAME)
}

pub fn get_smtp_pwd(hidden: bool) -> Result<String, FileOperationError> {
    let smtp_pwd = get_secret_var(PROP_SMTP_PASSWORD);
    if let Ok(mut key) = smtp_pwd {
        if hidden {
            key.set_state(passwords::PasswordState::Hidden);
        } else {
            key.set_state(passwords::PasswordState::PlainText);
        }
        let key_str = get_decrypt_key(get_project_path()?)?;
        key.get_value(Some(&key_str))
            .map_err(|e| FileOperationError::Message(format!("Could not retrieve secret. {}", e)))
    } else {
        Ok(String::new())
    }
}

pub fn get_project_path() -> Result<String, FileOperationError> {
    let mut project_path = get_string_var(PROP_PROJECT_PATH)?;
    if project_path.is_empty() {
        env_vars::get_project_path();
    }
    if !Path::new(&project_path).is_dir() {
        if !project_path.is_empty() {
            eprintln!(
                "Directory does not exist: '{}'. Project path set to current working directory.",
                project_path
            );
        }
        project_path = std::env::current_dir()?.display().to_string();
        set_project_path(&project_path)?;
    }
    Ok(project_path)
}

pub fn get_test_path() -> Result<String, FileOperationError> {
    let mut test_path = get_string_var(PROP_TEST_PATH)?;
    if test_path.is_empty() {
        test_path = env_vars::get_test_path();
    }
    if !Path::new(&test_path).is_dir() {
        if !test_path.is_empty() {
            eprintln!(
                "Directory does not exist: '{}'. Default test path set.",
                test_path
            );
        }
        let path_buf: PathBuf = [&get_project_path()?, DEFAULT_TEST_DIR].iter().collect();
        test_path = path_buf.display().to_string();
        general::create_dir(&test_path)?;
        set_test_path(&test_path)?;
    }
    Ok(test_path)
}

pub fn get_sliding_steam_appid() -> Result<u32, FileOperationError> {
    let steam_appid = get_integer_var(PROP_SLIDING_STEAM_APPID)?;
    if steam_appid > 0 {
        Ok(steam_appid as u32)
    } else {
        Ok(0)
    }
}

pub fn get_test_mode() -> Result<i16, FileOperationError> {
    Ok(get_integer_var(PROP_TEST_MODE)? as i16)
}

pub fn is_testing_enabled() -> Result<bool, FileOperationError> {
    let state = get_integer_var(PROP_TEST_MODE)?;
    Ok(state == 1)
}

// Setters

pub fn set_steam_api_key(api_key: String) -> Result<(), FileOperationError> {
    let mut properties = load_properties()?;
    match Password::new(&get_decrypt_key(get_project_path()?)?, api_key) {
        Ok(steam_key) if let Ok(data) = steam_key.get_value(None) => {
            *properties
                .get_mut(PROP_STEAM_API_KEY)
                .ok_or(FileOperationError::Message(format!(
                    "Missing '{}' in properties",
                    PROP_STEAM_API_KEY
                )))? = json!(&data);
            match serde_json::to_string_pretty(&properties) {
                Ok(properties_str) => {
                    if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                        return Err(FileOperationError::Message(format!(
                            "Property steam api key could not be set: {}",
                            e
                        )));
                    }
                }
                Err(_) => {
                    return Err(FileOperationError::Message(
                        "Property steam api key could not be set.".into(),
                    ));
                }
            }
        }
        _ => (),
    }
    Ok(())
}

pub fn set_recipient(email_str: &str) -> Result<(), FileOperationError> {
    let mut properties = load_properties()?;
    *properties
        .get_mut(PROP_RECIPIENT_EMAIL)
        .ok_or(FileOperationError::Message(format!(
            "Missing '{}' in properties",
            PROP_RECIPIENT_EMAIL
        )))? = json!(email_str);
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Property set recipient could not be set: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Property set recipient could not be set.".into(),
            ));
        }
    }
    Ok(())
}

pub fn set_stmp_vars(
    host: String,
    port: u16,
    email: String,
    user: String,
    pass: String,
) -> Result<(), FileOperationError> {
    let mut properties = load_properties()?;
    if !host.is_empty() {
        *properties
            .get_mut(PROP_SMTP_HOST)
            .ok_or(FileOperationError::Message(format!(
                "Missing '{}' in properties",
                PROP_SMTP_HOST
            )))? = json!(host);
    }
    if port != 0 {
        *properties
            .get_mut(PROP_SMTP_PORT)
            .ok_or(FileOperationError::Message(format!(
                "Missing '{}' in properties",
                PROP_SMTP_PORT
            )))? = json!(port);
    }
    if !email.is_empty() {
        *properties
            .get_mut(PROP_SMTP_EMAIL)
            .ok_or(FileOperationError::Message(format!(
                "Missing '{}' in properties",
                PROP_SMTP_EMAIL
            )))? = json!(email);
    }
    if !user.is_empty() {
        *properties
            .get_mut(PROP_SMTP_USERNAME)
            .ok_or(FileOperationError::Message(format!(
                "Missing '{}' in properties",
                PROP_SMTP_USERNAME
            )))? = json!(user);
    }
    if !pass.is_empty() {
        match Password::new(&get_decrypt_key(get_project_path()?)?, pass) {
            Ok(val) if let Ok(data) = val.get_value(None) => {
                *properties
                    .get_mut(PROP_SMTP_PASSWORD)
                    .ok_or(FileOperationError::Message(format!(
                        "Missing '{}' in properties",
                        PROP_SMTP_PASSWORD
                    )))? = json!(&data)
            }
            _ => (),
        }
    }
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Smtp properties could not be set: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Smtp properties could not be set.".into(),
            ));
        }
    }
    Ok(())
}

pub fn set_project_path(path: &str) -> Result<(), FileOperationError> {
    if !Path::new(&path).is_dir() {
        return Err(FileOperationError::Message(format!(
            "Project path was not set because '{}' is not a directory.",
            path
        )));
    }
    let mut properties = load_properties()?;
    *properties
        .get_mut(PROP_PROJECT_PATH)
        .ok_or(FileOperationError::Message(format!(
            "Missing '{}' in properties",
            PROP_PROJECT_PATH
        )))? = json!(path);
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Project path property could not be set: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Project path property could not be set.".into(),
            ));
        }
    }
    Ok(())
}

pub fn set_test_path(path: &str) -> Result<(), FileOperationError> {
    if !Path::new(&path).is_dir() {
        return Err(FileOperationError::Message(format!(
            "Test path was not set because '{}' is not a directory.",
            path
        )));
    }
    let mut properties = load_properties()?;
    *properties
        .get_mut(PROP_TEST_PATH)
        .ok_or(FileOperationError::Message(format!(
            "Missing '{}' in properties",
            PROP_TEST_PATH
        )))? = json!(path);
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Test path property could not be created/updated: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Test path property could not be created/updated.".into(),
            ));
        }
    }
    Ok(())
}

pub fn set_sliding_steam_appid(steam_appid: u32) -> Result<(), FileOperationError> {
    let mut properties = load_properties()?;
    *properties
        .get_mut(PROP_SLIDING_STEAM_APPID)
        .ok_or(FileOperationError::Message(format!(
            "Missing '{}' in properties",
            PROP_SLIDING_STEAM_APPID
        )))? = json!(steam_appid);
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Sliding steam appid property could not be set: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Sliding steam appid property could not be set.".into(),
            ));
        }
    }
    Ok(())
}

pub fn set_test_mode(is_enabled: bool) -> Result<(), FileOperationError> {
    let mut properties = load_properties()?;
    let enabled = if is_enabled { 1 } else { 0 };
    *properties
        .get_mut(PROP_TEST_MODE)
        .ok_or(FileOperationError::Message(format!(
            "Missing '{}' in properties",
            PROP_TEST_MODE
        )))? = json!(enabled);
    match serde_json::to_string_pretty(&properties) {
        Ok(properties_str) => {
            if let Err(e) = general::write_to_file(get_properties_path()?, properties_str) {
                return Err(FileOperationError::Message(format!(
                    "Test mode property could not be set: {}",
                    e
                )));
            }
        }
        Err(_) => {
            return Err(FileOperationError::Message(
                "Test mode property could not be set.".into(),
            ));
        }
    }
    Ok(())
}
