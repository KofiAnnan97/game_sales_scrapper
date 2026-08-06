use std::{env, fs};
use std::path::{Path, PathBuf};
use serde_json::json;
use std::sync::{Mutex, MutexGuard, OnceLock};

use files::general;
use constants::operations::thresholds::{ALIAS_MAP, THRESHOLDS, THRESHOLD_FILENAME};
use constants::operations::settings::{ALIASES_ENABLED, ALLOW_ALIAS_REUSE_AFTER_CREATION, 
                                      SELECTED_STORES, SETTINGS_FILENAME};
use constants::operations::properties::{CONFIG_DIR, DATA_DIR, PROJECT_PATH_ENV, TEST_PATH_ENV, DEFAULT_TEST_DIR};
use constants::stores::steam::{CACHE_FILENAME};
use types::response::steam::App;

static TMP_DIR_PREFIX: &str = "gss_tests";
static TEST_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

// Utility functions for setting up temporary environment for tests

pub fn test_lock() -> &'static Mutex<()> {
    TEST_LOCK.get_or_init(|| Mutex::new(()))
}

pub fn retrieve_env_var(env_var: &str) -> Option<String> {
    env::var(env_var).ok()
}

pub fn restore_env_var(env_var: &str, value: Option<String>) {
    unsafe {
        match value {
            Some(data) => env::set_var(env_var, data),
            None => env::remove_var(env_var),
        }
    }
}

// Temporary environment setup
pub struct TempEnvironment {
    pub _guard: MutexGuard<'static, ()>,
    pub temp_dir: PathBuf,
    pub prev_project_path: Option<String>,
    pub prev_test_path: Option<String>,
    pub prev_dir: PathBuf,
}

impl TempEnvironment {
    pub fn tear_down(self) {
        clean_up(&self.prev_dir, &self.temp_dir, self.prev_project_path, self.prev_test_path);
    }
}

fn get_dir(title: &str) -> PathBuf {
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = env::temp_dir().join(format!("{}_{}_{}", TMP_DIR_PREFIX, title, timestamp));
    fs::create_dir_all(&path).unwrap();
    path
}

pub fn create(title: &str, steam_cache: Vec<App>) -> PathBuf {
    // Initial file setup for testing
    let settings_json = serde_json::to_string_pretty(&json!({
        SELECTED_STORES.to_string(): [],
        ALIASES_ENABLED.to_string(): 1,
        ALLOW_ALIAS_REUSE_AFTER_CREATION.to_string(): 1
    })).expect("Failed to write settings for testing");

    let thresholds_json = serde_json::to_string_pretty(&json!({
        THRESHOLDS.to_string(): [],
        ALIAS_MAP.to_string(): {}
    })).expect("Failed to write thresholds for testing");
    
    // Setup based temporary directory structure for testing
    let base_dir = get_dir(title);
    let data_dir = base_dir.join(DATA_DIR);
    let config_dir = base_dir.join(CONFIG_DIR);

    general::create_dir(&data_dir.display().to_string());
    general::create_dir(&config_dir.display().to_string());

    let tmp_thresholds_file = data_dir.join(THRESHOLD_FILENAME).with_extension("json").display().to_string();
    let tmp_settings_file = config_dir.join(SETTINGS_FILENAME).with_extension("json").display().to_string();
    let tmp_steam_cache_file = data_dir.join(CACHE_FILENAME).with_extension("json").display().to_string();

    general::write_to_file(tmp_steam_cache_file, serde_json::to_string_pretty(&steam_cache).expect("Failed to write steam cache for testing"));
    general::write_to_file(tmp_settings_file, settings_json.clone());
    general::write_to_file(tmp_thresholds_file, thresholds_json.clone());

    // Setup default test subdirectory structure within the temporary directory (in case properties sets test to subdirectory)
    let test_root = base_dir.join(DEFAULT_TEST_DIR);
    let test_data_dir = test_root.join(DATA_DIR);
    let test_config_dir = test_root.join(CONFIG_DIR);
    
    general::create_dir(&test_data_dir.display().to_string());
    general::create_dir(&test_config_dir.display().to_string());

    let tmp_thresholds_file_test = test_data_dir.join(THRESHOLD_FILENAME).with_extension("json").display().to_string();
    let tmp_settings_file_test = test_config_dir.join(SETTINGS_FILENAME).with_extension("json").display().to_string();
    let tmp_steam_cache_file_test = test_data_dir.join(CACHE_FILENAME).with_extension("json").display().to_string();

    general::write_to_file(tmp_steam_cache_file_test, serde_json::to_string_pretty(&steam_cache).expect("Failed to write steam cache for testing"));    
    general::write_to_file(tmp_settings_file_test, settings_json.clone());
    general::write_to_file(tmp_thresholds_file_test, thresholds_json.clone());

    base_dir
}

pub fn setup_tmp_environment(title: &str, steam_cache: Vec<App>) -> TempEnvironment {
    let guard = test_lock().lock().unwrap();
    let temp_dir = create(title, steam_cache);
    let prev_project_path = retrieve_env_var(PROJECT_PATH_ENV);
    let prev_test_path = retrieve_env_var(TEST_PATH_ENV);
    let prev_dir = env::current_dir().unwrap();

    unsafe {
        env::set_var(PROJECT_PATH_ENV, temp_dir.display().to_string());
        env::set_var(TEST_PATH_ENV, temp_dir.display().to_string());
    }
    env::set_current_dir(&temp_dir).unwrap();

    TempEnvironment {
        _guard: guard,
        temp_dir,
        prev_project_path,
        prev_test_path,
        prev_dir,
    }
}

pub fn clean_up(prev_dir: &Path, temp_dir: &Path, prev_project_path: Option<String>, prev_test_path: Option<String>) {
    env::set_current_dir(prev_dir).unwrap();
    restore_env_var(PROJECT_PATH_ENV, prev_project_path);
    restore_env_var(TEST_PATH_ENV, prev_test_path);
    if temp_dir.exists() {
        fs::remove_dir_all(temp_dir).unwrap();
        println!("Cleaned up temporary directory: {}", temp_dir.display());
    }
}