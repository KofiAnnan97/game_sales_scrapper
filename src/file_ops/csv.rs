use std::fs::{File};
use std::{error::Error};

use crate::structs::data::SimpleGameThreshold;

pub fn parse_game_prices(file_path: &str) -> Result<Vec<SimpleGameThreshold>, Box<dyn Error>>{
    let mut game_list: Vec<SimpleGameThreshold> = Vec::new();
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records(){
        let record = result?;
        if record.len() == 2 {
            let mut threshold_price : f64 = -1.0;
            match record.get(1) {
                Some(val) => {
                    match val.trim().parse::<f64>() {
                        Ok(f_val) => {
                            threshold_price = f_val;
                            game_list.push(SimpleGameThreshold {
                               name: record.get(0).unwrap().to_string(),
                               price: threshold_price,
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