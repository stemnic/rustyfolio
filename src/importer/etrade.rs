use calamine::{
    Error, RangeDeserializerBuilder, Reader, Xlsx, deserialize_as_f64_or_none, open_workbook,
};
use log::{debug, info};
use serde::Deserialize;
use std::collections::HashMap;

use crate::portfolio::{Action, Positions, Stock};

use super::Importer;

#[derive(Debug)]
struct SimpleError(String);

impl std::fmt::Display for SimpleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl std::error::Error for SimpleError {}

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
    #[serde(rename = "Symbol")]
    symbol: Option<String>,
    #[serde(rename = "Vested Qty.")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    total_vested: Option<f64>,
    #[serde(rename = "Grant Number")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    grant_number: Option<f64>,
}

#[derive(Debug, Deserialize)]
struct RsuGrantVest {
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
#[derive(Debug, Deserialize)]
struct GainAndLoss {
    #[serde(rename = "Record Type")]
    event: Option<String>,
    #[serde(rename = "Symbol")]
    symbol: Option<String>,
    #[serde(rename = "Qty.")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    num: Option<f64>,
    #[serde(rename = "Date Sold")]
    date: Option<String>,
    #[serde(rename = "Proceeds Per Share")]
    #[serde(deserialize_with = "deserialize_as_f64_or_none")]
    price: Option<f64>,
    #[serde(rename = "Order Type")]
    order_type: Option<String>,
}

pub struct EtradeImporter {
    positions: Vec<crate::Positions>,
    espp: Vec<EsppRecord>,
    rsugrant: Vec<RsuGrant>,
    rsugrantvest: Vec<RsuGrantVest>,
    rsutax: Vec<RsuTax>,
    gl_expanded: Vec<GainAndLoss>,
}

impl EtradeImporter {
    pub fn new() -> Self {
        EtradeImporter {
            positions: vec![],
            espp: vec![],
            rsugrant: vec![],
            rsugrantvest: vec![],
            rsutax: vec![],
            gl_expanded: vec![],
        }
    }
    fn parse_xlsx_file(&mut self, file_path: &str) -> Result<(), calamine::Error> {
        let mut workbook: Xlsx<_> = open_workbook(file_path)?;
        let espp = workbook.worksheet_range("ESPP");
        let rsu = workbook.worksheet_range("Restricted Stock");
        let gl_expanded = workbook.worksheet_range("G&L_Expanded");
        if espp.is_ok() {
            debug!("espp:");
            let espp = espp?;
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Symbol",
                "Purchase Date",
                "Purchase Price",
                "Purchased Qty.",
                "Grant Date FMV",
                "Purchase Date FMV",
            ])
            .from_range(&espp)?;
            for val in iter {
                if val.is_err() {
                    continue;
                }
                let record: EsppRecord = val?;
                debug!("{:?}", record);
                self.espp.push(record);
            }
        }
        if rsu.is_ok() {
            debug!("rsu");
            let rsu = rsu?;
            // Finds RSU total issue
            let iter =
                RangeDeserializerBuilder::with_headers(&["Symbol", "Vested Qty.", "Grant Number"])
                    .from_range(&rsu)?;
            for val in iter {
                if val.is_err() {
                    continue;
                }
                let record: RsuGrant = val?;
                debug!("{:?}", record);
                self.rsugrant.push(record);
            }
            // Finds RSU total issue
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Record Type",
                "Grant Number",
                "Vest Period",
                "Vest Date",
                "Reason for cancelled qty",
                "Released Qty",
            ])
            .from_range(&rsu)?;
            for val in iter {
                if val.is_err() {
                    continue;
                }
                let record: RsuGrantVest = val?;
                if record.cancel_reason.is_some() {
                    // Only filled when stock grant has been terminated, therefore they have never been granted
                    continue;
                }
                debug!("{:?}", record);
                self.rsugrantvest.push(record);
            }
            // Decode value per share based on tax value
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Record Type",
                "Grant Number",
                "Vest Period",
                "Taxable Gain",
            ])
            .from_range(&rsu)?;
            for val in iter {
                if val.is_err() {
                    continue;
                }
                let record: RsuTax = val?;
                debug!("{:?}", record);
                self.rsutax.push(record);
            }
        }
        if gl_expanded.is_ok() {
            // Sell events
            debug!("gl found:");
            let gl_expanded = gl_expanded?;
            let iter = RangeDeserializerBuilder::with_headers(&[
                "Record Type",
                "Symbol",
                "Qty.",
                "Date Sold",
                "Proceeds Per Share",
                "Order Type",
            ])
            .from_range(&gl_expanded)?;
            for val in iter {
                if val.is_err() {
                    continue;
                }
                let record: GainAndLoss = val?;
                debug!("{:?}", record);
                self.gl_expanded.push(record);
            }
        }
        Ok(())
    }
    fn process_rsu(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        // Check if we have as many rsugrant and rsutax entries
        debug!("Proccesing RSUs:");
        if self.rsugrantvest.len() != self.rsutax.len() {
            return Err(Box::new(SimpleError(
                "There is not an equal rsugran and rsutax entries".into(),
            )));
        }
        for rsugrant in self.rsugrant.iter() {
            let vest_total = rsugrant.total_vested.unwrap();
            let symbol = rsugrant.symbol.clone().unwrap();
            let mut found_total = 0.0;
            let mut rsu_shares: Vec<Stock> = vec![];
            for rsugrantvest in self.rsugrantvest.iter() {
                if rsugrant.grant_number == rsugrantvest.grant_number {
                    for rsutax in self.rsutax.iter() {
                        if (rsugrantvest.grant_number == rsutax.grant_number)
                            && (rsugrantvest.vest_period == rsutax.vest_period)
                        {
                            let date = chrono::NaiveDate::parse_from_str(
                                rsugrantvest.vest_date.clone().unwrap().as_str(),
                                "%m/%d/%Y",
                            )?;
                            let amount = rsugrantvest.release_qty.clone().unwrap();
                            let price = rsutax.taxable_gain.clone().unwrap() / amount;
                            found_total = found_total + amount;
                            let grant_number = rsutax.grant_number.clone().unwrap();
                            let vest_period = rsutax.vest_period.clone().unwrap();
                            info!(
                                "RSU {} {:?} {:?} {:?} {}-{}",
                                symbol, date, amount, price, grant_number, vest_period
                            );
                            let metadata_string = format!("RSU-{}-{}", grant_number, vest_period);
                            rsu_shares.push(Stock {
                                date: date,
                                price: price,
                                currency: "USD".to_string(),
                                unit: amount,
                                action: Action::Buy,
                                metadata: metadata_string,
                            });
                        }
                    }
                }
            }
            if vest_total != found_total {
                return Err(Box::new(SimpleError(
                    "Did not find the correct numbers of RSUs".into(),
                )));
            }
            let mut ticker_found = false;
            for position in self.positions.iter_mut() {
                if position.ticker == symbol {
                    ticker_found = true;
                    position.shares.append(&mut rsu_shares);
                }
            }
            if !ticker_found {
                self.positions.push(Positions {
                    ticker: symbol,
                    shares: rsu_shares,
                });
            }
        }

        Ok(())
    }
    fn process_espp(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for espp_action in self.espp.iter() {
            let symbol = espp_action.symbol.clone().unwrap();
            let date = chrono::NaiveDate::parse_from_str(
                espp_action.purchase_date.clone().unwrap().as_str(),
                "%d-%b-%Y",
            )?;
            let amount = espp_action.purchased_qty.unwrap();
            let price_string = espp_action
                .purchase_date_fmv
                .clone()
                .unwrap()
                .replace("$", "");
            let price: f64 = price_string.parse().unwrap();

            info!("ESPP {} {:?} {:?} {:?}", symbol, date, amount, price);
            let metadata_string = format!("ESPP");
            let share = Stock {
                date: date,
                price: price,
                currency: "USD".to_string(),
                unit: amount,
                action: Action::Buy,
                metadata: metadata_string,
            };
            let mut ticker_found = false;
            for position in self.positions.iter_mut() {
                if position.ticker == symbol {
                    ticker_found = true;
                    position.shares.push(share.clone());
                }
            }
            if !ticker_found {
                self.positions.push(Positions {
                    ticker: symbol,
                    shares: vec![share],
                });
            }
        }
        Ok(())
    }
    fn process_gl(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for gl_action in self.gl_expanded.iter() {
            let symbol = gl_action.symbol.clone().unwrap();
            let date = chrono::NaiveDate::parse_from_str(
                gl_action.date.clone().unwrap().as_str(),
                "%m/%d/%Y",
            )?;
            let amount = gl_action.num.unwrap();
            let price: f64 = gl_action.price.unwrap();
            let order_type = gl_action.order_type.clone().unwrap();

            let metadata_string = format!("{}", order_type);
            let share = Stock {
                date: date,
                price: price,
                currency: "USD".to_string(),
                unit: amount,
                action: Action::Sell,
                metadata: metadata_string,
            };
            let mut ticker_found = false;
            for position in self.positions.iter_mut() {
                if position.ticker == symbol {
                    ticker_found = true;
                    position.shares.push(share.clone());
                }
            }
            if !ticker_found {
                self.positions.push(Positions {
                    ticker: symbol,
                    shares: vec![share],
                });
            }
        }
        Ok(())
    }
}

impl Importer for EtradeImporter {
    fn import(
        &mut self,
        file_paths: Vec<String>,
    ) -> Result<&Vec<crate::Positions>, Box<dyn std::error::Error>> {
        // Need BenefitHistory.xlsx and G&L_Expanded.xlsx
        for file in file_paths.iter() {
            self.parse_xlsx_file(file)?;
        }
        self.process_rsu()?;
        self.process_espp()?;
        self.process_gl()?;

        Ok(&self.positions)
    }
}

#[cfg(test)]
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
            "{}/test_files/test_portfolio.json",
            env!("CARGO_MANIFEST_DIR")
        );
        let mut file = std::fs::File::open(test_portfolio)?;
        let mut porfolio_data = String::new();
        file.read_to_string(&mut porfolio_data)?;
        let port: Portfolio = serde_json::from_str(porfolio_data.as_str())?;
        Ok(port)
    }

    #[test]
    fn test_etrade_importer() {
        let test_portfolio = init().unwrap();
        let etrade = EtradeImporter::new();
        let mut importer =
            ImporterService::new_importer(etrade).expect("Creating Etrade importer failed");
        let benifit_history = format!(
            "{}/test_files/G&L_Expanded.xlsx",
            env!("CARGO_MANIFEST_DIR")
        );
        let gl_expanded = format!(
            "{}/test_files/BenefitHistory.xlsx",
            env!("CARGO_MANIFEST_DIR")
        );
        let res = importer.run(vec![benifit_history, gl_expanded]).unwrap();
        debug!("{:?}", res);
        assert_eq!(res, &test_portfolio.stocks);
    }
}
