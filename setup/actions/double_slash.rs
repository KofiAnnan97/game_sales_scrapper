use std::env;

fn add_double_slash(dir_path: String) -> String {
    let mut new_path: String = String::new();
    for i in 0..dir_path.len(){
        new_path.push_str(
            if &dir_path[i..i+1] == "\\" && (&dir_path[i+1..i+2] != "\\" && &dir_path[i-1..i] != "\\"){ r"\\" }
            else { &dir_path[i..i+1] }
        );
    }
    new_path
}

fn main() {
    match env::args().nth(1){
        Some(dir_path) => println!("{}", add_double_slash(dir_path)),
        None => println!("Please provide the filepath encapsulated in double quotes (\"\").")
    }
}