mod etrade;

use calamine::{
    Error, RangeDeserializerBuilder, Reader, Xlsx, deserialize_as_f64_or_none, open_workbook,
};
use log::{debug, info};
use serde::Deserialize;
use std::collections::HashMap;

pub use crate::importer::etrade::EtradeImporter;
use crate::portfolio::{Action, Positions, Stock};

enum ImporterTypes {
    ETrade,
}

pub trait Importer {
    fn import(
        &mut self,
        file_paths: Vec<String>,
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
        file_paths: Vec<String>,
    ) -> Result<&Vec<crate::Positions>, Box<dyn std::error::Error>> {
        self.importer.import(file_paths)
    }
}
