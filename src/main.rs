mod importer;
mod portfolio;
mod tax;
use importer::{EtradeImporter, ImporterService};
use log::{error, info};
use portfolio::{Portfolio, Positions, Stock};

static MENU_OPTIONS: &str = r#"
    1. Show Position
    2. Import statement
    3. Run Fifo calc
    9. Exit
    "#;
static IMPORTER_SUBMENU_OPTIONS: &str = r#"
    1. Etrade
    Other option go back
    "#;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    //env_logger::init();
    let _ = env_logger::builder()
        .filter_level(log::LevelFilter::Trace)
        .try_init();
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
                println!("tbd");
            }
            "2" => {
                println!("{}", IMPORTER_SUBMENU_OPTIONS);
                let mut buffer = String::new();
                std::io::stdin().read_line(&mut buffer)?;
                buffer = buffer.replace("\r", "");
                buffer = buffer.replace("\n", "");
                match buffer.as_str() {
                    "1" => {
                        println!("Pass path for BenefitHistory.xlsx and G&L_Expanded.xlsx");
                        let mut files: Vec<String> = vec![];
                        loop {
                            let mut buffer = String::new();
                            std::io::stdin().read_line(&mut buffer)?;
                            buffer = buffer.replace("\r", "");
                            buffer = buffer.replace("\n", "");
                            if buffer.len() == 0 {
                                break;
                            }
                            files.push(buffer);
                        }
                        let mut invalid_files = false;
                        for f in files.iter() {
                            let path = std::path::Path::new(f);
                            if !path.is_file() {
                                invalid_files = true;
                                error!("{} is not a valid file", path.to_str().unwrap());
                            }
                        }
                        if !invalid_files {
                            let imp = EtradeImporter::new();
                            let mut importer = ImporterService::new_importer(imp)?;
                            let imported_port = importer.run(files)?;
                            portfolio.merge_postions(imported_port)?;
                        }
                    }
                    _ => {}
                }
            }
            "3" => {
                tax::TaxCalculatorService::fifo_calculation(&portfolio);
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
