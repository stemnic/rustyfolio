use calamine::{deserialize_as_f64_or_none, open_workbook, Error, Xlsx, Reader, RangeDeserializerBuilder};
use std::collections::HashMap;
use serde::Deserialize;

#[derive(Debug, Clone)]
struct DoubleError;

impl std::fmt::Display for DoubleError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "invalid first item to double")
    }
}

enum ImporterTypes {
    ETrade
}

pub trait Importer {
    fn import(&self, file_paths : Vec<&str>) -> Result<crate::Positions, Box<dyn std::error::Error>>;
}

#[derive(Debug, Deserialize)]
struct EsppRecord {
    #[serde(rename = "Symbol")]
    symbol: Option<String>,
    #[serde(rename = "Purchase Date")]
    purchase_date: Option<String>,
    #[serde(rename = "Purchase Price")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    purchase_price: Option<f64>,
    #[serde(rename = "Purchased Qty.")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    purchased_qty: Option<f64>,
    #[serde(rename = "Grant Date FMV")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    grant_date_fmv: Option<f64>,
    #[serde(rename = "Purchase Date FMV")]
    purchase_date_fmv: Option<String>,
}

#[derive(Debug, Deserialize)]
struct RsuGrant {
    #[serde(rename = "Record Type")]
    record: Option<String>,
    #[serde(rename = "Grant Number")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    grant_number: Option<f64>,
    #[serde(rename = "Vest Period")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    vest_period: Option<f64>,
    #[serde(rename = "Vest Date")]
    vest_date: Option<String>,
    #[serde(rename = "Reason for cancelled qty")]
    cancel_reason: Option<String>,
    #[serde(rename = "Released Qty")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    release_qty: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RsuTax {
    #[serde(rename = "Record Type")]
    record: Option<String>,
    #[serde(rename = "Grant Number")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    grant_number: Option<f64>,
    #[serde(rename = "Vest Period")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    vest_period: Option<f64>,
    #[serde(rename = "Taxable Gain")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    taxable_gain: Option<f64>,
}

pub struct EtradeImporter {
    positions : Vec<crate::Positions>
}

impl EtradeImporter {
    fn new() -> Self {
        EtradeImporter { positions: vec![] }
    }
    fn parse_xlsx_file(&self, file_path : &str) -> Result<(), calamine::Error> {
        let mut workbook: Xlsx<_> = open_workbook(file_path)?;
        let espp = workbook.worksheet_range("ESPP");
        let rsu = workbook.worksheet_range("Restricted Stock");
        let gl_expanded = workbook.worksheet_range("G&L_Expanded");
        if espp.is_ok() {
            println!("espp:");
            let espp = espp?;
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Symbol",
                "Purchase Date",
                "Purchase Price",
                "Purchased Qty.",
                "Grant Date FMV",
                "Purchase Date FMV",
            ]).from_range(&espp)?;
            for val in iter {
                if val.is_err(){
                    continue;
                }
                let record: EsppRecord = val?;

                println!("{:?}", record);
            }
        }
        if rsu.is_ok() {
            println!("rsu");
            let rsu = rsu?;
            // Finds RSU total issue
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Record Type",
                "Grant Number",
                "Vest Period",
                "Vest Date",
                "Reason for cancelled qty",
                "Released Qty",
            ]).from_range(&rsu)?;
            for val in iter {
                if val.is_err(){
                    continue;
                }
                let record: RsuGrant = val?;
                if record.cancel_reason.is_some(){
                    // Only filled when stock grant has been terminated, therefore they have never been granted
                    continue;
                }

                println!("{:?}", record);
            }
            // Decode value per share based on tax value
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Record Type",
                "Grant Number",
                "Vest Period",
                "Taxable Gain",
            ]).from_range(&rsu)?;
            for val in iter {
                if val.is_err(){
                    continue;
                }
                let record: RsuTax = val?;

                println!("{:?}", record);
            }
        }
        if gl_expanded.is_ok() {
            // Sell events
            println!("gl found");
        }
        Ok(())
    }
}

impl Importer for EtradeImporter {
    fn import(&self, file_paths : Vec<&str>) -> Result<crate::Positions, Box<dyn std::error::Error>> {
        // Need BenefitHistory.xlsx and G&L_Expanded.xlsx

        for file in file_paths.iter() {
            self.parse_xlsx_file(file).unwrap();
        }
        todo!("ETrade importer not implemented");
    }
}

pub struct ImporterService <I : Importer> {
    importer : I
}

impl<I: Importer> ImporterService<I> {
    pub fn new_importer(imp : I) -> Result<Self, Box<dyn std::error::Error>>{
        Ok(ImporterService{ importer :  imp})
    }
    pub fn run(&self, file_paths : Vec<&str>) -> Result<crate::Positions, Box<dyn std::error::Error>> {
        self.importer.import(file_paths)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_etrade_importer() {
        let etrade = EtradeImporter::new();
        let importer = ImporterService::new_importer(etrade).expect("Creating Etrade importer failed");
        let benifit_history = format!("{}/test_files/G&L_Expanded.xlsx", env!("CARGO_MANIFEST_DIR"));
        let gl_expanded = format!("{}/test_files/BenefitHistory.xlsx", env!("CARGO_MANIFEST_DIR"));
        let res = importer.run(vec![benifit_history.as_str(), gl_expanded.as_str()]).expect("Importing failed");
        //assert_eq!(res, res_expected);
    }
}

