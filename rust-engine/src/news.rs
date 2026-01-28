use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct NewsAggregator {
    sentiment_cache: Arc<DashMap<String, f64>>,
    client: reqwest::Client,
    sentiment_service_url: String,
}

#[derive(Debug, Deserialize)]
struct SentimentResponse {
    score: f64,
    sentiment: String,
    confidence: f64,
}

impl NewsAggregator {
    pub fn new() -> Self {
        Self {
            sentiment_cache: Arc::new(DashMap::new()),
            client: reqwest::Client::new(),
            sentiment_service_url: "http://localhost:5000".to_string(),
        }
    }

    pub async fn start(&self) {
        info!("📰 Starting News Aggregator with Yahoo RSS + Local FinBERT");
        
        // Check if sentiment service is running
        match self.check_sentiment_service().await {
            Ok(()) => info!("✅ FinBERT sentiment service is running"),
            Err(e) => {
                error!("❌ FinBERT sentiment service not available: {}", e);
                error!("💡 Start it with: cd sentiment-service && python sentiment_server.py");
                return;
            }
        }
        
        let mut tick = interval(Duration::from_secs(300)); // Every 5 minutes - fresh sentiment without spam

        loop {
            tick.tick().await;
            
            // Symbols to track
            let symbols = vec![
                "AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
                "META", "NFLX", "NVDA", "GME", "PLTR",
                "RIOT", "COIN", "MSTR"
            ];
            
            for symbol in symbols {
                if let Err(e) = self.fetch_and_analyze_news(symbol).await {
                    warn!("Failed to fetch news for {}: {}", symbol, e);
                }
                
                // Small delay to avoid hammering Yahoo
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
        }
    }
    
    async fn check_sentiment_service(&self) -> Result<(), Box<dyn std::error::Error>> {
        let url = format!("{}/health", self.sentiment_service_url);
        let response = self.client.get(&url).send().await?;
        
        if response.status().is_success() {
            Ok(())
        } else {
            Err("Sentiment service not healthy".into())
        }
    }
    
    async fn fetch_and_analyze_news(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        // Fetch Yahoo Finance RSS feed for symbol
        let rss_url = format!("https://finance.yahoo.com/rss/headline?s={}", symbol);
        
        let response = self.client
            .get(&rss_url)
            .header("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64)")
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Yahoo RSS returned {}", response.status()).into());
        }
        
        let rss_content = response.text().await?;
        
        // Parse RSS feed
        let channel = rss::Channel::read_from(rss_content.as_bytes())?;
        
        if channel.items().is_empty() {
            info!("📰 No news found for {}", symbol);
            return Ok(());
        }
        
        // Get the latest 5 headlines
        let headlines: Vec<String> = channel.items()
            .iter()
            .take(5)
            .filter_map(|item| item.title().map(|t| t.to_string()))
            .collect();
        
        if headlines.is_empty() {
            return Ok(());
        }
        
        info!("📰 {} - Found {} headlines", symbol, headlines.len());
        
        // Analyze sentiment of all headlines
        let sentiments = self.analyze_batch_sentiment(&headlines).await?;
        
        // Average the sentiment scores
        if !sentiments.is_empty() {
            let avg_sentiment: f64 = sentiments.iter().sum::<f64>() / sentiments.len() as f64;
            
            self.sentiment_cache.insert(symbol.to_string(), avg_sentiment);
            
            info!("🤖 {} - Sentiment: {:.3} (from {} headlines)", 
                  symbol, avg_sentiment, sentiments.len());
        }
        
        Ok(())
    }
    
    async fn analyze_batch_sentiment(&self, texts: &[String]) -> Result<Vec<f64>, Box<dyn std::error::Error>> {
        let url = format!("{}/batch", self.sentiment_service_url);
        
        #[derive(Serialize)]
        struct BatchRequest {
            texts: Vec<String>,
        }
        
        #[derive(Deserialize)]
        struct BatchResponse {
            results: Vec<SentimentResponse>,
        }
        
        let request = BatchRequest {
            texts: texts.to_vec(),
        };
        
        let response = self.client
            .post(&url)
            .json(&request)
            .send()
            .await?;
        
        if !response.status().is_success() {
            return Err(format!("Sentiment service returned {}", response.status()).into());
        }
        
        let batch_response: BatchResponse = response.json().await?;
        
        let scores: Vec<f64> = batch_response.results
            .iter()
            .map(|r| r.score)
            .collect();
        
        Ok(scores)
    }

    pub fn get_sentiment(&self, symbol: &str) -> f64 {
        self.sentiment_cache
            .get(symbol)
            .map(|v| *v)
            .unwrap_or(0.0) // Default to neutral if no data
    }
}
