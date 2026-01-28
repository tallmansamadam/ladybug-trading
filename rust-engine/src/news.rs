use std::sync::Arc;
use dashmap::DashMap;
use tokio::time::{interval, Duration};
use tracing::{info, error, warn};
use crate::alpaca::AlpacaClient;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use futures_util::{StreamExt, SinkExt};
use serde_json::json;

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
        // Clone for the background task
        let sentiment_cache = self.sentiment_cache.clone();
        let alpaca = self.alpaca.clone();
        
        // Spawn REST API polling task (backup/historical)
        let rest_task = {
            let sentiment_cache = sentiment_cache.clone();
            let alpaca = alpaca.clone();
            tokio::spawn(async move {
                Self::rest_polling_task(sentiment_cache, alpaca).await;
            })
        };
        
        // Spawn WebSocket streaming task (real-time)
        let ws_task = {
            let sentiment_cache = sentiment_cache.clone();
            let alpaca = alpaca.clone();
            tokio::spawn(async move {
                Self::websocket_stream_task(sentiment_cache, alpaca).await;
            })
        };
        
        // Wait for both tasks (they run forever)
        let _ = tokio::join!(rest_task, ws_task);
    }
    
    /// REST API polling task - runs every 5 minutes for backup/historical data
    async fn rest_polling_task(
        sentiment_cache: Arc<DashMap<String, f64>>,
        alpaca: Arc<AlpacaClient>,
    ) {
        let mut tick = interval(Duration::from_secs(300)); // Every 5 minutes

        loop {
            tick.tick().await;
            
            // Update sentiment for commonly traded symbols
            let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
                             "META", "NFLX", "NVDA"];
            
            for symbol in symbols {
                match alpaca.get_news_sentiment(symbol).await {
                    Ok(sentiment) => {
                        sentiment_cache.insert(symbol.to_string(), sentiment);
                        info!("📰 REST: {} sentiment updated to {:.3}", symbol, sentiment);
                    }
                    Err(e) => {
                        error!("Failed to fetch REST sentiment for {}: {}", symbol, e);
                    }
                }
            }
        }
    }
    
    /// WebSocket streaming task - real-time news as it happens
    async fn websocket_stream_task(
        sentiment_cache: Arc<DashMap<String, f64>>,
        alpaca: Arc<AlpacaClient>,
    ) {
        loop {
            info!("🔌 Connecting to Alpaca News WebSocket...");
            
            match Self::run_websocket_stream(&sentiment_cache, &alpaca).await {
                Ok(_) => {
                    warn!("WebSocket stream ended normally, reconnecting...");
                }
                Err(e) => {
                    error!("WebSocket error: {}, reconnecting in 10s...", e);
                    tokio::time::sleep(Duration::from_secs(10)).await;
                }
            }
        }
    }
    
    async fn run_websocket_stream(
        sentiment_cache: &Arc<DashMap<String, f64>>,
        alpaca: &Arc<AlpacaClient>,
    ) -> Result<(), anyhow::Error> {
        // Connect to Alpaca News WebSocket
        let url = "wss://stream.data.alpaca.markets/v1beta1/news";
        let (ws_stream, _) = connect_async(url).await?;
        
        info!("✅ Connected to Alpaca News WebSocket");
        
        let (mut write, mut read) = ws_stream.split();
        
        // Authenticate
        let api_key = std::env::var("ALPACA_API_KEY")?;
        let api_secret = std::env::var("ALPACA_API_SECRET")?;
        
        let auth_msg = json!({
            "action": "auth",
            "key": api_key,
            "secret": api_secret
        });
        
        write.send(Message::Text(auth_msg.to_string())).await?;
        info!("🔑 Sent authentication");
        
        // Subscribe to news for our symbols
        let symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
                          "META", "NFLX", "NVDA", "GME", "PLTR"];
        
        let subscribe_msg = json!({
            "action": "subscribe",
            "news": symbols
        });
        
        write.send(Message::Text(subscribe_msg.to_string())).await?;
        info!("📡 Subscribed to news for {} symbols", symbols.len());
        
        // Process incoming news
        while let Some(message) = read.next().await {
            match message {
                Ok(Message::Text(text)) => {
                    if let Err(e) = Self::process_news_message(&text, sentiment_cache) {
                        warn!("Failed to process news message: {}", e);
                    }
                }
                Ok(Message::Close(_)) => {
                    info!("WebSocket closed by server");
                    break;
                }
                Ok(Message::Ping(data)) => {
                    write.send(Message::Pong(data)).await?;
                }
                Err(e) => {
                    error!("WebSocket receive error: {}", e);
                    break;
                }
                _ => {}
            }
        }
        
        Ok(())
    }
    
    fn process_news_message(
        text: &str,
        sentiment_cache: &Arc<DashMap<String, f64>>,
    ) -> Result<(), anyhow::Error> {
        let value: serde_json::Value = serde_json::from_str(text)?;
        
        // Check message type
        if let Some(msg_type) = value.get("T").and_then(|v| v.as_str()) {
            match msg_type {
                "success" => {
                    info!("✅ {}", value.get("msg").and_then(|v| v.as_str()).unwrap_or("Success"));
                }
                "subscription" => {
                    info!("📡 Subscription confirmed");
                }
                "n" => {
                    // News article!
                    if let Some(symbols) = value.get("symbols").and_then(|v| v.as_array()) {
                        let headline = value.get("headline")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Unknown");
                        
                        let sentiment_score = value.get("sentiment_score")
                            .and_then(|v| v.as_f64())
                            .unwrap_or(0.0);
                        
                        // Update sentiment for all mentioned symbols
                        for symbol in symbols {
                            if let Some(sym) = symbol.as_str() {
                                sentiment_cache.insert(sym.to_string(), sentiment_score);
                                info!("⚡ LIVE NEWS: {} - '{}' (sentiment: {:.3})", 
                                      sym, headline, sentiment_score);
                            }
                        }
                    }
                }
                "error" => {
                    let msg = value.get("msg").and_then(|v| v.as_str()).unwrap_or("Unknown error");
                    warn!("❌ WebSocket error: {}", msg);
                }
                _ => {
                    // Unknown message type, ignore
                }
            }
        }
        
        Ok(())
    }

    pub fn get_sentiment(&self, symbol: &str) -> f64 {
        self.sentiment_cache
            .get(symbol)
            .map(|v| *v)
            .unwrap_or(0.0) // Default to neutral if no data
    }
}
