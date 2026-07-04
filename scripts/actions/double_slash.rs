use std::env;

static BACK_SLASH : &str = "\\";
static DB_BACK_SLASH : &str = r"\\";
static EMPTY : &str = "";

fn main() {
    match env::args().nth(1){
        Some(dir_path) => {
            let mut new_path: String = String::new();
            for i in 0..dir_path.len(){
                let path_idx = new_path.len();
                new_path.push_str(
                    if i == 0 && &dir_path[i..i+1] == BACK_SLASH { DB_BACK_SLASH }
                    else if i == dir_path.len()-1{
                        if &dir_path[i..i+1] == BACK_SLASH && &dir_path[i-1..i] != BACK_SLASH { DB_BACK_SLASH }
                        else if &dir_path[i..i+1] != BACK_SLASH { &dir_path[i..i+1] }
                        else { EMPTY }
                    } 
                    else if &dir_path[i..i+1] == BACK_SLASH && (&dir_path[i+1..i+2] != BACK_SLASH && &dir_path[i-1..i] != BACK_SLASH){ DB_BACK_SLASH }
                    else if path_idx > 1 && &new_path[path_idx-2..path_idx-1] == BACK_SLASH && &dir_path[i..i+1] == BACK_SLASH { EMPTY }
                    else { &dir_path[i..i+1] }
                );
            }
            println!("{}", new_path)
        },
        None => println!("Please provide the filepath encapsulated in double quotes (\"\").")
    }
}