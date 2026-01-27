# LadyBug Trading Bot

Production algorithmic trading system.

## Quick Start

1. Edit `.env` with your Alpaca API credentials
2. Run `install.bat` to install dependencies
3. Run `run-dev.bat` to start databases
4. Open 2 terminals:
   - Terminal 2: `cd rust-engine && cargo run`
   - Terminal 3: `cd gui && npm run dev`
5. Access http://localhost:3000

## Requirements

- Rust 1.83+
- Node.js 20+
- Docker
- Alpaca account
