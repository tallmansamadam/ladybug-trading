use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{interval, Duration};
use tracing::{info, error};

#[derive(Clone)]
pub struct NewsAggregator {
    sentiment_cache: Arc<DashMap<String, f64>>,
    client: reqwest::Client,
}

#[derive(Debug, Deserialize)]
struct RedditPost {
    data: RedditData,
}

#[derive(Debug, Deserialize)]
struct RedditData {
    children: Vec<RedditChild>,
}

#[derive(Debug, Deserialize)]
struct RedditChild {
    data: RedditPostData,
}

#[derive(Debug, Deserialize)]
struct RedditPostData {
    title: String,
    selftext: String,
    score: i32,
}

impl NewsAggregator {
    pub fn new() -> Self {
        Self {
            sentiment_cache: Arc::new(DashMap::new()),
            client: reqwest::Client::new(),
        }
    }

    pub async fn start(&self) {
        let mut tick = interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            tick.tick().await;
            
            let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN"];
            
            for symbol in symbols {
                if let Err(e) = self.fetch_sentiment(symbol).await {
                    error!("Failed to fetch sentiment for {}: {}", symbol, e);
                }
            }
        }
    }

    async fn fetch_sentiment(&self, symbol: &str) -> Result<()> {
        info!("Fetching news sentiment for {}", symbol);
        
        // Fetch from Reddit
        let sentiment = self.fetch_reddit_sentiment(symbol).await?;
        
        self.sentiment_cache.insert(symbol.to_string(), sentiment);
        
        Ok(())
    }

    async fn fetch_reddit_sentiment(&self, symbol: &str) -> Result<f64> {
        let url = format!(
            "https://www.reddit.com/r/wallstreetbets/search.json?q={}&sort=hot&limit=25",
            symbol
        );
        
        let response = self.client
            .get(&url)
            .header("User-Agent", "LadyBug Trading Bot 1.0")
            .send()
            .await?;

        if !response.status().is_success() {
            return Ok(0.0);
        }

        let posts: RedditPost = response.json().await?;
        
        // Simple sentiment: positive score = bullish
        let total_score: i32 = posts.data.children.iter()
            .map(|child| child.data.score)
            .sum();
        
        let sentiment = if posts.data.children.is_empty() {
            0.0
        } else {
            (total_score as f64 / posts.data.children.len() as f64) / 100.0
        };
        
        Ok(sentiment.clamp(-1.0, 1.0))
    }

    pub fn get_sentiment(&self, symbol: &str) -> f64 {
        self.sentiment_cache
            .get(symbol)
            .map(|s| *s)
            .unwrap_or(0.0)
    }
}