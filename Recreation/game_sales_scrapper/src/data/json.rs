use dotenv::dotenv;
use std::{fs, env};
use std::fs::{File, write};
use std::path::Path;

pub fn get_path(path_str: &str) -> String{
    let path = Path::new(path_str);
    let mut load_fp = String::from("");
    if !path.is_file(){
        File::create_new(path_str).expect("Failed to create load file");
        load_fp = path.display().to_string();
        println!("File created: {}", load_fp);
    }
    else{ load_fp = path.display().to_string(); }
    return load_fp;
}

pub fn write_to_file(path: String, data: String){
    write(path, data).expect("Data could not be saved.\n");
}

pub fn get_data_path() -> String {
    dotenv().ok();
    let mut data_path = String::from(".");
    match std::env::var("PRGM_PATH") {
        Ok(path) =>  data_path = path,
        Err(_) => println!("\'PRGM_PATH\' was not set in .env")
    }
    data_path.push_str("/data");
    if Path::new(&data_path).is_dir() != true {
        let _ = fs::create_dir(&data_path);
    }
    return data_path;
}