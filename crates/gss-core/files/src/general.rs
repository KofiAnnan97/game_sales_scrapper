use std::fs::{self, File, OpenOptions, write};
use std::io::{ErrorKind, prelude::*};
use std::path::{Path, PathBuf};

use errors::input::FileOperationError;

pub fn get_path(path_str: &str) -> Result<String, FileOperationError> {
    let path = Path::new(path_str);
    let mut is_new = false;
    if !path.is_file() {
        match OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(path_str)
        {
            Ok(_) => is_new = true,
            Err(e) => {
                if e.kind() != ErrorKind::AlreadyExists {
                    return Err(FileOperationError::Message(format!(
                        "Failed to create/load file: {}",
                        e
                    )));
                }
            }
        }
    }
    let load_fp = path.display().to_string();
    if is_new {
        println!("File created: {}", load_fp);
    }
    Ok(load_fp)
}

pub fn get_file(file_path: &str) -> Result<File, FileOperationError> {
    let file = File::open(file_path)?;
    Ok(file)
}

pub fn get_file_size(path: &PathBuf) -> Result<u64, FileOperationError> {
    match fs::metadata(path) {
        Ok(metadata) => Ok(metadata.len()),
        Err(e) => Err(e.into()),
    }
}

pub fn create_dir(file_path: &str) -> Result<(), FileOperationError> {
    if Path::new(file_path).is_dir() {
        return Ok(());
    }

    match fs::create_dir_all(file_path) {
        Ok(_) => Ok(()),
        Err(e) => Err(FileOperationError::Message(format!(
            "Failed to create directory: {}, {}",
            file_path,
            e
        ))),
    }
}

pub fn write_to_file(path: String, data: String) -> Result<(), FileOperationError> {
    match write(&path, data) {
        Ok(_) => Ok(()),
        Err(e) => Err(FileOperationError::Message(format!(
            "An error occurred while writing to \'{}\'\n{}",
            path, e
        ))),
    }
}

pub fn write_file(path: &Path, filename: &str, data: &str) -> Result<(), FileOperationError> {
    let file_path = path.join(filename).display().to_string();
    match write_to_file(file_path, data.to_string()) {
        Ok(_) => Ok(()),
        Err(e) => Err(e),
    }
}

pub fn append_to_file(path: &str, data: &str) -> Result<(), FileOperationError> {
    let file = OpenOptions::new().create(true).append(true).open(path);
    match file {
        Ok(mut f) => match f.write_all(data.as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(FileOperationError::Message(format!(
                "Could not append to file, {}",
                e
            ))),
        },
        Err(e) => Err(FileOperationError::Message(format!(
            "Could not open file. {}",
            e
        ))),
    }
}

pub fn get_contents(path: &str) -> Result<String, FileOperationError> {
    match fs::read_to_string(path) {
        Ok(contents) => Ok(contents),
        Err(e) => Err(FileOperationError::Message(format!("{}", e))),
    }
}

pub fn delete_file(file_path: String) -> Result<(), FileOperationError> {
    let path_str = get_path(&file_path)?;
    match fs::remove_file(path_str) {
        Ok(_) => Ok(()),
        Err(e) => Err(FileOperationError::Message(format!("{}", e))),
    }
}
