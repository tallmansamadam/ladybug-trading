use anyhow::Result;
use tracing::{info, error, warn};
use axum::{
    routing::{get, post},
    Router, Json,
    extract::State,
    http::StatusCode,
};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{interval, Duration};
use std::env;
use chrono::Utc;

mod alpaca;
mod crypto;
mod news;
mod technical;
mod activity;

use alpaca::{AlpacaClient, OrderRequest};
use crypto::{CryptoClient, CryptoOrderRequest};
use news::NewsAggregator;
use technical::TechnicalAnalysis;
use activity::{ActivityLogger, LogLevel};

#[derive(Clone)]
struct AppState {
    alpaca: Arc<AlpacaClient>,
    crypto: Arc<CryptoClient>,
    news: Arc<NewsAggregator>,
    trading_enabled: Arc<RwLock<bool>>,
    crypto_trading_enabled: Arc<RwLock<bool>>,
    logger: Arc<ActivityLogger>,
    portfolio_history: Arc<RwLock<Vec<PortfolioSnapshot>>>,
    trade_history: Arc<RwLock<Vec<TradeRecord>>>,
    news_symbols: Arc<RwLock<Vec<String>>>,
    trading_mode: Arc<RwLock<TradingMode>>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Position {
    symbol: String,
    quantity: f64,
    entry_price: f64,
    current_price: f64,
    pnl: f64,
    pnl_percent: f64,
    market_value: f64,
    asset_type: String,  // "stock" or "crypto"
}

#[derive(Clone, Serialize, Deserialize, Debug, PartialEq)]
#[serde(rename_all = "PascalCase")]
enum TradingMode {
    Conservative,
    Volatile,
    Hybrid,
}

impl TradingMode {
    fn get_stocks(&self) -> Vec<&'static str> {
        match self {
            TradingMode::Conservative => vec![
                "AAPL", "GOOGL", "MSFT", "TSLA", "AMZN",
                "NVDA", "META", "NFLX", "AMD", "INTC",
                "PYPL", "ADBE", "CRM", "ORCL", "QCOM",
                "TXN", "AVGO", "CSCO", "ASML", "AMAT"
            ],
            TradingMode::Volatile => vec![
                "TSLA", "GME", "PLTR", "RIOT",
                "MARA", "MSTR", "COIN", "ROKU", "SNAP",
                "SQ", "SHOP", "ARKK", "UPST", "CRWD",
                "ZM", "UBER", "LYFT", "DKNG", "HOOD", "SOFI"
            ],
            TradingMode::Hybrid => vec![
                // 10 stable
                "AAPL", "GOOGL", "MSFT", "AMZN", "META",
                "NFLX", "ADBE", "CRM", "ORCL", "CSCO",
                // 10 volatile (NO DUPLICATES)
                "TSLA", "GME", "PLTR", "RIOT", "COIN",
                "MSTR", "SNAP", "ROKU", "MARA", "ARKK"
            ],
        }
    }

    fn get_crypto(&self) -> Vec<&'static str> {
        match self {
            TradingMode::Conservative => vec![
                "BTC/USD", "ETH/USD", "XRP/USD"
            ],
            TradingMode::Volatile => vec![
                "BTC/USD", "ETH/USD", "SOL/USD",
                "DOGE/USD", "AVAX/USD", "MATIC/USD"
            ],
            TradingMode::Hybrid => vec![
                "BTC/USD", "ETH/USD", "SOL/USD",
                "DOGE/USD", "AVAX/USD"
            ],
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct PortfolioSnapshot {
    timestamp: String,
    total_value: f64,
    cash: f64,
    positions_value: f64,
}

#[derive(Clone, Serialize, Deserialize)]
struct TradeRecord {
    id: String,
    timestamp: String,
    symbol: String,
    action: String,
    quantity: f64,
    price: f64,
    pnl: f64,
}

#[derive(Deserialize)]
struct ToggleRequest {
    enabled: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt().init();
    
    info!("🐞 LadyBug Trading Engine v0.2.0 - Stocks + Crypto");
    
    dotenv::dotenv().ok();
    
    let api_key = env::var("ALPACA_API_KEY").unwrap_or_default();
    let api_secret = env::var("ALPACA_API_SECRET").unwrap_or_default();
    
    let has_credentials = !api_key.is_empty() && !api_secret.is_empty();
    
    if !has_credentials {
        info!("⚠️  No Alpaca credentials found - running in demo mode");
    } else {
        info!("✓ Alpaca credentials loaded");
    }
    
    let alpaca = Arc::new(AlpacaClient::new(api_key.clone(), api_secret.clone(), true));
    let crypto = Arc::new(CryptoClient::new(api_key, api_secret, true));
    let news = Arc::new(NewsAggregator::new()); // Yahoo RSS + local sentiment
    let logger = Arc::new(ActivityLogger::new());
    
    logger.success("System", "LadyBug Trading Engine started");
    
    // Initialize with starting portfolio value
    let initial_snapshot = PortfolioSnapshot {
        timestamp: Utc::now().to_rfc3339(),
        total_value: 100000.0,
        cash: 100000.0,
        positions_value: 0.0,
    };
    
    let state = AppState {
        alpaca: alpaca.clone(),
        crypto: crypto.clone(),
        news: news.clone(),
        trading_enabled: Arc::new(RwLock::new(true)),  // AUTO-ENABLED
        crypto_trading_enabled: Arc::new(RwLock::new(true)),  // AUTO-ENABLED
        logger: logger.clone(),
        portfolio_history: Arc::new(RwLock::new(vec![initial_snapshot])),
        trade_history: Arc::new(RwLock::new(vec![])),
        news_symbols: Arc::new(RwLock::new(vec![
            "AAPL".to_string(),
            "GOOGL".to_string(),
            "BTC/USD".to_string(),
            "ETH/USD".to_string(),
        ])),
        trading_mode: Arc::new(RwLock::new(TradingMode::Hybrid)),
    };
    
    // Log startup status
    logger.success("System", "✅ Stock Trading AUTO-ENABLED");
    logger.success("System", "✅ Crypto Trading AUTO-ENABLED");
    
    // Start news aggregator
    let news_clone = news.clone();
    tokio::spawn(async move {
        news_clone.start().await;
    });
    
    // Start trading engine
    let state_clone = state.clone();
    let has_creds = has_credentials;
    tokio::spawn(async move {
        if has_creds {
            trading_loop(state_clone).await;
        } else {
            demo_loop(state_clone).await;
        }
    });
    
    // Start crypto trading engine
    let state_clone = state.clone();
    let has_creds = has_credentials;
    tokio::spawn(async move {
        if has_creds {
            crypto_trading_loop(state_clone).await;
        }
    });
    
    // Portfolio tracking loop
    let state_clone = state.clone();
    tokio::spawn(async move {
        portfolio_tracking_loop(state_clone).await;
    });
    
    let app = Router::new()
        .route("/", get(root))
        .route("/health", get(health))
        .route("/status", get(status))
        .route("/positions", get(get_positions))
        .route("/positions/crypto", get(get_crypto_positions))
        .route("/toggle", post(toggle_trading))
        .route("/toggle/crypto", post(toggle_crypto_trading))
        .route("/account", get(get_account))
        .route("/logs", get(get_logs))
        .route("/portfolio/history", get(get_portfolio_history))
        .route("/trades/history", get(get_trade_history))
        .route("/news/symbols", get(get_news_symbols))
        .route("/news/symbols", post(set_news_symbols))
        .route("/trading-mode", get(get_trading_mode))
        .route("/trading-mode", post(set_trading_mode))
        .route("/book-profit/:symbol", post(book_profit_single))
        .route("/book-all-profits", post(book_all_profits))
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state.clone());
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("API server listening on http://localhost:8080");
    info!("Dashboard available at http://localhost:3000");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    
    // Graceful shutdown handler
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    
    info!("🛑 Server shut down gracefully");
    Ok(())
}

async fn shutdown_signal() {
    let ctrl_c = async {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("🛑 Received Ctrl+C signal");
        },
        _ = terminate => {
            info!("🛑 Received terminate signal");
        },
    }
}

async fn portfolio_tracking_loop(state: AppState) {
    // RATE LIMIT FRIENDLY: Update portfolio every 60 seconds (plenty fast for tracking)
    let mut tick = interval(Duration::from_secs(60));
    
    loop {
        tick.tick().await;
        
        // Get REAL account data from Alpaca
        let account = match state.alpaca.get_account().await {
            Ok(acc) => acc,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(10)).await;
                continue;
            }
        };
        
        // Use Alpaca's ACTUAL portfolio_value (includes everything)
        let total_value: f64 = account.portfolio_value.parse().unwrap_or(100000.0);
        let cash: f64 = account.cash.parse().unwrap_or(100000.0);
        
        // Calculate positions value from total
        let positions_value = total_value - cash;
        
        let snapshot = PortfolioSnapshot {
            timestamp: Utc::now().to_rfc3339(),
            total_value,
            cash,
            positions_value,
        };
        
        let mut history = state.portfolio_history.write().await;
        history.push(snapshot);
        
        // Keep only last 100 snapshots
        if history.len() > 100 {
            history.remove(0);
        }
    }
}

async fn demo_loop(state: AppState) {
    let mut tick = interval(Duration::from_secs(30));
    
    loop {
        tick.tick().await;
        
        let trading_enabled = *state.trading_enabled.read().await;
        if !trading_enabled {
            continue;
        }
        
        state.logger.info("Demo", "Simulating market analysis...");
    }
}

async fn trading_loop(state: AppState) {
    // RATE LIMIT FRIENDLY: Slower cycles to avoid hitting Alpaca limits
    // Stock cycle: 5 minutes for sustainable trading
    let mut tick = interval(Duration::from_secs(300));
    
    loop {
        tick.tick().await;
        
        let trading_enabled = *state.trading_enabled.read().await;
        if !trading_enabled {
            continue;
        }
        
        // CHECK IF MARKET IS OPEN (9:30 AM - 4:00 PM ET, Monday-Friday)
        let now = Utc::now().with_timezone(&chrono_tz::America::New_York);
        let weekday = now.format("%u").to_string().parse::<u32>().unwrap(); // 1=Mon, 7=Sun
        let is_weekday = weekday <= 5; // Mon-Fri
        let market_open = chrono::NaiveTime::from_hms_opt(9, 30, 0).unwrap();
        let market_close = chrono::NaiveTime::from_hms_opt(16, 0, 0).unwrap();
        let current_time = now.time();
        
        let is_market_open = is_weekday && current_time >= market_open && current_time < market_close;
        
        if !is_market_open {
            // Market is closed - skip this cycle
            info!("📈 Market CLOSED - Skipping stock analysis (opens 9:30 AM ET Mon-Fri)");
            continue;
        }
        
        // Get symbols based on current trading mode
        let mode = state.trading_mode.read().await;
        let symbols = mode.get_stocks();
        
        info!("📈 Trading Mode: {:?} | Analyzing {} symbols", *mode, symbols.len());
        info!("📈 ========== STOCK TRADING CYCLE START ==========");
        state.logger.info("Stocks", "🔄 Starting market analysis cycle");
        
        let mut successful_analyses = 0;
        let mut failed_analyses = 0;
        let mut buy_signals = 0;
        let mut sell_signals = 0;
        let mut neutral_signals = 0;
        
        for symbol in &symbols {
            match process_stock(&state, symbol).await {
                Ok(result) => {
                    successful_analyses += 1;
                    match result.as_str() {
                        "buy" => buy_signals += 1,
                        "sell" => sell_signals += 1,
                        "neutral" => neutral_signals += 1,
                        _ => {}
                    }
                }
                Err(e) => {
                    failed_analyses += 1;
                    error!("❌ Error processing stock {}: {}", symbol, e);
                }
            }
            
            tokio::time::sleep(Duration::from_millis(300)).await;
        }
        
        info!("📊 Cycle Summary: {} analyzed | {} BUY signals | {} SELL signals | {} neutral | {} failed", 
              successful_analyses, buy_signals, sell_signals, neutral_signals, failed_analyses);
        
        state.logger.info("Stocks", &format!(
            "Cycle complete: {} stocks analyzed, {} buy signals, {} sell signals",
            successful_analyses, buy_signals, sell_signals
        ));
        
        info!("📈 ========== STOCK TRADING CYCLE END ==========\n");
    }
}

async fn process_stock(state: &AppState, symbol: &str) -> Result<String> {
    info!("🔍 Analyzing {}", symbol);
    
    // Get current live price
    let current_price = match state.alpaca.get_latest_quote(symbol).await {
        Ok(price) => {
            info!("💵 {} LIVE PRICE: ${:.2}", symbol, price);
            state.logger.info("Price", &format!("{}: ${:.2}", symbol, price));
            price
        },
        Err(e) => {
            warn!("⚠️  {} - Could not fetch live price: {}", symbol, e);
            state.logger.warning("Data", &format!("{} - Price fetch failed", symbol));
            return Err(e);
        }
    };
    
    // Get historical bars
    let bars = match state.alpaca.get_bars(symbol, "5Min", 50).await {
        Ok(bars) if bars.len() >= 20 => {
            info!("📊 {} - Got {} bars for analysis", symbol, bars.len());
            bars
        }
        Ok(bars) => {
            info!("⚠️  {} - Only {} bars (need 20+), skipping", symbol, bars.len());
            return Ok("insufficient_data".to_string());
        }
        Err(e) => {
            warn!("❌ {} - Failed to fetch bars: {}", symbol, e);
            return Err(e);
        }
    };
    
    let sentiment = state.news.get_sentiment(symbol);
    let signal = TechnicalAnalysis::generate_signal(&bars, sentiment);
    
    info!("📈 {} ANALYSIS: Signal={:.3}, Sentiment={:.3}", symbol, signal, sentiment);
    
    state.logger.analysis(
        &format!("${:.2} | Signal: {:.3} | Sentiment: {:.3}", current_price, signal, sentiment),
        symbol
    );
    
    let positions = match state.alpaca.get_positions().await {
        Ok(p) => p,
        Err(e) => {
            warn!("Failed to fetch positions: {}", e);
            return Err(e);
        }
    };
    
    let has_position = positions.iter().any(|p| p.symbol == symbol);
    
    // CONSERVATIVE THRESHOLDS - Smarter, fewer trades
    // BUY when signal > 0.15 (strong bullish)
    // SELL when signal < -0.15 (strong bearish) OR profit > 15%
    
    // PROFIT TAKING: Auto-sell if position has 15%+ profit
    if has_position {
        if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
            let entry: f64 = pos.avg_entry_price.parse().unwrap_or(0.0);
            let profit_percent = if entry > 0.0 {
                ((current_price - entry) / entry) * 100.0
            } else {
                0.0
            };
            
            if profit_percent >= 15.0 {
                info!("💰 {} PROFIT TAKING! {}% gain - SELLING", symbol, profit_percent.round());
                let pnl: f64 = pos.unrealized_pl.parse().unwrap_or(0.0);
                
                match state.alpaca.close_position(symbol).await {
                    Ok(_) => {
                        info!("✅ PROFIT BOOKED! {} - ${:.2} (+{}%)", symbol, pnl, profit_percent.round());
                        state.logger.trade(
                            LogLevel::Success,
                            &format!("💰 PROFIT TAKING ${:.2} (+{}%)", pnl, profit_percent.round()),
                            symbol
                        );
                        return Ok("profit_taking".to_string());
                    },
                    Err(e) => error!("❌ Profit taking failed: {}", e),
                }
            }
        }
    }
    
    if signal > 0.15 && !has_position {  // Raised from 0.05 to 0.15 for quality
        info!("🟢 {} STRONG BUY SIGNAL ({:.3}) - EXECUTING TRADE", symbol, signal);
        state.logger.signal(&format!("🟢 BUY signal ({:.3})", signal), symbol);
        
        let account = match state.alpaca.get_account().await {
            Ok(acc) => acc,
            Err(e) => {
                error!("Failed to get account: {}", e);
                return Err(e);
            }
        };
        
        let buying_power: f64 = account.buying_power.parse().unwrap_or(0.0);
        info!("💰 Available buying power: ${:.2}", buying_power);
        
        let position_size = (buying_power * 0.05).min(5000.0); // 5% of buying power, max $5k
        let qty = (position_size / current_price).floor() as i32;
        
        info!("📦 Calculated order: {} shares of {} at ${:.2} (${:.2} total)", 
              qty, symbol, current_price, qty as f64 * current_price);
        
        if qty > 0 {
            let order = OrderRequest {
                symbol: symbol.to_string(),
                qty: qty.to_string(),
                side: "buy".to_string(),
                order_type: "market".to_string(),
                time_in_force: "day".to_string(),
            };
            
            info!("📤 Submitting BUY order for {} shares of {}...", qty, symbol);
            
            match state.alpaca.place_order(order).await {
                Ok(order_response) => {
                    info!("✅ ORDER PLACED! {} - {} shares at ${:.2}", symbol, qty, current_price);
                    state.logger.trade(
                        LogLevel::Success,
                        &format!("✅ BUY {} shares at ${:.2} (Order ID: {})", qty, current_price, &order_response.id[..8]),
                        symbol
                    );
                    
                    let trade = TradeRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        symbol: symbol.to_string(),
                        action: "BUY".to_string(),
                        quantity: qty as f64,
                        price: current_price,
                        pnl: 0.0,
                    };
                    state.trade_history.write().await.push(trade);
                    
                    return Ok("buy".to_string());
                },
                Err(e) => {
                    error!("❌ ORDER FAILED for {}: {}", symbol, e);
                    state.logger.trade(LogLevel::Error, &format!("Failed: {}", e), symbol);
                }
            }
        } else {
            warn!("⚠️  {} - Quantity would be 0, skipping trade", symbol);
        }
    } else if signal < -0.15 && has_position {  // Raised from -0.05 to -0.15 for quality
        info!("🔴 {} STRONG SELL SIGNAL ({:.3}) - EXECUTING TRADE", symbol, signal);
        state.logger.signal(&format!("🔴 SELL signal ({:.3})", signal), symbol);
        
        if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
            let pnl: f64 = pos.unrealized_pl.parse().unwrap_or(0.0);
            info!("📤 Submitting SELL order to close {} position (P&L: ${:.2})...", symbol, pnl);
            
            match state.alpaca.close_position(symbol).await {
                Ok(_) => {
                    info!("✅ POSITION CLOSED! {} - P&L: ${:.2}", symbol, pnl);
                    state.logger.trade(
                        LogLevel::Success,
                        &format!("✅ SELL at ${:.2} | P&L: ${:.2}", current_price, pnl),
                        symbol
                    );
                    
                    let trade = TradeRecord {
                        id: uuid::Uuid::new_v4().to_string(),
                        timestamp: Utc::now().to_rfc3339(),
                        symbol: symbol.to_string(),
                        action: "SELL".to_string(),
                        quantity: pos.qty.parse().unwrap_or(0.0),
                        price: current_price,
                        pnl,
                    };
                    state.trade_history.write().await.push(trade);
                    
                    return Ok("sell".to_string());
                },
                Err(e) => {
                    error!("❌ CLOSE FAILED for {}: {}", symbol, e);
                    state.logger.trade(LogLevel::Error, &format!("Failed: {}", e), symbol);
                }
            }
        }
    } else {
        if has_position {
            info!("⚪ {} - Signal {:.3} not strong enough to SELL (threshold: -0.15)", symbol, signal);
        } else {
            info!("⚪ {} - Signal {:.3} not strong enough to BUY (threshold: 0.15)", symbol, signal);
        }
        return Ok("neutral".to_string());
    }
    
    Ok("neutral".to_string())
}

async fn crypto_trading_loop(state: AppState) {
    // RATE LIMIT FRIENDLY: 10 minutes for crypto (markets are 24/7, no rush)
    let mut tick = interval(Duration::from_secs(600));
    
    loop {
        tick.tick().await;
        let crypto_enabled = *state.crypto_trading_enabled.read().await;
        if !crypto_enabled { continue; }
        
        // Get crypto symbols based on current trading mode
        let mode = state.trading_mode.read().await;
        let crypto_symbols = mode.get_crypto();
        
        info!("₿ Trading Mode: {:?} | Analyzing {} crypto", *mode, crypto_symbols.len());
        info!("₿ ========== CRYPTO TRADING CYCLE START ==========");
        state.logger.info("Crypto", "🔄 Starting crypto market analysis");
        
        let mut successful_analyses = 0;
        let mut failed_analyses = 0;
        let mut buy_signals = 0;
        let mut sell_signals = 0;
        
        for symbol in &crypto_symbols {
            match process_crypto(&state, symbol).await {
                Ok(result) => {
                    successful_analyses += 1;
                    match result.as_str() {
                        "buy" => buy_signals += 1,
                        "sell" => sell_signals += 1,
                        _ => {}
                    }
                }
                Err(e) => {
                    failed_analyses += 1;
                    error!("❌ Error processing crypto {}: {}", symbol, e);
                }
            }
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        info!("₿ Crypto Summary: {} analyzed | {} BUY | {} SELL | {} failed", 
              successful_analyses, buy_signals, sell_signals, failed_analyses);
        state.logger.info("Crypto", &format!(
            "Cycle complete: {} cryptos analyzed, {} buy, {} sell",
            successful_analyses, buy_signals, sell_signals
        ));
        info!("₿ ========== CRYPTO TRADING CYCLE END ==========\n");
    }
}

async fn process_crypto(state: &AppState, symbol: &str) -> Result<String> {
    info!("₿ Analyzing {}", symbol);
    
    let current_price = match state.crypto.get_latest_crypto_price(symbol).await {
        Ok(price) => {
            info!("💰 {} LIVE PRICE: ${:.2}", symbol, price);
            state.logger.info("Crypto Price", &format!("{}: ${:.2}", symbol, price));
            price
        },
        Err(e) => {
            warn!("⚠️  {} - Could not fetch crypto price: {}", symbol, e);
            return Err(e);
        }
    };
    
    let bars = match state.crypto.get_crypto_bars(symbol, "5Min", 50).await {
        Ok(bars) if bars.len() >= 20 => {
            info!("📊 {} - Got {} crypto bars", symbol, bars.len());
            let converted_bars: Vec<alpaca::Bar> = bars.iter().map(|b| alpaca::Bar {
                t: b.t.clone(), o: b.o, h: b.h, l: b.l, c: b.c, v: b.v as i64,
            }).collect();
            converted_bars
        }
        Ok(bars) => {
            info!("⚠️  {} - Only {} bars, skipping", symbol, bars.len());
            return Ok("insufficient_data".to_string());
        }
        Err(e) => { warn!("❌ {} - Failed to fetch bars: {}", symbol, e); return Err(e); }
    };
    
    let sentiment = state.news.get_sentiment(symbol);
    let signal = TechnicalAnalysis::generate_signal(&bars, sentiment);
    info!("₿ {} ANALYSIS: Signal={:.3}, Sentiment={:.3}", symbol, signal, sentiment);
    state.logger.analysis(&format!("${:.2} | Signal: {:.3} | Sentiment: {:.3}", current_price, signal, sentiment), symbol);
    
    let positions = state.alpaca.get_positions().await.unwrap_or_default();
    let has_position = positions.iter().any(|p| p.symbol == symbol);
    
    // PROFIT TAKING for crypto: Auto-sell if 20%+ profit
    if has_position {
        if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
            let entry: f64 = pos.avg_entry_price.parse().unwrap_or(0.0);
            let profit_percent = if entry > 0.0 {
                ((current_price - entry) / entry) * 100.0
            } else {
                0.0
            };
            
            if profit_percent >= 20.0 {
                info!("💰 {} CRYPTO PROFIT TAKING! {}% gain", symbol, profit_percent.round());
                let pnl: f64 = pos.unrealized_pl.parse().unwrap_or(0.0);
                
                match state.crypto.close_crypto_position(symbol).await {
                    Ok(_) => {
                        info!("✅ CRYPTO PROFIT BOOKED! {} - ${:.2}", symbol, pnl);
                        state.logger.trade(
                            LogLevel::Success,
                            &format!("💰 CRYPTO PROFIT ${:.2} (+{}%)", pnl, profit_percent.round()),
                            symbol
                        );
                        return Ok("profit_taking".to_string());
                    },
                    Err(e) => error!("❌ Crypto profit taking failed: {}", e),
                }
            }
        }
    }
    
    if signal > 0.20 && !has_position {  // Raised from 0.10 to 0.20 for quality
        info!("🟢 {} STRONG CRYPTO BUY SIGNAL ({:.3})", symbol, signal);
        let account = state.alpaca.get_account().await?;
        let buying_power: f64 = account.buying_power.parse().unwrap_or(0.0);
        let position_size = (buying_power * 0.02).min(2000.0);
        let qty = position_size / current_price;
        
        if qty > 0.0 {
            let order = CryptoOrderRequest {
                symbol: symbol.to_string(), qty: format!("{:.6}", qty),
                side: "buy".to_string(), order_type: "market".to_string(),
                time_in_force: "gtc".to_string(),
            };
            
            match state.crypto.place_crypto_order(order).await {
                Ok(_) => {
                    info!("✅ CRYPTO ORDER PLACED! {}", symbol);
                    state.logger.trade(LogLevel::Success, &format!("✅ BUY {:.6} at ${:.2}", qty, current_price), symbol);
                    state.trade_history.write().await.push(TradeRecord {
                        id: uuid::Uuid::new_v4().to_string(), timestamp: Utc::now().to_rfc3339(),
                        symbol: symbol.to_string(), action: "BUY".to_string(),
                        quantity: qty, price: current_price, pnl: 0.0,
                    });
                    return Ok("buy".to_string());
                },
                Err(e) => error!("❌ CRYPTO ORDER FAILED: {}", e),
            }
        }
    } else if signal < -0.20 && has_position {  // Raised from -0.10 to -0.20
        if let Some(pos) = positions.iter().find(|p| p.symbol == symbol) {
            let pnl: f64 = pos.unrealized_pl.parse().unwrap_or(0.0);
            match state.crypto.close_crypto_position(symbol).await {
                Ok(_) => {
                    info!("✅ CRYPTO POSITION CLOSED! {} P&L: ${:.2}", symbol, pnl);
                    state.logger.trade(LogLevel::Success, &format!("✅ SELL at ${:.2} | P&L: ${:.2}", current_price, pnl), symbol);
                    state.trade_history.write().await.push(TradeRecord {
                        id: uuid::Uuid::new_v4().to_string(), timestamp: Utc::now().to_rfc3339(),
                        symbol: symbol.to_string(), action: "SELL".to_string(),
                        quantity: pos.qty.parse().unwrap_or(0.0), price: current_price, pnl,
                    });
                    return Ok("sell".to_string());
                },
                Err(e) => error!("❌ CLOSE FAILED: {}", e),
            }
        }
    }
    Ok("neutral".to_string())
}

async fn root() -> Json<serde_json::Value> {
    Json(json!({
        "name": "LadyBug Trading Engine",
        "version": "0.2.0",
        "features": ["stocks", "crypto"]
    }))
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "healthy" }))
}

async fn status(State(state): State<AppState>) -> Json<serde_json::Value> {
    let trading_enabled = *state.trading_enabled.read().await;
    let crypto_trading_enabled = *state.crypto_trading_enabled.read().await;
    
    let positions_count = match state.alpaca.get_positions().await {
        Ok(positions) => positions.len(),
        Err(_) => 0,
    };
    
    let crypto_positions_count = match state.alpaca.get_positions().await {
        Ok(positions) => positions.iter().filter(|p| p.symbol.contains("/")).count(),
        Err(_) => 0,
    };
    
    Json(json!({
        "running": true,
        "version": "0.2.0",
        "trading_enabled": trading_enabled,
        "crypto_trading_enabled": crypto_trading_enabled,
        "active_positions": positions_count,
        "crypto_positions": crypto_positions_count,
        "mode": "paper_trading"
    }))
}

async fn get_positions(State(state): State<AppState>) -> Json<Vec<Position>> {
    let mut all_positions = Vec::new();
    
    // Get real positions from Alpaca
    match state.alpaca.get_positions().await {
        Ok(positions) => {
            let real_positions: Vec<Position> = positions.iter().map(|p| {
                let qty = p.qty.parse().unwrap_or(0.0);
                let entry = p.avg_entry_price.parse().unwrap_or(0.0);
                let current = p.current_price.parse().unwrap_or(0.0);
                let pnl = p.unrealized_pl.parse().unwrap_or(0.0);
                let market_value = qty * current;
                let pnl_percent = if entry > 0.0 { ((current - entry) / entry) * 100.0 } else { 0.0 };
                
                // Detect crypto: contains "/" OR ends with "USD" (Alpaca format)
                let is_crypto = p.symbol.contains("/") || 
                               p.symbol.ends_with("USD") && 
                               !p.symbol.starts_with("USD") &&
                               p.symbol.len() > 3;
                let asset_type = if is_crypto { "crypto" } else { "stock" };
                
                Position {
                    symbol: p.symbol.clone(),
                    quantity: qty,
                    entry_price: entry,
                    current_price: current,
                    pnl,
                    pnl_percent,
                    market_value,
                    asset_type: asset_type.to_string(),
                }
            }).collect();
            all_positions.extend(real_positions);
        },
        Err(_) => {}
    }
    
    // ONLY REAL ALPACA POSITIONS - NO TEST DATA
    Json(all_positions)
}

async fn get_account(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    match state.alpaca.get_account().await {
        Ok(account) => Ok(Json(json!({
            "buying_power": account.buying_power,
            "cash": account.cash,
            "portfolio_value": account.portfolio_value,
        }))),
        Err(_) => Err(StatusCode::INTERNAL_SERVER_ERROR),
    }
}

async fn get_logs(State(state): State<AppState>) -> Json<Vec<activity::ActivityLog>> {
    Json(state.logger.get_logs())
}

#[derive(Deserialize)]
struct NewsSymbolsRequest {
    symbols: Vec<String>,
}

async fn get_news_symbols(State(state): State<AppState>) -> Json<Vec<String>> {
    let symbols = state.news_symbols.read().await;
    Json(symbols.clone())
}

async fn set_news_symbols(
    State(state): State<AppState>,
    Json(payload): Json<NewsSymbolsRequest>,
) -> StatusCode {
    let mut symbols = state.news_symbols.write().await;
    *symbols = payload.symbols.clone();
    state.logger.info("News", &format!("Updated news tracking: {:?}", payload.symbols));
    StatusCode::OK
}

async fn get_portfolio_history(State(state): State<AppState>) -> Json<Vec<PortfolioSnapshot>> {
    let history = state.portfolio_history.read().await;
    Json(history.clone())
}

async fn get_trade_history(State(state): State<AppState>) -> Json<Vec<TradeRecord>> {
    let trades = state.trade_history.read().await;
    Json(trades.clone())
}

async fn toggle_trading(
    State(state): State<AppState>,
    Json(payload): Json<ToggleRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut trading_enabled = state.trading_enabled.write().await;
    *trading_enabled = payload.enabled;
    
    if payload.enabled {
        state.logger.success("System", "Trading ENABLED ✅");
    } else {
        state.logger.warning("System", "Trading DISABLED ❌");
    }
    
    info!("Trading {}", if payload.enabled { "ENABLED ✅" } else { "DISABLED ❌" });
    Ok(StatusCode::OK)
}

async fn toggle_crypto_trading(
    State(state): State<AppState>,
    Json(payload): Json<ToggleRequest>,
) -> Result<StatusCode, StatusCode> {
    let mut crypto_trading_enabled = state.crypto_trading_enabled.write().await;
    *crypto_trading_enabled = payload.enabled;
    
    if payload.enabled {
        state.logger.success("Crypto", "Crypto Trading ENABLED ₿✅");
    } else {
        state.logger.warning("Crypto", "Crypto Trading DISABLED ❌");
    }
    
    info!("Crypto Trading {}", if payload.enabled { "ENABLED ₿✅" } else { "DISABLED ❌" });
    Ok(StatusCode::OK)
}

async fn get_crypto_positions(State(state): State<AppState>) -> Json<Vec<Position>> {
    match state.alpaca.get_positions().await {
        Ok(positions) => {
            let crypto_positions: Vec<Position> = positions.iter()
                .filter(|p| {
                    // Crypto: contains "/" OR ends with "USD" (like BTCUSD, ETHUSD)
                    p.symbol.contains("/") || 
                    (p.symbol.ends_with("USD") && 
                     !p.symbol.starts_with("USD") &&
                     p.symbol.len() > 3)
                })
                .map(|p| {
                    let qty = p.qty.parse().unwrap_or(0.0);
                    let entry = p.avg_entry_price.parse().unwrap_or(0.0);
                    let current = p.current_price.parse().unwrap_or(0.0);
                    let pnl = p.unrealized_pl.parse().unwrap_or(0.0);
                    let market_value = qty * current;
                    let pnl_percent = if entry > 0.0 { ((current - entry) / entry) * 100.0 } else { 0.0 };
                    
                    Position {
                        symbol: p.symbol.clone(),
                        quantity: qty,
                        entry_price: entry,
                        current_price: current,
                        pnl,
                        pnl_percent,
                        market_value,
                        asset_type: "crypto".to_string(),
                    }
                }).collect();
            Json(crypto_positions)
        },
        Err(_) => Json(vec![]),
    }
}

// TEST DATA FUNCTIONS REMOVED - USING ONLY REAL ALPACA PAPER TRADING


// Get current trading mode
async fn get_trading_mode(State(state): State<AppState>) -> Json<TradingMode> {
    let mode = state.trading_mode.read().await;
    Json(mode.clone())
}

// Set trading mode
#[derive(Deserialize)]
struct TradingModeRequest {
    mode: TradingMode,
}

async fn set_trading_mode(
    State(state): State<AppState>,
    Json(req): Json<TradingModeRequest>,
) -> StatusCode {
    let mut mode = state.trading_mode.write().await;
    *mode = req.mode.clone();
    drop(mode);
    
    state.logger.success("Config", &format!("Trading mode set to: {:?}", req.mode));
    info!("🎯 Trading mode changed to: {:?}", req.mode);
    
    StatusCode::OK
}

// Book profit for a single position
async fn book_profit_single(
    State(state): State<AppState>,
    axum::extract::Path(symbol): axum::extract::Path<String>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("💰 Manual profit booking requested for {}", symbol);
    
    // Get position info BEFORE closing
    let position_info = match state.alpaca.get_positions().await {
        Ok(positions) => {
            positions.iter()
                .find(|p| p.symbol == symbol)
                .map(|p| {
                    let qty = p.qty.parse().unwrap_or(0.0);
                    let entry = p.avg_entry_price.parse().unwrap_or(0.0);
                    let current = p.current_price.parse().unwrap_or(0.0);
                    let pnl = p.unrealized_pl.parse().unwrap_or(0.0);
                    (qty, entry, current, pnl)
                })
        },
        Err(_) => None,
    };
    
    if position_info.is_none() {
        error!("❌ Position not found: {}", symbol);
        return Err(StatusCode::NOT_FOUND);
    }
    
    let (qty, _entry_price, current_price, pnl) = position_info.unwrap();
    
    // Check if it's crypto or stock
    let is_crypto = symbol.contains("/") || 
                   (symbol.ends_with("USD") && 
                    !symbol.starts_with("USD") &&
                    symbol.len() > 3);
    
    let result = if is_crypto {
        state.crypto.close_crypto_position(&symbol).await
    } else {
        state.alpaca.close_position(&symbol).await
    };
    
    match result {
        Ok(_) => {
            // Record the trade in history
            state.trade_history.write().await.push(TradeRecord {
                id: uuid::Uuid::new_v4().to_string(),
                timestamp: Utc::now().to_rfc3339(),
                symbol: symbol.clone(),
                action: "SELL".to_string(),
                quantity: qty,
                price: current_price,
                pnl,
            });
            
            state.logger.success(
                "Manual Profit", 
                &format!("💰 {} position closed by user - P&L: ${:.2}", symbol, pnl)
            );
            info!("✅ Manual profit booked: {} - ${:.2}", symbol, pnl);
            
            Ok(Json(json!({
                "success": true,
                "symbol": symbol,
                "pnl": pnl,
                "message": "Position closed successfully"
            })))
        },
        Err(e) => {
            error!("❌ Failed to book profit for {}: {}", symbol, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

// Book profits for ALL positions
async fn book_all_profits(State(state): State<AppState>) -> Result<Json<serde_json::Value>, StatusCode> {
    info!("💰💰💰 Manual profit booking requested for ALL positions");
    
    let mut closed_count = 0;
    let mut failed_count = 0;
    let mut closed_symbols = Vec::new();
    let mut total_pnl = 0.0;
    
    // Get all positions
    match state.alpaca.get_positions().await {
        Ok(positions) => {
            for pos in positions {
                let qty = pos.qty.parse().unwrap_or(0.0);
                let _entry = pos.avg_entry_price.parse().unwrap_or(0.0);
                let current = pos.current_price.parse().unwrap_or(0.0);
                let pnl = pos.unrealized_pl.parse().unwrap_or(0.0);
                
                let is_crypto = pos.symbol.contains("/") || 
                               (pos.symbol.ends_with("USD") && 
                                !pos.symbol.starts_with("USD") &&
                                pos.symbol.len() > 3);
                
                let result = if is_crypto {
                    state.crypto.close_crypto_position(&pos.symbol).await
                } else {
                    state.alpaca.close_position(&pos.symbol).await
                };
                
                match result {
                    Ok(_) => {
                        // Record trade in history
                        state.trade_history.write().await.push(TradeRecord {
                            id: uuid::Uuid::new_v4().to_string(),
                            timestamp: Utc::now().to_rfc3339(),
                            symbol: pos.symbol.clone(),
                            action: "SELL".to_string(),
                            quantity: qty,
                            price: current,
                            pnl,
                        });
                        
                        closed_count += 1;
                        closed_symbols.push(pos.symbol.clone());
                        total_pnl += pnl;
                        info!("✅ Closed {} - P&L: ${:.2}", pos.symbol, pnl);
                    },
                    Err(e) => {
                        failed_count += 1;
                        error!("❌ Failed to close {}: {}", pos.symbol, e);
                    }
                }
                
                // Small delay between orders
                tokio::time::sleep(Duration::from_millis(200)).await;
            }
        },
        Err(e) => {
            error!("❌ Failed to get positions: {}", e);
            return Err(StatusCode::INTERNAL_SERVER_ERROR);
        }
    }
    
    state.logger.success(
        "Manual Profit", 
        &format!("💰 Closed {} positions - Total P&L: ${:.2}", closed_count, total_pnl)
    );
    
    Ok(Json(json!({
        "success": true,
        "closed_count": closed_count,
        "failed_count": failed_count,
        "total_pnl": total_pnl,
        "closed_symbols": closed_symbols
    })))
}