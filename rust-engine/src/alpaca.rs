use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use reqwest::Client;

#[derive(Clone)]
pub struct AlpacaClient {
    client: Client,
    api_key: String,
    api_secret: String,
    base_url: String,
    data_url: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    pub buying_power: String,
    pub cash: String,
    pub portfolio_value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Position {
    pub symbol: String,
    pub qty: String,
    pub avg_entry_price: String,
    pub current_price: String,
    pub unrealized_pl: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Order {
    pub id: String,
    pub symbol: String,
    pub qty: String,
    pub side: String,
    pub order_type: String,
    pub status: String,
}

#[derive(Debug, Serialize)]
pub struct OrderRequest {
    pub symbol: String,
    pub qty: String,
    pub side: String,
    #[serde(rename = "type")]
    pub order_type: String,
    pub time_in_force: String,
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct NewsArticle {
    pub headline: String,
    pub summary: String,
    #[serde(default)]
    pub sentiment: String,
    #[serde(default)]
    pub sentiment_score: f64,
}

#[derive(Debug, Deserialize)]
struct NewsResponse {
    pub news: Vec<NewsArticle>,
}

#[derive(Debug, Clone, Deserialize)]
#[allow(dead_code)]
pub struct Bar {
    pub t: String,
    pub o: f64,
    pub h: f64,
    pub l: f64,
    pub c: f64,
    pub v: i64,
}

#[derive(Debug, Deserialize)]
struct BarsResponse {
    #[serde(default)]
    bars: Vec<Bar>,
}

impl AlpacaClient {
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
            data_url: "https://data.alpaca.markets/v2".to_string(),
        }
    }

    pub async fn get_account(&self) -> Result<Account> {
        let url = format!("{}/account", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await
            .context("Failed to get account")?;

        if !response.status().is_success() {
            let error_text = response.text().await?;
            anyhow::bail!("Alpaca API error: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_positions(&self) -> Result<Vec<Position>> {
        let url = format!("{}/positions", self.base_url);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(vec![]);
        }

        Ok(response.json().await?)
    }

    pub async fn place_order(&self, request: OrderRequest) -> Result<Order> {
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
            anyhow::bail!("Failed to place order: {}", error_text);
        }

        Ok(response.json().await?)
    }

    pub async fn get_bars(&self, symbol: &str, timeframe: &str, limit: u32) -> Result<Vec<Bar>> {
        let url = format!("{}/stocks/{}/bars", self.data_url, symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .query(&[
                ("timeframe", timeframe),
                ("limit", &limit.to_string()),
                ("adjustment", "raw"),
                ("feed", "iex")
            ])
            .send()
            .await
            .context(format!("Failed to fetch bars for {}", symbol))?;

        if !response.status().is_success() {
            tracing::warn!("Alpaca data API error for {}", symbol);
            return Ok(vec![]);
        }

        let text = response.text().await?;
        
        // Debug: Log actual response for troubleshooting
        if text.contains("null") {
            tracing::debug!("Alpaca returned null bars for {}: {}", symbol, text);
        }
        
        let response_data: BarsResponse = match serde_json::from_str(&text) {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse bars for {} - Response: {} - Error: {}", symbol, text, e);
                return Ok(vec![]);
            }
        };
        
        Ok(response_data.bars)
    }

    pub async fn get_latest_quote(&self, symbol: &str) -> Result<f64> {
        // CRITICAL: Use latest TRADE price, not ask/bid which can be fake
        let url = format!("{}/stocks/{}/trades/latest", self.data_url, symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .query(&[("feed", "iex")])
            .send()
            .await
            .context(format!("Failed to fetch latest trade for {}", symbol))?;

        if !response.status().is_success() {
            tracing::warn!("Failed to get latest trade for {}, falling back to bars", symbol);
            // Fallback to latest bar close price
            let bars = self.get_bars(symbol, "1Min", 1).await?;
            if let Some(bar) = bars.first() {
                return Ok(bar.c);
            }
            anyhow::bail!("No price data available for {}", symbol);
        }

        let text = response.text().await?;
        let value: serde_json::Value = serde_json::from_str(&text)?;
        
        // Get actual trade price (not ask/bid which can be manipulated)
        if let Some(price) = value["trade"]["p"].as_f64() {
            if price > 0.0 {
                tracing::info!("✅ {} REAL TRADE PRICE: ${:.2}", symbol, price);
                return Ok(price);
            }
        }
        
        // Fallback to latest bar
        tracing::warn!("No trade price for {}, using latest bar", symbol);
        let bars = self.get_bars(symbol, "1Min", 1).await?;
        if let Some(bar) = bars.first() {
            return Ok(bar.c);
        }
        
        anyhow::bail!("No price data available for {}", symbol)
    }

    pub async fn close_position(&self, symbol: &str) -> Result<()> {
        let url = format!("{}/positions/{}", self.base_url, symbol);
        
        self.client
            .delete(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .send()
            .await?;

        Ok(())
    }

    pub async fn get_news_sentiment(&self, symbol: &str) -> Result<f64> {
        // News API uses data URL, not base_url!
        let url = format!("{}/v1beta1/news", self.data_url);
        
        tracing::debug!("📰 News API URL: {}", url);
        tracing::debug!("📰 Requesting news for: {}", symbol);
        
        let response = self.client
            .get(&url)
            .header("APCA-API-KEY-ID", &self.api_key)
            .header("APCA-API-SECRET-KEY", &self.api_secret)
            .query(&[
                ("symbols", symbol),
                ("limit", "10"),
                ("sort", "desc"),
            ])
            .send()
            .await
            .context(format!("Failed to fetch news for {}", symbol))?;

        if !response.status().is_success() {
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            tracing::warn!("Alpaca news API error for {}: {} - Response: {}", 
                          symbol, status, body);
            return Ok(0.0); // Default to neutral
        }

        let news_response: NewsResponse = match response.json().await {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!("Failed to parse news for {}: {}", symbol, e);
                return Ok(0.0);
            }
        };

        if news_response.news.is_empty() {
            tracing::debug!("No news articles found for {}", symbol);
            return Ok(0.0);
        }

        // Average the sentiment scores from recent articles
        let total_sentiment: f64 = news_response.news.iter()
            .map(|article| article.sentiment_score)
            .sum();
        
        let avg_sentiment = total_sentiment / news_response.news.len() as f64;
        
        tracing::info!("📰 {} NEWS: {} articles, avg sentiment: {:.3}", 
                      symbol, news_response.news.len(), avg_sentiment);
        
        Ok(avg_sentiment.clamp(-1.0, 1.0))
    }
}