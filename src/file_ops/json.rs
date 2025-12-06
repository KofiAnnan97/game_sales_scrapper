use dotenv::dotenv;
use std::{fs};
use std::fs::{File, write};
use std::path::Path;

const TEST_VAR_NAME : &str = "TEST_PATH";
const PROJECT_VAR_NAME : &str = "PROJECT_PATH";

#[cfg(test)]
const PATH_ENV_VAR: &str = TEST_VAR_NAME;

#[cfg(not(test))]
const PATH_ENV_VAR: &str = PROJECT_VAR_NAME;

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
    write(path, data).expect("Data could not be saved.\n");
}

fn delete_file(file_path: String){
    match fs::remove_file(get_path(&file_path)){
        Ok(_) => println!("Successfully deleted {}", file_path),
        Err(e) => {eprintln!("{}",e)}
    }
}

pub fn get_data_path() -> String {
    dotenv().ok();
    let mut data_path = std::env::var(PATH_ENV_VAR).unwrap_or_else(|_| String::from("."));
    data_path.push_str("/data");
    //println!("Path: {}", data_path);
    if Path::new(&data_path).is_dir() != true {
        let _ = fs::create_dir(&data_path);
    }
    return data_path;
}