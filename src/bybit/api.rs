use anyhow::Result;
use serde::Deserialize;
use tracing::{debug, error};

#[derive(Deserialize)]
struct TickerResponse {
    #[serde(rename = "retCode")]
    ret_code: i32,
    #[serde(rename = "retMsg")]
    ret_msg: String,
    result: TickerResult,
}

#[derive(Deserialize)]
struct TickerResult {
    list: Vec<TickerData>,
}

#[derive(Deserialize)]
struct TickerData {
    #[serde(rename = "fundingRate")]
    funding_rate: String,
}

pub struct BybitAPI {
    client: reqwest::Client,
}

impl BybitAPI {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get_funding_rate(&self, symbol: &str) -> Result<String> {
        let url = format!(
            "https://api.bybit.com/v5/market/tickers?category=linear&symbol={}",
            symbol
        );

        debug!("Requesting FR for symbol: {}", symbol);
        
        let response = self.client
            .get(&url)
            .send()
            .await?;

        if !response.status().is_success() {
            error!("API request failed with status: {}", response.status());
            anyhow::bail!("APIリクエストが失敗: status {}", response.status());
        }

        let response: TickerResponse = response.json().await?;

        if response.ret_code != 0 {
            error!("API returned error code {}: {}", response.ret_code, response.ret_msg);
            anyhow::bail!("APIエラー: {} (コード: {})", response.ret_msg, response.ret_code);
        }

        if let Some(ticker) = response.result.list.first() {
            match ticker.funding_rate.parse::<f64>() {
                Ok(fr) => {
                    Ok(format!("{:.4}", fr * 100.0))
                }
                Err(e) => {
                    error!("Failed to parse funding rate '{}': {}", ticker.funding_rate, e);
                    anyhow::bail!("Funding Rateのパースに失敗: {}", e)
                }
            }
        } else {
            error!("No ticker data found for symbol: {}", symbol);
            anyhow::bail!("ティッカーデータが見つかりません: {}", symbol)
        }
    }
}