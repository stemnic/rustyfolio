mod csv;
mod etrade;

use log::{debug, info};

pub use crate::importer::csv::CsvImporter;
pub use crate::importer::etrade::EtradeImporter;

pub trait Importer {
    fn import(
        &mut self,
        file_paths: &Vec<String>,
    ) -> Result<&Vec<crate::Positions>, Box<dyn std::error::Error>>;
}

pub struct ImporterService<I: Importer> {
    importer: I,
}

impl<I: Importer> ImporterService<I> {
    pub fn new_importer(imp: I) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(ImporterService { importer: imp })
    }
    pub fn run(
        &mut self,
        file_paths: &Vec<String>,
    ) -> Result<&Vec<crate::Positions>, Box<dyn std::error::Error>> {
        self.importer.import(file_paths)
    }
}
