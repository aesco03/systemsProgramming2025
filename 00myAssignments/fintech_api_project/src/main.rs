use serde::Deserialize;
use std::fs::OpenOptions;
use std::io::Write;
use std::thread;
use std::time::Duration;

// Trait
trait Pricing {
    fn fetch_price(&self) -> Result<f64, Box<dyn std::error::Error>>;
    fn save_to_file(&self, price: f64) -> Result<(), Box<dyn std::error::Error>>;
    fn name(&self) -> &'static str;
}

// Structs
struct Bitcoin;
struct Ethereum;
struct SP500;

// Bitcoin implementation
impl Pricing for Bitcoin {
    fn fetch_price(&self) -> Result<f64, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct BtcWrapper {
            usd: f64,
        }
        #[derive(Deserialize)]
        struct Root {
            bitcoin: BtcWrapper,
        }

        let url =
            "https://api.coingecko.com/api/v3/simple/price?ids=bitcoin&vs_currencies=usd";
        let mut resp = ureq::get(url).call()?;
        let data: Root = resp.body_mut().read_json()?;
        Ok(data.bitcoin.usd)
    }

    fn save_to_file(&self, price: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("btc_price.txt")?;
        writeln!(file, "{}", price)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Bitcoin"
    }
}

// Ethereum implementation
impl Pricing for Ethereum {
    fn fetch_price(&self) -> Result<f64, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct EthWrapper {
            usd: f64,
        }
        #[derive(Deserialize)]
        struct Root {
            ethereum: EthWrapper,
        }

        let url =
            "https://api.coingecko.com/api/v3/simple/price?ids=ethereum&vs_currencies=usd";
        let mut resp = ureq::get(url).call()?;
        let data: Root = resp.body_mut().read_json()?;
        Ok(data.ethereum.usd)
    }

    fn save_to_file(&self, price: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("eth_price.txt")?;
        writeln!(file, "{}", price)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "Ethereum"
    }
}

// S&P 500 implementation
impl Pricing for SP500 {
    fn fetch_price(&self) -> Result<f64, Box<dyn std::error::Error>> {
        #[derive(Deserialize)]
        struct Root {
            chart: Chart,
        }
        #[derive(Deserialize)]
        struct Chart {
            result: Option<Vec<ResultItem>>,
        }
        #[derive(Deserialize)]
        struct ResultItem {
            meta: Meta,
            indicators: Option<Indicators>,
        }
        #[derive(Deserialize)]
        #[allow(non_snake_case)]
        struct Meta {
            regularMarketPrice: Option<f64>,
        }
        #[derive(Deserialize)]
        struct Indicators {
            quote: Option<Vec<Quote>>,
        }
        #[derive(Deserialize)]
        struct Quote {
            close: Option<Vec<Option<f64>>>,
        }

        let url = "https://query2.finance.yahoo.com/v8/finance/chart/%5EGSPC";
        let mut resp = ureq::get(url).call()?;
        let root: Root = resp.body_mut().read_json()?;

        if let Some(results) = root.chart.result {
            if let Some(first) = results.first() {
                if let Some(p) = first.meta.regularMarketPrice {
                    return Ok(p);
                }
                if let Some(ind) = &first.indicators {
                    if let Some(quotes) = &ind.quote {
                        if let Some(q) = quotes.first() {
                            if let Some(closes) = &q.close {
                                if let Some(last) = closes.iter().rev().flatten().next() {
                                    return Ok(*last);
                                }
                            }
                        }
                    }
                }
            }
        }

        Err("Could not extract S&P 500 price from Yahoo Finance response".into())
    }

    fn save_to_file(&self, price: f64) -> Result<(), Box<dyn std::error::Error>> {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open("sp500_price.txt")?;
        writeln!(file, "{}", price)?;
        Ok(())
    }

    fn name(&self) -> &'static str {
        "S&P 500"
    }
}

fn main() {
    let assets: Vec<Box<dyn Pricing>> = vec![
        Box::new(Bitcoin),
        Box::new(Ethereum),
        Box::new(SP500),
    ];

    println!("Starting price fetch loop (every 10 seconds). Press Ctrl+C to stop.");

    loop {
        for asset in &assets {
            match asset.fetch_price() {
                Ok(p) => {
                    println!("{} price: {}", asset.name(), p);
                    if let Err(e) = asset.save_to_file(p) {
                        eprintln!("Failed to save {} price: {}", asset.name(), e);
                    }
                }
                Err(e) => eprintln!("Failed to fetch {} price: {}", asset.name(), e),
            }
        }

        thread::sleep(Duration::from_secs(10));
    }
}
