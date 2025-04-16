mod portfolio;
use portfolio::{Portfolio, Positions};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut portfolio = Portfolio::new().expect("Failed to create portfolio");
    println!("Welcome to rustyfolio! What do you want todo?");
    portfolio.load_from_disk()?;
    loop {
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        println!("stdin:{:?} ", buffer);
        buffer = buffer.replace("\r", "");
        buffer = buffer.replace("\n", "");
        match buffer.as_str() {
            "1" => {
                portfolio.new_stock(Positions{ ticker: "INTC".to_string(), shares: vec![]})?;
            },
            "9" => {
                break;
            }
            _ => {
                println!("Invalid option")
            }
        }
    }
    portfolio.store_to_disk()?;
    Ok(())
}
