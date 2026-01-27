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
    test_positions: Arc<RwLock<Vec<Position>>>,
    news_symbols: Arc<RwLock<Vec<String>>>,  // NEW: Symbols to track for news
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
    let news = Arc::new(NewsAggregator::new());
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
        test_positions: Arc::new(RwLock::new(vec![])),
        news_symbols: Arc::new(RwLock::new(vec![
            "AAPL".to_string(),
            "GOOGL".to_string(),
            "BTC/USD".to_string(),
            "ETH/USD".to_string(),
        ])),
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
        .route("/test/generate", post(generate_test_data))
        .route("/test/clear", post(clear_test_data))  // NEW
        .layer(tower_http::cors::CorsLayer::permissive())
        .with_state(state);
    
    let addr = SocketAddr::from(([0, 0, 0, 0], 8080));
    info!("API server listening on http://localhost:8080");
    info!("Dashboard available at http://localhost:3000");
    
    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;
    
    Ok(())
}

async fn portfolio_tracking_loop(state: AppState) {
    let mut tick = interval(Duration::from_secs(30));
    
    loop {
        tick.tick().await;
        
        // Calculate current portfolio value
        let positions = match state.alpaca.get_positions().await {
            Ok(p) => p,
            Err(_) => vec![],
        };
        
        let positions_value: f64 = positions.iter()
            .map(|p| {
                let qty: f64 = p.qty.parse().unwrap_or(0.0);
                let price: f64 = p.current_price.parse().unwrap_or(0.0);
                qty * price
            })
            .sum();
        
        let account = match state.alpaca.get_account().await {
            Ok(acc) => acc,
            Err(_) => {
                tokio::time::sleep(Duration::from_secs(30)).await;
                continue;
            }
        };
        
        let cash: f64 = account.cash.parse().unwrap_or(100000.0);
        let total_value = cash + positions_value;
        
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
    let mut tick = interval(Duration::from_secs(90));
    
    // Expanded list of active stocks
    let symbols = vec![
        "AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", 
        "NVDA", "META", "NFLX", "AMD", "INTC",
        "PYPL", "ADBE", "CRM", "ORCL", "QCOM",
        "TXN", "AVGO", "CSCO", "ASML", "AMAT"
    ];
    
    loop {
        tick.tick().await;
        
        let trading_enabled = *state.trading_enabled.read().await;
        if !trading_enabled {
            continue;
        }
        
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
    
    // VERY AGGRESSIVE THRESHOLDS - trades will happen!
    if signal > 0.05 && !has_position {  // Lowered from 0.15 to 0.05 for more activity
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
    } else if signal < -0.05 && has_position {  // Lowered from -0.15 to -0.05 for more activity
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
            info!("⚪ {} - Signal {:.3} not strong enough to SELL (threshold: -0.05)", symbol, signal);
        } else {
            info!("⚪ {} - Signal {:.3} not strong enough to BUY (threshold: 0.05)", symbol, signal);
        }
        return Ok("neutral".to_string());
    }
    
    Ok("neutral".to_string())
}

async fn crypto_trading_loop(state: AppState) {
    let mut tick = interval(Duration::from_secs(120));
    let crypto_symbols = vec!["BTC/USD", "ETH/USD", "XRP/USD"];
    
    loop {
        tick.tick().await;
        let crypto_enabled = *state.crypto_trading_enabled.read().await;
        if !crypto_enabled { continue; }
        
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
    
    if signal > 0.10 && !has_position {  // Lowered from 0.25 to 0.10 for more activity
        info!("🟢 {} BUY SIGNAL ({:.3})", symbol, signal);
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
    } else if signal < -0.10 && has_position {  // Lowered from -0.25 to -0.10
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
                let asset_type = if p.symbol.contains("/") { "crypto" } else { "stock" };
                
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
    
    // Add test positions
    let test_positions = state.test_positions.read().await;
    all_positions.extend(test_positions.clone());
    
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
                .filter(|p| p.symbol.contains("/"))
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

async fn generate_test_data(State(state): State<AppState>) -> StatusCode {
    state.logger.info("Test", "🧪 Generating test data with LIVE prices...");
    
    let mut test_positions = state.test_positions.write().await;
    test_positions.clear();
    
    // ALL stock symbols we trade
    let stock_symbols = vec!["AAPL", "GOOGL", "MSFT", "TSLA", "AMZN", "NVDA", "META", "NFLX", 
                             "AMD", "INTC", "PYPL", "ADBE", "CRM", "ORCL", "QCOM", "TXN", 
                             "AVGO", "CSCO", "ASML", "AMAT"];
    
    // Fetch LIVE prices for stocks
    for symbol in &stock_symbols {
        match state.alpaca.get_latest_quote(symbol).await {
            Ok(current_price) => {
                let entry_multiplier = rand::random::<f64>() * 0.16 - 0.08; // -8% to +8%
                let entry_price = current_price / (1.0 + entry_multiplier);
                let quantity = (rand::random::<f64>() * 15.0 + 5.0).floor(); // 5-20 shares
                
                let pnl = (current_price - entry_price) * quantity;
                let pnl_percent = ((current_price - entry_price) / entry_price) * 100.0;
                let market_value = current_price * quantity;
                
                test_positions.push(Position {
                    symbol: symbol.to_string(),
                    quantity,
                    entry_price,
                    current_price,
                    pnl,
                    pnl_percent,
                    market_value,
                    asset_type: "stock".to_string(),
                });
                
                state.logger.info("Test", &format!("📈 {} @ ${:.2}", symbol, current_price));
            },
            Err(e) => {
                state.logger.warning("Test", &format!("⚠️ Failed to get price for {}: {}", symbol, e));
            }
        }
    }
    
    // ALL crypto symbols we trade
    let crypto_symbols = vec!["BTC/USD", "ETH/USD", "XRP/USD"];
    
    // Fetch LIVE prices for crypto
    for symbol in &crypto_symbols {
        match state.crypto.get_latest_crypto_price(symbol).await {
            Ok(current_price) => {
                let entry_multiplier = rand::random::<f64>() * 0.24 - 0.12; // -12% to +12%
                let entry_price = current_price / (1.0 + entry_multiplier);
                
                let quantity = if symbol.contains("BTC") {
                    rand::random::<f64>() * 0.2 + 0.05 // 0.05-0.25 BTC
                } else if symbol.contains("ETH") {
                    rand::random::<f64>() * 3.5 + 1.5 // 1.5-5.0 ETH
                } else {
                    (rand::random::<f64>() * 2000.0 + 1000.0).floor() // 1000-3000 XRP
                };
                
                let pnl = (current_price - entry_price) * quantity;
                let pnl_percent = ((current_price - entry_price) / entry_price) * 100.0;
                let market_value = current_price * quantity;
                
                test_positions.push(Position {
                    symbol: symbol.to_string(),
                    quantity,
                    entry_price,
                    current_price,
                    pnl,
                    pnl_percent,
                    market_value,
                    asset_type: "crypto".to_string(),
                });
                
                state.logger.info("Test", &format!("₿ {} @ ${:.2}", symbol, current_price));
            },
            Err(e) => {
                state.logger.warning("Test", &format!("⚠️ Failed to get price for {}: {}", symbol, e));
            }
        }
    }
    
    let total = test_positions.len();
    drop(test_positions);
    
    // Generate portfolio snapshots
    let base_time = Utc::now();
    let mut portfolio_history = state.portfolio_history.write().await;
    portfolio_history.clear();
    
    let initial = 100000.0;
    for i in 0..30 {
        let trend = (i as f64) * 100.0;
        let volatility = (rand::random::<f64>() * 1300.0) - 500.0;
        let total_value = initial + trend + volatility;
        let pos_pct = rand::random::<f64>() * 0.30 + 0.15;
        let positions_value = total_value * pos_pct;
        
        portfolio_history.push(PortfolioSnapshot {
            timestamp: (base_time - chrono::Duration::minutes(30 - i)).to_rfc3339(),
            total_value,
            cash: total_value - positions_value,
            positions_value,
        });
    }
    drop(portfolio_history);
    
    // Generate trade history
    let mut trade_history = state.trade_history.write().await;
    trade_history.clear();
    
    let all_symbols = vec!["AAPL", "GOOGL", "NVDA", "TSLA", "BTC/USD", "ETH/USD"];
    for i in 0..(rand::random::<usize>() % 7 + 8) {
        let symbol = all_symbols[rand::random::<usize>() % all_symbols.len()];
        let action = if i % 3 == 0 { "SELL" } else { "BUY" };
        let qty = if symbol.contains("/") {
            rand::random::<f64>() * 1.9 + 0.1
        } else {
            (rand::random::<f64>() * 15.0 + 5.0).floor()
        };
        let price = if symbol.contains("BTC") {
            rand::random::<f64>() * 20000.0 + 85000.0
        } else if symbol.contains("ETH") {
            rand::random::<f64>() * 1000.0 + 3000.0
        } else {
            rand::random::<f64>() * 350.0 + 150.0
        };
        let pnl = if action == "SELL" {
            qty * (rand::random::<f64>() * 200.0 - 50.0)
        } else {
            0.0
        };
        
        trade_history.push(TradeRecord {
            id: uuid::Uuid::new_v4().to_string(),
            timestamp: (base_time - chrono::Duration::minutes(30 - i as i64 * 2)).to_rfc3339(),
            symbol: symbol.to_string(),
            action: action.to_string(),
            quantity: qty,
            price,
            pnl,
        });
    }
    
    let trades = trade_history.len();
    drop(trade_history);
    
    state.logger.success("Test", &format!("✓ Generated {} positions (LIVE prices), {} trades", total, trades));
    StatusCode::OK
}


async fn clear_test_data(State(state): State<AppState>) -> StatusCode {
    state.logger.info("Test", "🧹 Clearing test data...");
    
    // Clear test positions
    let mut test_positions = state.test_positions.write().await;
    test_positions.clear();
    drop(test_positions);
    
    // Clear portfolio history (reset to initial)
    let mut portfolio_history = state.portfolio_history.write().await;
    portfolio_history.clear();
    portfolio_history.push(PortfolioSnapshot {
        timestamp: Utc::now().to_rfc3339(),
        total_value: 100000.0,
        cash: 100000.0,
        positions_value: 0.0,
    });
    drop(portfolio_history);
    
    // Clear trade history
    let mut trade_history = state.trade_history.write().await;
    trade_history.clear();
    drop(trade_history);
    
    state.logger.success("Test", "✓ Test data cleared successfully");
    
    StatusCode::OK
}