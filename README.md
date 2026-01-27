# 🐞 LadyBug Trading Engine v0.2.0

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Node.js](https://img.shields.io/badge/node.js-20%2B-green.svg)](https://nodejs.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Version](https://img.shields.io/badge/version-0.2.0-blue.svg)](https://github.com/tallmansamadam/ladybug-trading)

A high-performance algorithmic trading system built with Rust and TypeScript, featuring real-time market analysis, automated trading execution for both **stocks and cryptocurrencies**, and a modern web-based dashboard.

## 🚀 Features

- **Real-time Market Analysis**: Technical indicators and sentiment analysis for stocks and crypto
- **Automated Stock Trading**: 20+ major tech stocks with configurable signals
- **Cryptocurrency Trading**: Bitcoin (BTC), Ethereum (ETH), and Ripple (XRP) support
- **Live Dashboard**: Monitor positions, P&L, and trading activity in real-time
- **Paper Trading**: Test strategies safely with Alpaca's paper trading API
- **Real Market Data**: 100% live prices and historical data from Alpaca (no fake test data)
- **Activity Logging**: Comprehensive trade and analysis history
- **Portfolio Tracking**: Real-time portfolio value and performance metrics
- **Dual Trading Modes**: Independent controls for stock and crypto trading

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Rust 1.83+**: [Install Rust](https://www.rust-lang.org/tools/install)
- **Node.js 20+**: [Install Node.js](https://nodejs.org/)
- **Docker**: [Install Docker](https://www.docker.com/get-started)
- **Alpaca Account**: [Sign up for Alpaca](https://alpaca.markets/) (free paper trading account)

## 🛠️ Installation

### 1. Clone the Repository

```bash
git clone https://github.com/tallmansamadam/ladybug-trading.git
cd ladybug-trading
```

### 2. Set Up Environment Variables

Copy the example environment file and add your Alpaca API credentials:

```bash
cp .env.example .env
```

Edit `.env` and add your credentials:

```env
ALPACA_API_KEY=your_api_key_here
ALPACA_API_SECRET=your_api_secret_here
```

### 3. Install Dependencies

**Windows:**
```bash
install.bat
```

**Manual Installation:**
```bash
# Install Rust dependencies
cd rust-engine
cargo build

# Install Node.js dependencies
cd ../gui
npm install
```

## 🏃 Running the Application

### Option 1: Quick Start (Windows)

```bash
run-dev.bat
```

### Option 2: Manual Start

**Terminal 1 - Start Docker Services:**
```bash
cd docker
docker-compose up
```

**Terminal 2 - Start Rust Trading Engine:**
```bash
cd rust-engine
cargo run
```

**Terminal 3 - Start Web Dashboard:**
```bash
cd gui
npm run dev
```

### Access the Dashboard

Open your browser and navigate to:
- **Dashboard**: http://localhost:3000
- **API**: http://localhost:8080

## 📊 Usage

### Enable Stock Trading

```bash
curl -X POST http://localhost:8080/toggle \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

Or use the dashboard toggle.

### Enable Cryptocurrency Trading

```bash
curl -X POST http://localhost:8080/toggle/crypto \
  -H "Content-Type: application/json" \
  -d '{"enabled": true}'
```

### View Positions

```bash
# Stock positions
curl http://localhost:8080/positions

# Crypto positions
curl http://localhost:8080/positions/crypto
```

### View Logs

Check the `/logs` directory for detailed trading activity logs.

### API Endpoints

The Rust backend exposes the following endpoints:

**General:**
- `GET /health` - Health check
- `GET /status` - Trading status (stocks + crypto)
- `GET /account` - Account information
- `GET /logs` - Activity logs
- `GET /portfolio/history` - Portfolio value over time
- `GET /trades/history` - Trade execution history

**Stock Trading:**
- `GET /positions` - Current stock positions
- `POST /toggle` - Enable/disable stock trading

**Cryptocurrency Trading:**
- `GET /positions/crypto` - Current crypto positions
- `POST /toggle/crypto` - Enable/disable crypto trading
- `POST /test/generate` - Generate test data for demo

## 🏗️ Project Structure

```
ladybug-trading/
├── rust-engine/          # Rust trading engine
│   ├── src/
│   │   ├── main.rs       # Main application entry
│   │   ├── alpaca.rs     # Alpaca API client
│   │   ├── news.rs       # News aggregation & sentiment
│   │   ├── technical.rs  # Technical analysis
│   │   └── activity.rs   # Activity logging
│   └── Cargo.toml
├── gui/                  # TypeScript/React dashboard
│   ├── src/
│   │   ├── components/
│   │   └── main.tsx
│   └── package.json
├── config/               # Configuration files
├── docker/               # Docker compose setup
└── .env                  # Environment variables
```

## ⚙️ Configuration

### Trading Parameters

Edit `rust-engine/src/main.rs` to adjust:

**Stocks:**
- **Buy/Sell Thresholds**: ±0.15 (aggressive)
- **Position Size**: 5% of buying power, max $5,000
- **Trading Cycle**: Every 90 seconds
- **Monitored Stocks**: 20 major tech stocks (AAPL, GOOGL, MSFT, etc.)

**Cryptocurrency:**
- **Buy/Sell Thresholds**: ±0.25 (conservative - crypto is volatile)
- **Position Size**: 2% of buying power, max $2,000
- **Trading Cycle**: Every 120 seconds (2 minutes)
- **Supported Cryptos**: BTC/USD, ETH/USD, XRP/USD
- **Fractional Support**: Trade partial coins (e.g., 0.00512 BTC)

### Risk Management

**Stocks:**
- Maximum position size capped at $5,000
- Diversification across 20+ stocks
- Paper trading mode for testing

**Cryptocurrency:**
- Maximum position size capped at $2,000 (lower due to volatility)
- Higher signal threshold to reduce false signals
- Smaller portfolio allocation (2% vs 5%)

**General:**
- Stop-loss and take-profit logic in technical analysis
- Real-time data prevents trading on stale information

## 🧪 Testing

### Run Tests

```bash
cd rust-engine
cargo test
```

## 🛑 Stopping the Application

**Windows:**
```bash
stop-all.bat
```

**Manual:**
```bash
# Stop Rust engine: Ctrl+C in Terminal 2
# Stop GUI: Ctrl+C in Terminal 3
# Stop Docker: cd docker && docker-compose down
```

## 📈 Trading Strategy

The LadyBug engine uses a combination of:

1. **Technical Indicators**:
   - Moving averages (SMA, EMA)
   - RSI (Relative Strength Index)
   - MACD (Moving Average Convergence Divergence)
   - Volume analysis

2. **Real Market Data** (v0.2.0+):
   - Live prices from Alpaca API
   - Actual historical bars (not simulated)
   - Real-time sentiment from news aggregation

3. **Signal Generation**:
   
   **Stocks:**
   - Buy when technical score > 0.15 + positive sentiment
   - Sell when technical score < -0.15 + negative sentiment
   - Position size: 5% of buying power (max $5,000)
   
   **Cryptocurrency:**
   - Buy when technical score > 0.25 + positive sentiment (higher threshold)
   - Sell when technical score < -0.25 + negative sentiment
   - Position size: 2% of buying power (max $2,000)
   - Fractional quantities supported

## 🔒 Security

- **Never commit `.env` files** - They contain sensitive API keys
- Use **paper trading** for testing strategies
- Review all trades before enabling live trading
- Keep API keys secure and rotate them regularly

## 🐛 Troubleshooting

### Common Issues

**"No Alpaca credentials found"**
- Check that `.env` file exists and contains valid credentials
- Ensure environment variables are loaded correctly

**"Port already in use"**
- Stop any existing instances with `stop-all.bat`
- Check for processes using ports 8080 or 3000

**"Failed to fetch positions"**
- Verify Alpaca API credentials are correct
- Check that you're using paper trading credentials
- Ensure internet connection is stable

## 📝 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## ⚠️ Disclaimer

This software is for educational purposes only. Trading stocks carries risk, and you should never trade with money you cannot afford to lose. The authors are not responsible for any financial losses incurred while using this software.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/AmazingFeature`)
3. Commit your changes (`git commit -m 'Add some AmazingFeature'`)
4. Push to the branch (`git push origin feature/AmazingFeature`)
5. Open a Pull Request

## 📧 Contact

For questions or support, please open an issue on GitHub.

---

**Happy Trading! 🐞📈**
