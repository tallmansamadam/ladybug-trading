@echo off
echo Installing dependencies...
cd rust-engine
cargo build
cd ..\gui
npm install
cd ..
echo.
echo Installation complete
echo Edit .env with your Alpaca credentials
pause
