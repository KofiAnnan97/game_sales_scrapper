use std::fs::File;
use std::{error::Error};

use structs::internal::data::SimpleGameThreshold;
use crate::common;

pub fn parse_game_prices(file_path: &str) -> Result<Vec<SimpleGameThreshold>, Box<dyn Error>>{
    let mut game_list: Vec<SimpleGameThreshold> = Vec::new();
    let file = File::open(file_path)?;
    let mut reader = csv::Reader::from_reader(file);
    for result in reader.records(){
        let record = result?;
        if record.len() == 2 {
            match record.get(1) {
                Some(val) => {
                    match val.trim().parse::<f64>() {
                        Ok(f_val) => {
                            let threshold_price = f_val;
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

pub fn generate_csv(file_path: &str, thresholds: Vec<SimpleGameThreshold>) {
    let mut data = String::from("game, price");
    for sgt in thresholds {
        let row = format!("\n{}, {}", sgt.name, sgt.price); 
        data.push_str(row.as_str());
    }
    common::write_to_file(file_path.to_owned(), data);
}