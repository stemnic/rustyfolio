use crate::portfolio::{Action, Portfolio, Stock};
use chrono::Datelike;
use log::{debug, info};
pub struct TaxCalculatorService {}

impl TaxCalculatorService {
    pub fn fifo_calculation(portfolio: &Portfolio) {
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
                        "{} {} \t {} {} \t {} {}",
                        sell_stock.date,
                        unit_sold,
                        g_lstring,
                        gains,
                        gains * unit_sold,
                        buy_pos.currency
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
            debug!("{:?}", buy.last())
        }
    }
}
