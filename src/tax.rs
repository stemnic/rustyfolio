use std::io::Write;

use crate::portfolio::{Action, Portfolio, Stock};
use chrono::Datelike;
use log::{debug, info};
pub struct TaxCalculatorService {}
static OUTPUT_FILE: &str = "output.csv";
impl TaxCalculatorService {
    fn overwrite_to_output_file(content: &str) -> Result<(), std::io::Error> {
        let filename = format!("{}/output/{}", env!("CARGO_MANIFEST_DIR"), OUTPUT_FILE);
        let mut file = std::fs::File::create(filename)?;
        file.write_all(content.as_bytes())?;
        info!("Writing output to {}", OUTPUT_FILE);
        Ok(())
    }

    pub fn fifo_calculation(portfolio: &Portfolio) {
        let mut output_string = String::new();
        for ticker in portfolio.stocks.iter() {
            info!("FIFO {}:", ticker.ticker);
            let mut sell: Vec<Stock> = vec![];
            let mut buy: Vec<Stock> = vec![];
            for stock in ticker.shares.iter() {
                match stock.action {
                    Action::Buy => buy.push(stock.clone()),
                    Action::Sell => sell.push(stock.clone()),
                }
            }
            sell.sort_by_key(|k| k.date);
            buy.sort_by(|a, b| b.date.cmp(&a.date)); // Reverse sort for FIFO function
            let mut year = 0;
            let mut last_year = 0;
            let mut sum_shares_sold = 0.0;
            let mut sum_gains_loss = 0.0;
            output_string = format!(
                "Ticker,Date,UnitsSold,GainOrLoss,BuyPrice,SellPrice,Diff,Profit,Currency,SellMetadata"
            );
            for sell_stock in sell.iter_mut() {
                year = sell_stock.date.year();
                if last_year != year {
                    info!("{}: {} {}", last_year, sum_shares_sold, sum_gains_loss);
                    sum_gains_loss = 0.0;
                    sum_shares_sold = 0.0;
                }
                while sell_stock.unit > 0.0 {
                    //debug! {"{:?}", sell_stock};
                    let mut buy_pos = buy.pop().unwrap();
                    let gains = sell_stock.price - buy_pos.price;

                    let mut unit_sold = sell_stock.unit;
                    let old_buy = buy_pos.unit;
                    buy_pos.unit = buy_pos.unit - sell_stock.unit;
                    sell_stock.unit = sell_stock.unit - old_buy;
                    if buy_pos.currency != sell_stock.currency {
                        todo!("Implement currency conversion logic.")
                    }

                    if buy_pos.unit < 0.0 {
                        unit_sold = old_buy;
                    }
                    let g_lstring = if gains >= 0.0 { "gain" } else { "loss" };
                    info!(
                        "{} {} \t {} {} {} {} \t {} {}",
                        sell_stock.date,
                        unit_sold,
                        g_lstring,
                        buy_pos.price,
                        sell_stock.price,
                        gains,
                        gains * unit_sold,
                        buy_pos.currency
                    );
                    output_string = format!(
                        "{}\n{},{},{},{},{},{},{},{},{},{}",
                        output_string,
                        ticker.ticker,
                        sell_stock.date,
                        unit_sold,
                        g_lstring,
                        buy_pos.price,
                        sell_stock.price,
                        gains,
                        gains * unit_sold,
                        buy_pos.currency,
                        sell_stock.metadata
                    );
                    sum_gains_loss = sum_gains_loss + (gains * unit_sold);
                    sum_shares_sold = sum_shares_sold + unit_sold;
                    if buy_pos.unit > 0.0 {
                        buy.push(buy_pos);
                    }
                }
                last_year = year;
            }
            info!("{}: {} {}", year, sum_shares_sold, sum_gains_loss);
            debug!("{:?}", buy.last());
            Self::overwrite_to_output_file(output_string.as_str());
        }
    }
}
