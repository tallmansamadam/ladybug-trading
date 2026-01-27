# Stop any running instances
Stop-Process -Name "ladybug-engine" -Force -ErrorAction SilentlyContinue
Start-Sleep -Seconds 1

# Build and run
cd rust-engine
cargo run