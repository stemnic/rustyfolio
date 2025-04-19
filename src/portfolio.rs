use std::io::{Read, Write};
use log::debug;
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Action {
    Buy,
    Sell,
}

#[derive(PartialEq, Clone)]
#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Stock {
    pub date: chrono::NaiveDate,
    pub price: f64,
    pub unit: f64,
    pub action: Action,
    pub metadata: String
}

#[derive(PartialEq)]
#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Positions {
    pub ticker: String,
    pub shares: Vec<Stock>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
pub struct Portfolio {
    pub name: String,
    pub description: String,
    pub stocks: Vec<Positions>,
}

static PORTFOLO_CONFIG_FILE : &str = "test_portfolio.json";

impl Portfolio {
    pub fn new() -> Result<Portfolio, Box<dyn std::error::Error>> {
        Ok(Portfolio { name: "My Portfolio".to_string(), description: "".to_string(), stocks: vec![] })
    }

    pub fn new_stock(&mut self, stock : Positions) -> Result<(), Box<dyn std::error::Error>>{
        self.merge_postions(&vec![stock])?;
        Ok(())
    }

    pub fn merge_postions(&mut self, to_be_merged_pos: &Vec<Positions>) -> Result<(), Box<dyn std::error::Error>> {
        for imp_pos in to_be_merged_pos.iter() {
            let mut ticker_exists = false;
            for pos in self.stocks.iter_mut() {
                if imp_pos.ticker == pos.ticker {
                    ticker_exists = true;
                    for imp_pos_stock in imp_pos.shares.iter() {
                        let mut share_exist = false;
                        for pos_stock in pos.shares.iter(){
                            if (imp_pos_stock.metadata == pos_stock.metadata) && (imp_pos_stock.date == pos_stock.date) && (imp_pos_stock.price == pos_stock.price){
                                share_exist = true;
                                debug!("Share already exists {:?}", imp_pos_stock);
                            }
                        }
                        if !share_exist {
                            debug!("Adding share {:?} to {}", imp_pos_stock, pos.ticker);
                            pos.shares.push(imp_pos_stock.clone());
                        }
                    }
                }
            }
            if !ticker_exists {
                debug!("Adding ticker {}", imp_pos.ticker);
                self.stocks.push(Positions { ticker: imp_pos.ticker.clone(), shares: imp_pos.shares.clone() });
            }
        }
        Ok(())
    }

    pub fn load_from_disk(&mut self) -> Result<(), std::io::Error>{
        let mut file = match File::open(PORTFOLO_CONFIG_FILE){
            Ok(res) => res,
            Err(err) => {
                match err.kind() {
                    std::io::ErrorKind::NotFound => {
                        return Ok(());
                    }
                    _ => {
                        todo!("Unhandled error");
                    }
                }
            }
        };
        let mut porfolio_data = String::new();
        file.read_to_string(&mut porfolio_data)?;
        let port : Portfolio = serde_json::from_str(porfolio_data.as_str())?;
        *self = port;
        Ok(())
    }
    pub fn store_to_disk(&self) -> Result<(), std::io::Error>{
        let portfolio_json = serde_json::to_string(self)?;
        let mut file = File::create(PORTFOLO_CONFIG_FILE)?;
        file.write_all(portfolio_json.as_bytes())?;
        Ok(())
    }
}
