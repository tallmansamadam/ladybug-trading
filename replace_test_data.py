import sys

# Read the file
with open(r'C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine\src\main.rs', 'r', encoding='utf-8') as f:
    content = f.read()

# The new function
new_function = '''async fn generate_test_data(State(state): State<AppState>) -> StatusCode {
    state.logger.info("Test", "🧪 Generating randomized test data...");
    
    let mut rng = rand::thread_rng();
    let mut test_positions = state.test_positions.write().await;
    test_positions.clear();
    
    let stocks = vec![
        ("AAPL", 150.0, 200.0, 10.0, 25.0),
        ("GOOGL", 120.0, 170.0, 5.0, 15.0),
        ("NVDA", 400.0, 600.0, 8.0, 30.0),
        ("TSLA", 180.0, 280.0, 8.0, 20.0),
        ("MSFT", 350.0, 450.0, 10.0, 20.0),
        ("AMZN", 140.0, 180.0, 8.0, 15.0),
    ];
    
    let num_stocks = rng.gen_range(4..=6);
    for (symbol, min_p, max_p, min_q, max_q) in stocks.iter().take(num_stocks) {
        let entry = rng.gen_range(*min_p..*max_p);
        let change_pct = rng.gen_range(-8.0..12.0);
        let current = entry * (1.0 + change_pct / 100.0);
        let qty = rng.gen_range(*min_q..*max_q);
        
        test_positions.push(Position {
            symbol: symbol.to_string(),
            quantity: qty,
            entry_price: entry,
            current_price: current,
            pnl: (current - entry) * qty,
            pnl_percent: ((current - entry) / entry) * 100.0,
            market_value: current * qty,
            asset_type: "stock".to_string(),
        });
    }
    
    let cryptos = vec![
        ("BTC/USD", 85000.0, 105000.0, 0.05, 0.25),
        ("ETH/USD", 3000.0, 4000.0, 1.5, 5.0),
        ("XRP/USD", 0.45, 0.75, 1000.0, 3000.0),
    ];
    
    let num_cryptos = rng.gen_range(2..=3);
    for (symbol, min_p, max_p, min_q, max_q) in cryptos.iter().take(num_cryptos) {
        let entry = rng.gen_range(*min_p..*max_p);
        let change_pct = rng.gen_range(-12.0..15.0);
        let current = entry * (1.0 + change_pct / 100.0);
        let qty = rng.gen_range(*min_q..*max_q);
        
        test_positions.push(Position {
            symbol: symbol.to_string(),
            quantity: qty,
            entry_price: entry,
            current_price: current,
            pnl: (current - entry) * qty,
            pnl_percent: ((current - entry) / entry) * 100.0,
            market_value: current * qty,
            asset_type: "crypto".to_string(),
        });
    }
    
    let total = test_positions.len();
    drop(test_positions);
    
    let base_time = Utc::now();
    let mut portfolio_history = state.portfolio_history.write().await;
    portfolio_history.clear();
    
    let initial = 100000.0;
    for i in 0..30 {
        let trend = (i as f64) * 100.0;
        let volatility = rng.gen_range(-500.0..800.0);
        let total_value = initial + trend + volatility;
        let pos_pct = rng.gen_range(0.15..0.45);
        let positions_value = total_value * pos_pct;
        
        portfolio_history.push(PortfolioSnapshot {
            timestamp: (base_time - chrono::Duration::minutes(30 - i)).to_rfc3339(),
            total_value,
            cash: total_value - positions_value,
            positions_value,
        });
    }
    drop(portfolio_history);
    
    let mut trade_history = state.trade_history.write().await;
    trade_history.clear();
    
    let symbols = vec!["AAPL", "GOOGL", "NVDA", "TSLA", "BTC/USD", "ETH/USD"];
    for i in 0..rng.gen_range(8..15) {
        let symbol = symbols[rng.gen_range(0..symbols.len())];
        let action = if i % 3 == 0 { "SELL" } else { "BUY" };
        let qty = if symbol.contains("/") { rng.gen_range(0.1..2.0) } else { rng.gen_range(5.0..20.0) };
        let price = if symbol.contains("BTC") { rng.gen_range(85000.0..105000.0) } 
                   else if symbol.contains("ETH") { rng.gen_range(3000.0..4000.0) } 
                   else { rng.gen_range(150.0..500.0) };
        let pnl = if action == "SELL" { qty * rng.gen_range(-50.0..150.0) } else { 0.0 };
        
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
    
    state.logger.success("Test", &format!("✓ Random data: {} positions, {} trades", total, trades));
    StatusCode::OK
}'''

# Find the start of the old function
start_marker = 'async fn generate_test_data(State(state): State<AppState>) -> StatusCode {'
start_pos = content.find(start_marker)

if start_pos == -1:
    print("ERROR: Could not find generate_test_data function")
    sys.exit(1)

# Find the end (next function or end of async fn clear_test_data)
end_marker = '\nasync fn clear_test_data'
end_pos = content.find(end_marker, start_pos)

if end_pos == -1:
    print("ERROR: Could not find end of function")
    sys.exit(1)

# Replace
new_content = content[:start_pos] + new_function + content[end_pos:]

# Write back
with open(r'C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine\src\main.rs', 'w', encoding='utf-8') as f:
    f.write(new_content)

print("✓ Replaced generate_test_data function with randomized version")
