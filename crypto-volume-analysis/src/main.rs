// Cargo.toml 不需要改，保持原来的依赖即可

use reqwest;
use serde::Deserialize;
use std::error::Error;
use tokio::time::{sleep, Duration};

#[derive(Deserialize, Debug)]
struct Ticker {
    symbol: String,
    #[serde(rename = "quoteVolume")]
    quote_volume: String,
}

#[derive(Deserialize, Debug)]
struct HistItem {
    #[serde(rename = "openInterest")]
    open_interest: String,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();

    // 1. 获取前 100（按 24h USDT 成交量）
    println!("正在获取 Binance 永续合约前 100（按 24h 成交量）...");
    let tickers: Vec<Ticker> = client
        .get("https://fapi.binance.com/fapi/v1/ticker/24hr")
        .send()
        .await?
        .json()
        .await?;

    let mut top: Vec<Ticker> = tickers
        .into_iter()
        .filter(|t| t.quote_volume != "0")
        .collect();

    top.sort_by(|a, b| {
        let va: f64 = a.quote_volume.parse().unwrap_or(0.0);
        let vb: f64 = b.quote_volume.parse().unwrap_or(0.0);
        vb.partial_cmp(&va).unwrap()
    });

    let top_symbols: Vec<String> = top.into_iter().take(100).map(|t| t.symbol).collect();

    // 2. 获取每个币种的 48 小时持仓变化（使用新接口）
    println!("正在扫描 100 个币种的 48 小时持仓变化（新接口）...");
    let mut results: Vec<(String, f64)> = vec![];

    for (i, symbol) in top_symbols.iter().enumerate() {
        let url = format!(
            "https://fapi.binance.com/futures/data/openInterestHist?symbol={}&period=1h&limit=49",
            symbol
        );

        let hist: Vec<HistItem> = match client.get(&url).send().await {
            Ok(resp) => match resp.json().await {
                Ok(data) => data,
                Err(e) => {
                    if i < 5 {
                        println!("  └─ {} 获取失败: {}", symbol, e); // 只打印前几个错误调试
                    }
                    vec![]
                }
            },
            Err(e) => {
                if i < 5 {
                    println!("  └─ {} 请求失败: {}", symbol, e);
                }
                vec![]
            }
        };

        if hist.len() >= 2 {
            let past_oi: f64 = hist[0].open_interest.parse().unwrap_or(0.0); // ≈48h 前
            let curr_oi: f64 = hist.last().unwrap().open_interest.parse().unwrap_or(0.0);

            if past_oi > 0.0 {
                let pct_change = (curr_oi / past_oi - 1.0) * 100.0;
                results.push((symbol.clone(), pct_change));
            }
        }

        // 防限流：每请求间隔 150ms（100 个请求大约 15 秒）
        sleep(Duration::from_millis(150)).await;
    }

    // 3. 排序并输出
    results.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    println!("\n=== 数字货币前 100（按 24h 成交量）48 小时持仓量增加百分比排序 ===");
    println!("排名 | 币种     | 48h 持仓变化 (%)");
    println!("-------------------------------------");
    for (i, (symbol, pct)) in results.iter().enumerate() {
        println!("{:>4} | {:<8} | {:>8.2}%", i + 1, symbol, pct);
    }

    if results.is_empty() {
        println!("依然没有数据？请把上面打印的错误信息发给我，我继续帮你调。");
    } else {
        println!("\n✅ 成功！最上面的是 48 小时持仓增加最多的币。");
    }

    Ok(())
}