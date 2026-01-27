# 🐞 LadyBug Trading Engine

[![Rust](https://img.shields.io/badge/rust-1.83%2B-orange.svg)](https://www.rust-lang.org/)
[![Node.js](https://img.shields.io/badge/node.js-20%2B-green.svg)](https://nodejs.org/)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A high-performance algorithmic trading system built with Rust and TypeScript, featuring real-time market analysis, automated trading execution, and a modern web-based dashboard.

## 🚀 Features

- **Real-time Market Analysis**: Technical indicators and sentiment analysis
- **Automated Trading**: Configurable buy/sell signals with risk management
- **Live Dashboard**: Monitor positions, P&L, and trading activity in real-time
- **Paper Trading**: Test strategies safely with Alpaca's paper trading API
- **Multi-Asset Support**: Trade 20+ major stocks (AAPL, GOOGL, TSLA, etc.)
- **Activity Logging**: Comprehensive trade and analysis history
- **Portfolio Tracking**: Real-time portfolio value and performance metrics

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

### Enable Trading

1. Open the dashboard at http://localhost:3000
2. Click the **"Enable Trading"** toggle
3. Monitor real-time positions and trades

### View Logs

Check the `/logs` directory for detailed trading activity logs.

### API Endpoints

The Rust backend exposes the following endpoints:

- `GET /health` - Health check
- `GET /status` - Trading status and statistics
- `GET /positions` - Current open positions
- `GET /account` - Account information
- `GET /logs` - Activity logs
- `GET /portfolio/history` - Portfolio value over time
- `GET /trades/history` - Trade execution history
- `POST /toggle` - Enable/disable trading
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

- **Buy/Sell Thresholds**: Currently set to ±0.15 (very aggressive)
- **Position Size**: 5% of buying power, max $5,000
- **Trading Cycle**: Runs every 90 seconds
- **Monitored Stocks**: 20 major tech stocks

### Risk Management


- Maximum position size capped at $5,000
- Diversification across 20+ stocks
- Paper trading mode for testing
- Stop-loss and take-profit logic in technical analysis

## 🧪 Testing

### Generate Test Data

Use the test endpoint to populate the dashboard with sample data:

```bash
curl -X POST http://localhost:8080/test/generate
```

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

2. **Sentiment Analysis**:
   - News aggregation
   - Market sentiment scoring

3. **Signal Generation**:
   - Buy signals when technical score > 0.15 and positive sentiment
   - Sell signals when technical score < -0.15 and negative sentiment
   - Position sizing based on signal strength

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
