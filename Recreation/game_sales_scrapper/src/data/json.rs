use std::fs::{File, write};
use std::path::Path;

pub fn get_path(path_str: &str) -> String{
    let path = Path::new(path_str);
    let mut load_fp = String::from("");
    if !path.is_file(){
        File::create_new(path_str).expect("Failed to create load file");
        println!("File created: {}", load_fp);
        load_fp = path.display().to_string();
    } 
    else{ load_fp = path.display().to_string(); }
    return load_fp;
}

pub fn write_to_file(path: String, data: String){
    write(path, data).expect("Data could not be saved.");
}