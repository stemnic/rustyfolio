use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use std::fs::File;

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub enum Action {
    Buy,
    Sell,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Stock {
    price: usize,
    unit: usize,
    action: Action
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Positions {
    pub ticker: String,
    pub shares: Vec<Stock>,
}

#[derive(Serialize, Deserialize)]
#[derive(Debug)]
pub struct Portfolio {
    name: String,
    description: String,
    stocks: Vec<Positions>,
}

static PORTFOLO_CONFIG_FILE : &str = "test_portfolio.json";

impl Portfolio {
    pub fn new() -> Result<Portfolio, Box<dyn std::error::Error>> {
        Ok(Portfolio { name: "My Portfolio".to_string(), description: "".to_string(), stocks: vec![] })
    }

    pub fn new_stock(&mut self, stock : Positions) -> Result<(), Box<dyn std::error::Error>>{
        self.stocks.push(stock);
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
