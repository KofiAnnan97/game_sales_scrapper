use std::fs::{File, write};
use std::path::Path;
use std::vec;
use std::{error::Error, io, process};

#[derive(Debug)]
pub struct DesiredGamePrice {
    pub name: String,
    pub price: f64,
}

pub fn parse_game_prices(file_path: &str) -> Result<Vec<DesiredGamePrice>, Box<dyn Error>>{
    let mut game_list: Vec<DesiredGamePrice> = Vec::new();
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records(){
        let record = result?;
        if record.len() == 2 {
            let mut price : f64 = -1.0;
            match record.get(1) {
                Some(val) => {
                    match val.trim().parse::<f64>() {
                        Ok(f_val) => {
                            price = f_val;
                            game_list.push(DesiredGamePrice {
                               name: record.get(0).unwrap().to_string(),
                               price: price,
                            });
                        },
                        Err(e) => eprintln!("{}", e),
                    }
                },
                None => eprintln!("Price value could not be parse. Please check CSV."),
            }
        }
    }
    Ok(game_list)
}