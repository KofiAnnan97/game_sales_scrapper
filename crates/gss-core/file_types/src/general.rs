use std::fs::{self, File, write};
use std::path::Path;

pub fn get_path(path_str: &str) -> String{
    let path = Path::new(path_str);
    let mut is_new = false;
    if !path.is_file(){
        File::create_new(path_str).expect("Failed to create/load file");
        is_new = true;
    }
    let load_fp =  path.display().to_string();
    if is_new { println!("File created: {}", load_fp); }
    load_fp
}

pub fn create_dir(file_path: &str){
    if !Path::new(file_path).is_dir() { 
        match fs::create_dir_all(file_path){
            Ok(_) => println!("Created directory: {}", file_path),
            Err(e) => println!("Failed to create directory: {}, {}", file_path, e)
        }
    }
}

pub fn write_to_file(path: String, data: String){
    match write(&path, data) {
        Ok(_) => (),
        Err(e) => eprintln!("An error occurred while writing to \'{}\'\n{}", &path, e)
    }
}

pub fn write_file(path: &Path, filename: &str, data: &str) {
    let file_path = path.join(filename).display().to_string();
    write_to_file(file_path, data.to_string());
}

pub fn delete_file(file_path: String){
    match fs::remove_file(get_path(&file_path)){
        Ok(_) => println!("Successfully deleted {}", file_path),
        Err(e) => {eprintln!("{}",e)}
    }
}