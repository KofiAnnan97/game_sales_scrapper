use dotenv::dotenv;
use std::sync::{Mutex};
use std::env;
use cfg_if::cfg_if;
use lazy_static::lazy_static;
use std::fs::{self, File, write};
use std::path::{self, Path, PathBuf};

static TEST_VAR_NAME : &str = "TEST_PATH";
static PROJECT_VAR_NAME : &str = "PROJECT_PATH";

lazy_static! {
    static ref PATH_ENV_VAR : Mutex<String> = {
        cfg_if! {
            if #[cfg(not(test))] { Mutex::new(PROJECT_VAR_NAME.to_string()) }
            else if #[cfg(test)] { Mutex::new(TEST_VAR_NAME.to_string()) }
        }
    };
}

pub fn enable_test_flag() {
    *PATH_ENV_VAR.lock().unwrap() = String::from(TEST_VAR_NAME);
}

pub fn get_path(path_str: &str) -> String{
    let path = Path::new(path_str);
    let mut load_fp = String::new();
    if !path.is_file(){
        File::create_new(path_str).expect("Failed to create load file");
        load_fp = path.display().to_string();
        println!("File created: {}", load_fp);
    }
    else{ load_fp = path.display().to_string(); }
    load_fp
}

pub fn write_to_file(path: String, data: String){
    match write(&path, data) {
        Ok(_) => (),
        Err(e) => eprintln!("An error occurred while writing to \'{}\'\n{}", &path, e)
    }
}

pub fn delete_file(file_path: String){
    match fs::remove_file(get_path(&file_path)){
        Ok(_) => println!("Successfully deleted {}", file_path),
        Err(e) => {eprintln!("{}",e)}
    }
}

pub fn get_data_path() -> String {
    dotenv().ok();
    let path_env = PATH_ENV_VAR.lock().unwrap().clone();
    //println!("Env var: {:?}", path_env);
    let mut data_path = env::var(path_env).unwrap_or_else(|_| String::from("."));
    let path: PathBuf = [&data_path, "data"].iter().collect();
    data_path = path.display().to_string();
    println!("Path: {}", data_path);
    if Path::new(&data_path).is_dir() != true {
        let _ = fs::create_dir(&data_path);
    }
    data_path
}