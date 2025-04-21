use log::{debug, info};
use serde::Deserialize;

use crate::portfolio::{Action, Positions, Stock};

use super::Importer;

pub struct CsvImporter {
    positions: Vec<crate::Positions>,
}

impl CsvImporter {
    pub fn new() -> Self {
        CsvImporter { positions: vec![] }
    }
    fn import_csv(file: &String) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}

impl Importer for CsvImporter {
    fn import(
        &mut self,
        file_paths: Vec<String>,
    ) -> Result<&Vec<crate::Positions>, Box<dyn std::error::Error>> {
        for file in file_paths.iter() {
            CsvImporter::import_csv(file)?;
        }
        todo!("Some post processing?");

        Ok(&self.positions)
    }
}

mod tests {
    use crate::importer::ImporterService;
    use crate::portfolio::{Portfolio, Positions};
    use std::io::Read;

    use super::*;

    fn init() -> Result<Portfolio, std::io::Error> {
        let _ = env_logger::builder()
            .is_test(true)
            .filter_level(log::LevelFilter::Trace)
            .try_init();
        let test_portfolio = format!(
            "{}/test_files/test_portfolio_csv.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut file = std::fs::File::open(test_portfolio)?;
        let mut porfolio_data = String::new();
        file.read_to_string(&mut porfolio_data)?;
        let port: Portfolio = serde_json::from_str(porfolio_data.as_str())?;
        Ok(port)
    }

    #[test]
    #[ignore = "not yet implemented"]
    fn test_csv_importer() {
        let test_portfolio = init().unwrap();
        let csvimporter = CsvImporter::new();
        let mut importer =
            ImporterService::new_importer(csvimporter).expect("Creating CSV importer failed");
        let custom_csv = format!("{}/test_files/custom.csv", env!("CARGO_MANIFEST_DIR"));
        let res = importer.run(vec![custom_csv]).unwrap();
        debug!("{:?}", res);
        assert_eq!(res, &test_portfolio.stocks);
    }
}
