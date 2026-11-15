use serde_json::{Value, json};
use std::fs::{metadata, read_to_string};
use std::path::PathBuf;

use constants::operations::settings::*;
use errors::file::FileOperationError;
use files::general;
use properties;
use types::internal::store::GameStore;

fn get_path() -> Result<String, FileOperationError> {
    let path_buf: PathBuf = [properties::get_config_path(), SETTINGS_FILENAME.to_string()]
        .iter()
        .collect();
    let config_path = path_buf.display().to_string();
    let path_str = general::get_path(&config_path)?; //Creates file if it does not exist already
    match metadata(&path_str) {
        Ok(md) => {
            if md.len() == 0 {
                let settings = json!({
                    SELECTED_STORES.to_string(): [],
                    ALIASES_ENABLED.to_string(): 1,
                    ALLOW_ALIAS_REUSE_AFTER_CREATION.to_string(): 0
                });
                match serde_json::to_string_pretty(&settings) {
                    Ok(settings_str) => {
                        general::write_to_file(config_path.to_string(), settings_str)?
                    }
                    Err(e) => {
                        return Err(FileOperationError::Message(format!(
                            "Initial settings could not be created. {}",
                            e
                        )));
                    }
                }
            }
        }
        Err(e) => return Err(FileOperationError::Message(format!("{}", e))),
    }
    Ok(path_str)
}

pub fn load_data() -> Result<Value, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?;
    let body: Value = serde_json::from_str(&data)?;
    Ok(body)
}

pub fn get_available_stores() -> Vec<GameStore> {
    let available_stores = vec![
        GameStore::STEAM,
        GameStore::GOOD_OLD_GAMES,
        GameStore::MICROSOFT_STORE_PC,
    ];
    available_stores
}

pub fn get_selected_stores() -> Result<Vec<GameStore>, FileOperationError> {
    let filepath = get_path()?;
    let mut stores: Vec<GameStore> = Vec::new();
    // TODO: Propagate file and JSON parsing errors instead of panicking
    let data = read_to_string(filepath)?;
    let body: Value =
        serde_json::from_str(&data).expect("Get selected stores - could not convert to JSON");
    let selected = serde_json::to_string(&body[SELECTED_STORES.to_string()]).unwrap();
    match serde_json::from_str::<Vec<String>>(&selected) {
        Ok(data) => {
            for id in data.iter() {
                if id == GameStore::STEAM.get_id() {
                    stores.push(GameStore::STEAM);
                } else if id == GameStore::GOOD_OLD_GAMES.get_id() {
                    stores.push(GameStore::GOOD_OLD_GAMES);
                } else if id == GameStore::MICROSOFT_STORE_PC.get_id() {
                    stores.push(GameStore::MICROSOFT_STORE_PC)
                }
            }
        }
        Err(e) => return Err(FileOperationError::Message(format!("Error: {}", e))),
    };
    Ok(stores)
}

pub fn get_alias_state() -> Result<bool, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?;
    let body: Value = serde_json::from_str(&data)?; //.expect("Get alias state - could not convert to JSON");
    let alias_enabled = serde_json::to_string(&body[ALIASES_ENABLED.to_string()]).unwrap();
    Ok(serde_json::from_str::<i32>(&alias_enabled)? == 1) // {
}

pub fn get_alias_reuse_state() -> Result<bool, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?;
    let body: Value = serde_json::from_str(&data)?; //.expect("Get alias dup state - could not convert to JSON");
    let allow_dups =
        serde_json::to_string(&body[ALLOW_ALIAS_REUSE_AFTER_CREATION.to_string()]).unwrap();
    Ok(serde_json::from_str::<i32>(&allow_dups)? == 1) // {
}

pub fn update_selected_stores(selected: Vec<GameStore>) -> Result<(), FileOperationError> {
    let data = load_data()?;
    let mut settings = data;
    // TODO: Propagate errors when required settings keys are missing
    let selected_stores = settings.get_mut(SELECTED_STORES.to_string()).unwrap();
    let mut unique_stores: Vec<String> =
        selected.iter().map(|store| store.get_id().into()).collect();
    unique_stores.dedup();
    *selected_stores = json!(unique_stores);
    let settings_str = serde_json::to_string_pretty(&settings)?;
    general::write_to_file(
        get_path()?,
        settings_str, //.expect("Cannot update store search settings"),
    )?;
    Ok(())
}

pub fn update_alias_state(is_enabled: i32) -> Result<(), FileOperationError> {
    let data = load_data()?;
    let mut settings = data;
    let enabled_status = if is_enabled == ENABLED_STATE || is_enabled == DISABLED_STATE {
        is_enabled
    } else {
        DISABLED_STATE
    };
    // TODO: Propagate errors when required settings keys are missing
    *settings.get_mut(ALIASES_ENABLED.to_string()).unwrap() = json!(enabled_status);
    let settings_str = serde_json::to_string_pretty(&settings)?;
    general::write_to_file(
        get_path()?,
        settings_str, //.expect("Cannot set state of aliases"),
    )?;
    Ok(())
}

pub fn update_alias_reuse_state(is_enabled: i32) -> Result<(), FileOperationError> {
    match load_data() {
        Ok(data) => {
            let mut settings = data;
            let enabled_status = if is_enabled == ENABLED_STATE || is_enabled == DISABLED_STATE {
                is_enabled
            } else {
                DISABLED_STATE
            };
            // TODO: Propagate errors when required settings keys are missing
            *settings
                .get_mut(ALLOW_ALIAS_REUSE_AFTER_CREATION.to_string())
                .unwrap() = json!(enabled_status);
            let settings_str = serde_json::to_string_pretty(&settings);
            general::write_to_file(
                get_path()?,
                settings_str.expect("Cannot set state of alias duplicates"),
            )?;
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}

pub fn clear_selected_stores() -> Result<(), FileOperationError> {
    match load_data() {
        Ok(data) => {
            let mut settings = data;
            // TODO: Propagate errors when required settings keys are missing
            let selected_stores = settings.get_mut(SELECTED_STORES.to_string()).unwrap();
            *selected_stores = json!([]);
            let settings_str = serde_json::to_string_pretty(&settings);
            general::write_to_file(
                get_path()?,
                settings_str.expect("Cannot clear store search settings"),
            )?;
        }
        Err(e) => eprintln!("Error: {}", e),
    }
    Ok(())
}

pub fn list_selected_stores() -> Result<(), FileOperationError> {
    let available_stores = get_available_stores();
    let selected = get_selected_stores()?;
    println!("Selected Stores");
    for store in available_stores.iter() {
        let is_selected = selected.contains(store);
        if is_selected {
            println!("  [X] {}", store.get_name());
        } else {
            println!("  [ ] {}", store.get_name());
        }
    }
    Ok(())
}
