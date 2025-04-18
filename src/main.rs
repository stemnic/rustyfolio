mod portfolio;
mod importer;
use portfolio::{Portfolio, Positions};

static MENU_OPTIONS: &str = r#"
    1. Add item
    2. Import statement
    9. Exit
    "#;
static IMPORTER_SUBMENU_OPTIONS: &str = r#"
    1. Etrade
    Other option go back
    "#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut portfolio = Portfolio::new().expect("Failed to create portfolio");
    println!("Welcome to rustyfolio! What do you want todo?");
    portfolio.load_from_disk()?;
    loop {
        println!("{}", MENU_OPTIONS);
        let mut buffer = String::new();
        std::io::stdin().read_line(&mut buffer)?;
        buffer = buffer.replace("\r", "");
        buffer = buffer.replace("\n", "");
        match buffer.as_str() {
            "1" => {
                portfolio.new_stock(Positions{ ticker: "INTC".to_string(), shares: vec![]})?;
            },
            "2" => {
                println!("{}", IMPORTER_SUBMENU_OPTIONS);
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                buffer = buffer.replace("\r", "");
                buffer = buffer.replace("\n", "");
                match buffer.as_str() {
                    "1" => {
                        println!("")
                    }
                    _ => {}
                }
            }
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
