use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{interval, Duration};
use tracing::{info, error};
use crate::alpaca::AlpacaClient;

#[derive(Clone)]
pub struct NewsAggregator {
    sentiment_cache: Arc<DashMap<String, f64>>,
    alpaca: Arc<AlpacaClient>,
}

impl NewsAggregator {
    pub fn new(alpaca: Arc<AlpacaClient>) -> Self {
        Self {
            sentiment_cache: Arc::new(DashMap::new()),
            alpaca,
        }
    }

    pub async fn start(&self) {
        let mut tick = interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            tick.tick().await;
            
            // Update sentiment for commonly traded symbols
            let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
                             "BTC/USD", "ETH/USD"];
            
            for symbol in symbols {
                if let Err(e) = self.fetch_sentiment(symbol).await {
                    error!("Failed to fetch sentiment for {}: {}", symbol, e);
                }
            }
        }
    }

    async fn fetch_sentiment(&self, symbol: &str) -> Result<(), Box<dyn std::error::Error>> {
        info!("📰 Fetching Alpaca news sentiment for {}", symbol);
        
        // Use Alpaca News API (real NLP sentiment analysis)
        let sentiment = self.alpaca.get_news_sentiment(symbol).await?;
        
        self.sentiment_cache.insert(symbol.to_string(), sentiment);
        
        Ok(())
    }

    pub fn get_sentiment(&self, symbol: &str) -> f64 {
        self.sentiment_cache
            .get(symbol)
            .map(|v| *v)
            .unwrap_or(0.0) // Default to neutral if no data
    }
}
