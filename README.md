# Rustyfolio
Rustyfolio is a command-line tool for tracking your investment portfolio and calculating realized gains/losses using the FIFO (First-In, First-Out) method.
Mainly created to do tax accounting for ETrade Stock Plan positions with RSUs and ESPPs

## How to get started
With the default rust toolchain installed you can simply clone the repo and run
```bash
cargo run
```

This will bring up the following menu you can use for importing your statements and run your FIFO tax calculation
```
Welcome to rustyfolio! What do you want todo?

    1. Show Position
    2. Import statement
    3. Run Fifo calc
    9. Exit
```

FIFO calc will generate a csv file in the output subfolder which you can then import into excel. If you only need the total value of gain or loss in a tax year the program will output this directly.

## Etrade - At work
- BenefitHistory.xlsx:  At work -> My Account -> Benefit History -> Download -> Download Expanded
- G&L_Expanded.xlsx:    At work -> My Account -> Gains & Losses -> (Change Tax Year) -> Download -> Download Expanded
