use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone)]
pub struct CryptoClient {
    client: Client,
    api_key: String,
    api_secret: String,
    base_url: String,
    data_url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoPosition {
    pub symbol: String,
    pub qty: String,
    pub avg_entry_price: String,
    pub current_price: String,
    pub unrealized_pl: String,
    pub asset_class: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CryptoBar {
    pub t: String,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: f64,
    pub vw: f64,
}

#[derive(Debug, Deserialize)]
struct CryptoBarsResponse {
    bars: std::collections::HashMap<String, Vec<CryptoBar>>,
}

#[derive(Debug, Serialize)]
pub struct CryptoOrderRequest {
    pub symbol: String,
    pub qty: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub time_in_force: String,
}

impl CryptoClient {
    pub fn new(api_key: String, api_secret: String, paper: bool) -> Self {
        let base_url = if paper {
            "https://paper-api.alpaca.markets/v2".to_string()
        } else {
            "https://api.alpaca.markets/v2".to_string()
        };
        
        Self {
            client: Client::new(),
            api_key,
            api_secret,
            base_url,
            data_url: "https://data.alpaca.markets/v1beta3".to_string(),
        }
    }

    pub async fn get_crypto_bars(&self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<CryptoBar>> {
        // Alpaca crypto symbols format: BTC/USD, ETH/USD, etc.
        let url = format!("{}/crypto/us/bars", self.data_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .query(&[
                ("symbols", symbol),
                ("timeframe", timeframe),
                ("limit", &limit.to_string()),
            ])
            .send()
            .await
            .context(format!("Failed to fetch crypto bars for {}", symbol))?;

        if !response.status().is_success() {
            tracing::warn!("Alpaca crypto data API error for {}", symbol);
            return Ok(vec![]);
        }

        let text = response.text().await?;
        
        let response_data: CryptoBarsResponse = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse crypto bars for {}: {}", symbol, e);
                return Ok(vec![]);
            }
        };
        
        // Get bars for the symbol
        if let Some(bars) = response_data.bars.get(symbol) {
            Ok(bars.clone())
        } else {
            Ok(vec![])
        }
    }

    pub async fn get_latest_crypto_price(&self, symbol: &str) -> Result<f64> {
        let url = format!("{}/crypto/us/latest/quotes", self.data_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .query(&[("symbols", symbol)])
            .send()
            .await
            .context(format!("Failed to fetch crypto quote for {}", symbol))?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to get latest crypto quote");
        }

        let text = response.text().await?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        
        // Try to get ask price
        if let Some(quotes) = value["quotes"].as_object() {
            if let Some(quote) = quotes.get(symbol) {
                if let Some(ask) = quote["ap"].as_f64() {
                    if ask > 0.0 {
                        return Ok(ask);
                    }
                }
                if let Some(bid) = quote["bp"].as_f64() {
                    if bid > 0.0 {
                        return Ok(bid);
                    }
                }
            }
        }
        
        // Fallback to latest bar
        let bars = self.get_crypto_bars(symbol, "1Min", 1).await?;
        if let Some(bar) = bars.first() {
            return Ok(bar.c);
        }
        
        anyhow::bail!("No crypto price data available")
    }

    pub async fn place_crypto_order(&self, request: CryptoOrderRequest) -> Result<serde_json::Value> {
        let url = format!("{}/orders", self.base_url);
        
        let response = self.client
            .post(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .json(&request)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Failed to place crypto order: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn close_crypto_position(&self, symbol: &str) -> Result<()> {
        let url = format!("{}/positions/{}", self.base_url, symbol);
        
        self.client
            .delete(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await?;

        Ok(())
    }
}
