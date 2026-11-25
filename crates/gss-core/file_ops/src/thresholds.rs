use serde_json::{Value, json};
use std::collections::HashMap;
use std::fs::{metadata, read_to_string};
use std::io::{self, Write};
use std::path::PathBuf;

use crate::settings;
use constants::operations::thresholds::*;
use errors::GenericError;
use errors::config::ConfigError;
use errors::input::FileOperationError;
use files::general;
use stores::pc::steam; //, gog, microsoft_store};
use stores::search::fuzzy;
use types::internal::{data::GameThreshold, store::GameStore};
use types::response::{gog::GameInfo as GOGGameInfo, microsoft_store::ProductInfo, steam::App};

use properties;

pub fn get_path() -> Result<String, FileOperationError> {
    let path_buf: PathBuf = [properties::get_data_path()?, THRESHOLD_FILENAME.to_string()]
        .iter()
        .collect();
    let thresh_path = path_buf.display().to_string();
    let path_str = general::get_path(&thresh_path)?;
    match metadata(&path_str) {
        Ok(md) => {
            if md.len() == 0 {
                let data = json!({
                    THRESHOLDS.to_string(): [],
                    ALIAS_MAP.to_string(): {},
                });
                match serde_json::to_string_pretty(&data) {
                    Ok(data_str) => general::write_to_file(thresh_path.to_string(), data_str)?,
                    Err(e) => return Err(FileOperationError::Message(format!("{}", e))),
                };
            }
        }
        Err(e) => return Err(FileOperationError::Message(format!("Error: {}", e))),
    }
    Ok(path_str)
}

pub fn load_data() -> Result<Value, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?;
    match serde_json::from_str(&data) {
        Ok(data) => Ok(data),
        Err(e) => Err(e.into()),
    }
}

pub fn load_thresholds() -> Result<Vec<GameThreshold>, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?; //.unwrap();
    let body: Value = serde_json::from_str(&data)?; //.expect("Load thresholds - could not convert to JSON");
    let thresholds = serde_json::to_string(&body[THRESHOLDS])?;
    match serde_json::from_str::<Vec<GameThreshold>>(&thresholds) {
        Ok(thresholds) => Ok(thresholds),
        Err(e) => Err(e.into()),
    }
}

pub fn load_alias_map() -> Result<HashMap<String, Vec<String>>, FileOperationError> {
    let filepath = get_path()?;
    let data = read_to_string(filepath)?;
    let body: Value = serde_json::from_str(&data)?; //.expect("Load alias map - could not convert to JSON");
    let alias_map = serde_json::to_string(&body[ALIAS_MAP])?;
    match serde_json::from_str::<HashMap<String, Vec<String>>>(&alias_map) {
        Ok(map) => Ok(map),
        Err(e) => Err(e.into()),
    }
}

pub fn update_thresholds(thresholds: Vec<GameThreshold>) -> Result<(), FileOperationError> {
    match load_data() {
        Ok(data) => {
            let mut thresholds_data = data;
            *thresholds_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            match serde_json::to_string_pretty(&thresholds_data) {
                Ok(thresholds_str) => general::write_to_file(get_path()?, thresholds_str)?,
                Err(e) => {
                    return Err(FileOperationError::Message(format!(
                        "Cannot update thresholds. {}",
                        e
                    )));
                }
            }
        }
        Err(e) => return Err(FileOperationError::Message(format!("Error: {}", e))),
    }
    Ok(())
}

pub fn update_thresholds_and_alias_map(
    mut thresholds: Vec<GameThreshold>,
    game_title: &str,
    alias: &str,
) -> Result<(), FileOperationError> {
    let mut alias_map = load_alias_map().unwrap_or_default();

    // Remove old alias for game in alias map
    let old_alias;
    if let Some(i) = thresholds
        .iter()
        .position(|threshold| threshold.title == game_title)
    {
        old_alias = thresholds[i].alias.to_string();
        thresholds[i].alias = String::from(alias);
    } else {
        return Ok(());
    }

    if !old_alias.is_empty() && alias_map.contains_key(&old_alias) {
        let alias_list = alias_map.get_mut(&old_alias).unwrap();
        if let Some(j) = alias_list
            .iter()
            .position(|threshold_title| threshold_title == game_title)
        {
            alias_list.remove(j);
        }
        if alias_list.is_empty() {
            alias_map.remove(&old_alias);
        }
    }

    // Add new alias for game in alias map
    if !alias.is_empty() {
        if alias_map.contains_key(alias) {
            alias_map
                .get_mut(alias)
                .unwrap()
                .push(game_title.to_string());
        } else {
            alias_map.insert(alias.to_string(), vec![game_title.to_string()]);
        }
    }

    // Save changes to thresholds and alias map
    let mut thresholds_data = load_data()?;
    *thresholds_data.get_mut(ALIAS_MAP.to_string()).unwrap() = json!(alias_map);
    *thresholds_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
    match serde_json::to_string_pretty(&thresholds_data) {
        Ok(alias_map_str) => general::write_to_file(get_path()?, alias_map_str)?,
        Err(e) => {
            return Err(FileOperationError::Message(format!(
                "Cannot update thresholds and alias map. {}",
                e
            )));
        }
    }
    Ok(())
}

pub fn update_alias_map(alias_map: HashMap<String, Vec<String>>) -> Result<(), FileOperationError> {
    let data = load_data()?;
    let mut alias_data = data;
    *alias_data.get_mut(ALIAS_MAP.to_string()).unwrap() = json!(alias_map);
    match serde_json::to_string_pretty(&alias_data) {
        Ok(alias_str) => general::write_to_file(get_path()?, alias_str)?,
        Err(e) => {
            return Err(FileOperationError::Message(format!(
                "Cannot update alias map. {}",
                e
            )));
        }
    }
    Ok(())
}

pub fn update_threshold_alias(
    game_title: String,
    new_alias: &str,
) -> Result<bool, FileOperationError> {
    let mut alias_map = load_alias_map().unwrap_or_default();

    // Remove old alias for game in alias map
    let mut thresholds = load_thresholds().unwrap_or_default();
    let old_alias;
    if let Some(i) = thresholds
        .iter()
        .position(|threshold| *threshold.title == game_title)
    {
        old_alias = thresholds[i].alias.to_string();
        thresholds[i].alias = String::from(new_alias);
    } else {
        return Ok(false);
    }

    if !old_alias.is_empty() && alias_map.contains_key(&old_alias) {
        let alias_list = alias_map.get_mut(&old_alias).unwrap();
        if let Some(j) = alias_list
            .iter()
            .position(|threshold_title| *threshold_title == game_title)
        {
            alias_list.remove(j);
        }
        if alias_list.is_empty() {
            alias_map.remove(&old_alias);
        }
    }

    // Add new alias for game in alias map
    if !new_alias.is_empty() {
        if alias_map.contains_key(new_alias) {
            alias_map
                .get_mut(new_alias)
                .unwrap()
                .push(game_title.clone());
        } else {
            alias_map.insert(new_alias.to_string(), vec![game_title.clone()]);
        }
    }

    // Save changes to thresholds and alias map
    match load_data() {
        Ok(data) => {
            let mut alias_data = data;
            *alias_data.get_mut(ALIAS_MAP.to_string()).unwrap() = json!(alias_map);
            *alias_data.get_mut(THRESHOLDS.to_string()).unwrap() = json!(thresholds);
            match serde_json::to_string_pretty(&alias_data) {
                Ok(alias_map_str) => general::write_to_file(get_path()?, alias_map_str)?,
                Err(e) => {
                    return Err(FileOperationError::Message(format!(
                        "Cannot update alias map. {}",
                        e
                    )));
                }
            }
            // println!("{} threshold alias set to \'{}\'", game_title, new_alias);
            Ok(true)
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            Ok(false)
        }
    }
}

// A CLI-only function for updating a threshold using fuzzy algos
pub fn update_threshold_alias_fuzzy(
    game_title: String,
    new_alias: &str,
) -> Result<(), ConfigError> {
    // Try to run normal update alias first
    let alias_updated = update_threshold_alias(game_title.clone(), new_alias)?;

    let mut fuzzy_thresholds: Vec<String> = Vec::new();
    if !alias_updated {
        // Find other potential game thresholds
        let thresholds = load_thresholds().unwrap_or_default();
        for thresh in thresholds {
            let mut dist = fuzzy::levenshtein_distance(&game_title, thresh.title.as_str());
            if 1.0 - (dist / thresh.title.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                fuzzy_thresholds.push(thresh.title);
            } else if !thresh.alias.is_empty() {
                dist = fuzzy::levenshtein_distance(&game_title, thresh.alias.as_str());
                if 1.0 - (dist / thresh.alias.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                    fuzzy_thresholds.push(thresh.title);
                }
            }
        }

        if fuzzy_thresholds.is_empty() {
            return Ok(());
        }

        println!("Did you mean one of the following?");
        for (idx, game_title) in fuzzy_thresholds.iter().enumerate() {
            println!("  [{}] {}", idx, game_title);
        }
        println!("  [c] Cancel");
        let mut input = String::new();
        print!("Update alias by index or type \'c\' to cancel: ");
        let _ = io::stdout().flush();
        // TODO: Propagate terminal input errors instead of panicking
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "c" {
            println!("Threshold alias update of '{}' cancelled.", game_title);
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < fuzzy_thresholds.len() {
                        let fuzzy_title = fuzzy_thresholds[idx].clone();
                        let _ = update_threshold_alias(fuzzy_title, new_alias);
                    } else if idx >= fuzzy_thresholds.len() {
                        return Err(ConfigError::Message(format!(
                            "Integer \"{}\" is invalid. Remove cancelled.",
                            idx
                        )));
                    }
                }
                Err(e) => {
                    return Err(ConfigError::Message(format!(
                        "Invalid input: {}\nError: {}",
                        input, e
                    )));
                }
            }
        }
    }
    Ok(())
}

pub fn does_alias_exist(alias_name: &str) -> bool {
    let alias_map = load_alias_map().unwrap_or_default();
    alias_map.contains_key(alias_name)
}

fn is_threshold(title: &str, game_thresh: &GameThreshold) -> bool {
    title == game_thresh.title || title == game_thresh.alias
}

pub async fn add_steam_game(
    new_alias: String,
    app: App,
    price: f64,
    client: &reqwest::Client,
) -> Result<String, GenericError> {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    match steam::get_price(app.app_id, client).await {
        Ok(po) => {
            let mut unique: bool = true;
            for elem in thresholds.iter() {
                if is_threshold(&app.name, elem) {
                    unique = false;
                    if elem.steam_id == 0 {
                        update_id(&elem.title, GameStore::STEAM, app.app_id);
                    }
                    break;
                }
            }
            let alias_str = new_alias;
            if unique {
                thresholds.push(GameThreshold {
                    title: app.name.clone(),
                    alias: alias_str.clone(),
                    steam_id: app.app_id,
                    gog_id: 0,
                    microsoft_store_id: String::new(),
                    currency: po.currency,
                    desired_price: price,
                });
                update_thresholds_and_alias_map(thresholds, &app.name, &alias_str)?;
            } else if !alias_str.is_empty() {
                update_threshold_alias(app.name.clone(), &alias_str)?;
            }
            Ok(app.name)
        }
        Err(e) => return Err(e.into()),
    }
}

pub fn add_gog_game(
    new_alias: String,
    game: &GOGGameInfo,
    price: f64,
) -> Result<String, ConfigError> {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let mut unique: bool = true;
    for elem in thresholds.iter() {
        if is_threshold(&game.title, elem) {
            unique = false;
            if elem.gog_id == 0 {
                let game_id = game.id.parse::<u32>().unwrap();
                update_id(&elem.title, GameStore::GOOD_OLD_GAMES, game_id);
            }
            break;
        }
    }
    let alias_str = new_alias;
    if unique {
        let currency_code = match &game.price {
            Some(price_data) => price_data.base_money.currency.clone(),
            None => "USD".to_string(),
        };
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            alias: alias_str.clone(),
            steam_id: 0,
            gog_id: game.id.parse::<u32>().unwrap(),
            microsoft_store_id: String::new(),
            //currency: game.price.currency.clone(), // Version 1
            currency: currency_code,
            desired_price: price,
        });
        update_thresholds_and_alias_map(thresholds, &game.title, &alias_str)?;
    } else if !alias_str.is_empty() {
        update_threshold_alias(game.title.clone(), &alias_str)?;
    }
    Ok(game.title.clone())
}

pub fn add_microsoft_store_game(
    new_alias: String,
    game: &ProductInfo,
    price: f64,
) -> Result<String, ConfigError> {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let mut unique: bool = true;
    for elem in thresholds.iter() {
        if is_threshold(&game.title, elem) {
            unique = false;
            if elem.microsoft_store_id.is_empty() {
                let game_id = &game.product_id;
                update_id_str(&elem.title, GameStore::MICROSOFT_STORE_PC, game_id);
            }
            break;
        }
    }
    let alias_str = new_alias;
    if unique {
        thresholds.push(GameThreshold {
            title: game.title.clone(),
            alias: alias_str.clone(),
            steam_id: 0,
            gog_id: 0,
            microsoft_store_id: game.product_id.clone(),
            currency: String::from("USD"),
            desired_price: price,
        });
        update_thresholds_and_alias_map(thresholds, &game.title, &alias_str)?;
    } else if !alias_str.is_empty() {
        update_threshold_alias(game.title.clone(), &alias_str)?;
    }
    Ok(game.title.clone())
}

pub fn get_alias_if_available(
    alias: &str,
) -> Result<(String, bool), FileOperationError> {
    if alias.is_empty() || !does_alias_exist(alias) {
        return Ok((alias.to_string(), true));
    }

    if settings::get_alias_reuse_state()? {
        Ok((alias.to_string(), true))
    } else {
        Ok((String::new(), false))
    }
}

pub fn set_game_alias() -> Result<String, ConfigError> {
    let mut alias = String::new();
    match settings::get_alias_state() {
        Ok(_) => {
            let mut input = String::new();
            print!("Do you want to assign an alias [Y\\N]? ");
            let _ = io::stdout().flush();
            // TODO: Propagate terminal input errors instead of panicking
            io::stdin()
                .read_line(&mut input)
                .expect("Failed to permission to assign alias.");
            if input.trim() == "Yes" || input.trim() == "Y" || input.trim() == "YES" {
                print!("Alias name: ");
                let _ = io::stdout().flush();
                let mut alias_name = String::new();
                // TODO: Propagate terminal input errors instead of panicking
                io::stdin()
                    .read_line(&mut alias_name)
                    .expect("Failed to read alias.");
                alias = alias_name.trim().to_string();
            }
        }
        Err(e) => return Err(e.into()),
    }
    Ok(alias)
}

pub fn update_price(title: &str, price: f64) -> bool {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let mut price_updated = false;
    for threshold in thresholds.iter_mut() {
        if is_threshold(title, threshold) {
            if price != threshold.desired_price {
                let old_threshold = threshold.desired_price;
                threshold.desired_price = price;
                println!(
                    "\"{}\": updated price threshold from {} to {}",
                    threshold.title, old_threshold, threshold.desired_price
                );
                price_updated = true;
            } else {
                println!(
                    "Price was not updated because it is already set to {}",
                    price
                );
            }
        }
    }
    if price_updated {
        let _ = update_thresholds(thresholds);
        true
    } else {
        // println!("\"{}\" does not have a configured threshold.", title);
        false
    }
}

pub fn update_price_fuzzy(title: &str, price: f64) {
    //Try to run the normal update price first
    let price_updated = update_price(title, price);

    if !price_updated {
        // Find other potential game thresholds
        let thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
        let mut fuzzy_thresholds: Vec<String> = Vec::new();
        let mut fuzzy_alias: Vec<String> = Vec::new();
        for thresh in thresholds {
            let mut dist = fuzzy::levenshtein_distance(title, thresh.title.as_str());
            if 1.0 - (dist / thresh.title.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                fuzzy_thresholds.push(thresh.title);
                fuzzy_alias.push(thresh.alias);
            } else if !thresh.alias.is_empty() {
                dist = fuzzy::levenshtein_distance(title, thresh.alias.as_str());
                if 1.0 - (dist / thresh.alias.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                    fuzzy_thresholds.push(thresh.title);
                    fuzzy_alias.push(thresh.alias);
                }
            }
        }

        if fuzzy_thresholds.is_empty() {
            return;
        }

        println!("Did you mean one of the following?");
        for (idx, game_title) in fuzzy_thresholds.iter().enumerate() {
            println!("  [{}] {}", idx, game_title);
        }
        println!("  [c] Cancel");
        let mut input = String::new();
        print!("Update price by index or type \'c\' to cancel: ");
        let _ = io::stdout().flush();
        // TODO: Propagate terminal input errors instead of panicking
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "c" {
            println!("Threshold price update of '{}' cancelled.", title);
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < fuzzy_thresholds.len() {
                        let alias = fuzzy_alias[idx].as_str();
                        if !alias.is_empty() {
                            update_price(alias, price);
                        } else {
                            update_price(&fuzzy_thresholds[idx], price);
                        }
                    } else if idx >= fuzzy_thresholds.len() {
                        eprintln!("Integer \"{}\" is invalid. Remove cancelled.", idx);
                    }
                }
                Err(e) => println!("Invalid input: {}\nError: {}", input, e),
            }
        }
    }
}

pub fn update_id(title: &str, store_type: GameStore, id: u32) {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let idx = thresholds
        .iter()
        .position(|threshold| is_threshold(title, threshold));
    if let Some(i) = idx {
        let mut updated_id: bool = false;
        match store_type {
            GameStore::STEAM => {
                thresholds[i].steam_id = id;
                updated_id = true;
            }
            GameStore::GOOD_OLD_GAMES => {
                thresholds[i].gog_id = id;
                updated_id = true;
            }
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let _update_err = format!(
                "Could not convert the {} id update to a string object.",
                store_type
            );
            let _ = update_thresholds(thresholds);
            // println!("Updated {} ID for \"{}\"", store_name, title);
        }
    }
}

pub fn update_id_str(title: &str, store_type: GameStore, id: &str) {
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let idx = thresholds
        .iter()
        .position(|threshold| is_threshold(title, threshold));
    if let Some(i) = idx {
        let mut updated_id: bool = false;
        match store_type {
            GameStore::MICROSOFT_STORE_PC => {
                thresholds[i].microsoft_store_id = id.to_string();
                updated_id = true;
            }
            _ => eprintln!("Unknown store type: {}", store_type),
        }
        if updated_id {
            let _update_err = format!(
                "Could not convert the {} id update to a string object.",
                store_type
            );
            let _ = update_thresholds(thresholds);
            // println!("Updated {} ID for \"{}\"", store_name, title);
        }
    }
}

pub fn remove(title: &str) -> bool {
    let mut alias_map: HashMap<String, Vec<String>> = load_alias_map().unwrap_or_default();
    let mut thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
    let mut threshold_removed = false;
    if does_alias_exist(title) {
        let title_list = alias_map.get(title).unwrap();
        for game_title in title_list.iter() {
            if let Some(idx) = thresholds
                .iter()
                .position(|threshold| *game_title == threshold.title)
            {
                println!("Removing \"{}\".", thresholds[idx].title);
                thresholds.remove(idx);
                threshold_removed = true;
            };
        }
        alias_map.remove(title);
    } else {
        for i in (0..thresholds.len()).rev() {
            if is_threshold(title, &thresholds[i]) {
                if !thresholds[i].alias.is_empty()
                    && let Some(aliases) = alias_map.get_mut(&thresholds[i].alias)
                {
                    if let Some(idx) = aliases
                        .iter()
                        .position(|title| title == &thresholds[i].title)
                    {
                        aliases.remove(idx);
                    };
                    if aliases.is_empty() {
                        alias_map.remove(&thresholds[i].alias);
                    }
                }
                println!("Removing \"{}\".", thresholds[i].title);
                thresholds.remove(i);
                threshold_removed = true;
            }
        }
    }
    if threshold_removed {
        let _ = update_alias_map(alias_map);
        let _ = update_thresholds(thresholds);
        true
    } else {
        // println!("Failed to remove game using title/alias: \"{}\".", title);
        false
    }
}

pub fn remove_fuzzy(title: &str) {
    //Try to run the normal remove first
    let is_removed = remove(title);

    if !is_removed {
        // Find other potential game thresholds
        let thresholds = load_thresholds().unwrap_or_else(|_e| Vec::new());
        let mut fuzzy_thresholds: Vec<String> = Vec::new();
        for thresh in thresholds {
            let mut dist = fuzzy::levenshtein_distance(title, thresh.title.as_str());
            if 1.0 - (dist / thresh.title.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                fuzzy_thresholds.push(thresh.title);
            } else if !thresh.alias.is_empty() {
                dist = fuzzy::levenshtein_distance(title, thresh.alias.as_str());
                if 1.0 - (dist / thresh.alias.len() as f32) >= LEVENSTEIN_DIST_PERCENTAGE {
                    fuzzy_thresholds.push(thresh.title);
                }
            }
        }

        if fuzzy_thresholds.is_empty() {
            return;
        }

        println!("Did you mean one of the following?");
        for (idx, game_title) in fuzzy_thresholds.iter().enumerate() {
            println!("  [{}] {}", idx, game_title);
        }
        println!("  [c] Cancel");
        let mut input = String::new();
        print!("Remove game title by index or type \'c\' to cancel: ");
        let _ = io::stdout().flush();
        // TODO: Propagate terminal input errors instead of panicking
        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read user input");
        if input.trim() == "c" {
            println!("Threshold removal of '{}' cancelled.", title);
        } else {
            match input.trim().parse::<usize>() {
                Ok(idx) => {
                    if idx < fuzzy_thresholds.len() {
                        remove(&fuzzy_thresholds[idx]);
                    } else if idx >= fuzzy_thresholds.len() {
                        eprintln!("Integer \"{}\" is invalid. Remove cancelled.", idx);
                    }
                }
                Err(e) => println!("Invalid input: {}\nError: {}", input, e),
            }
        }
    }
}

pub fn list_games() {
    match load_thresholds() {
        Ok(data) => {
            println!("Price Thresholds");
            for threshold in data.iter() {
                if threshold.alias.is_empty() {
                    println!(
                        "  - {} => {} ({})",
                        threshold.title, threshold.desired_price, threshold.currency
                    );
                } else {
                    println!(
                        "  - {} [{}] => {} ({})",
                        threshold.title,
                        threshold.alias,
                        threshold.desired_price,
                        threshold.currency
                    );
                }
            }
        }
        Err(e) => println!("Error: {}", e),
    }
}
