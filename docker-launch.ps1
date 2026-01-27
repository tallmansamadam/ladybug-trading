# Docker Launch Script for LadyBug Trading Engine

Write-Host "🐳 LadyBug Trading Engine - Docker Launch" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Gray
Write-Host ""

# Check if Docker is running
Write-Host "🔍 Checking Docker..." -ForegroundColor Yellow
try {
    docker version | Out-Null
    Write-Host "  ✅ Docker is running" -ForegroundColor Green
} catch {
    Write-Host "  ❌ Docker is not running!" -ForegroundColor Red
    Write-Host "     Please start Docker Desktop" -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Check if .env file exists
Write-Host "🔍 Checking environment file..." -ForegroundColor Yellow
$envPath = "C:\Users\frank\Documents\scripts\trading-scripts\ladybug\rust-engine\.env"
if (Test-Path $envPath) {
    Write-Host "  ✅ .env file found" -ForegroundColor Green
} else {
    Write-Host "  ❌ .env file NOT found!" -ForegroundColor Red
    Write-Host "     Create $envPath with your Alpaca credentials" -ForegroundColor Yellow
    exit 1
}

Write-Host ""

# Navigate to project directory
Set-Location "C:\Users\frank\Documents\scripts\trading-scripts\ladybug"

# Check if containers are already running
Write-Host "🔍 Checking existing containers..." -ForegroundColor Yellow
$runningBackend = docker ps --filter "name=ladybug-backend" --format "{{.Names}}" 2>$null
$runningFrontend = docker ps --filter "name=ladybug-frontend" --format "{{.Names}}" 2>$null

if ($runningBackend -or $runningFrontend) {
    Write-Host "  ⚠️  Containers already running" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "Options:" -ForegroundColor Cyan
    Write-Host "  1. Restart containers" -ForegroundColor White
    Write-Host "  2. Stop and remove containers" -ForegroundColor White
    Write-Host "  3. View logs" -ForegroundColor White
    Write-Host "  4. Exit" -ForegroundColor White
    Write-Host ""
    $choice = Read-Host "Select option (1-4)"
    
    switch ($choice) {
        "1" {
            Write-Host "🔄 Restarting containers..." -ForegroundColor Yellow
            docker-compose restart
        }
        "2" {
            Write-Host "🛑 Stopping and removing containers..." -ForegroundColor Yellow
            docker-compose down
            Write-Host "  ✅ Containers stopped and removed" -ForegroundColor Green
            exit 0
        }
        "3" {
            Write-Host "📋 Viewing logs (Ctrl+C to exit)..." -ForegroundColor Yellow
            docker-compose logs -f
            exit 0
        }
        "4" {
            Write-Host "👋 Exiting..." -ForegroundColor Yellow
            exit 0
        }
        default {
            Write-Host "❌ Invalid option" -ForegroundColor Red
            exit 1
        }
    }
} else {
    # Start containers
    Write-Host "🚀 Starting LadyBug containers..." -ForegroundColor Green
    Write-Host ""
    
    docker-compose up -d
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host ""
        Write-Host "✅ Containers started successfully!" -ForegroundColor Green
        Write-Host ""
        
        # Wait for services to be ready
        Write-Host "⏳ Waiting for services to start..." -ForegroundColor Yellow
        Start-Sleep -Seconds 10
        
        # Check health
        Write-Host ""
        Write-Host "🔍 Checking service health..." -ForegroundColor Yellow
        
        try {
            $health = Invoke-RestMethod -Uri "http://localhost:8080/health" -ErrorAction Stop
            Write-Host "  ✅ Backend is healthy" -ForegroundColor Green
        } catch {
            Write-Host "  ⚠️  Backend not responding yet (may need more time)" -ForegroundColor Yellow
        }
        
        Write-Host ""
        Write-Host "=" * 60 -ForegroundColor Gray
        Write-Host "🎉 LADYBUG IS RUNNING!" -ForegroundColor Green
        Write-Host "=" * 60 -ForegroundColor Gray
        Write-Host ""
        Write-Host "🌐 Access Points:" -ForegroundColor Cyan
        Write-Host "   Backend API:  http://localhost:8080" -ForegroundColor White
        Write-Host "   Frontend UI:  http://localhost:3000" -ForegroundColor White
        Write-Host ""
        Write-Host "📊 Generate Test Data:" -ForegroundColor Cyan
        Write-Host "   Invoke-RestMethod -Uri 'http://localhost:8080/test/generate' -Method Post" -ForegroundColor Gray
        Write-Host ""
        Write-Host "📋 View Logs:" -ForegroundColor Cyan
        Write-Host "   docker-compose logs -f" -ForegroundColor Gray
        Write-Host ""
        Write-Host "🛑 Stop Containers:" -ForegroundColor Cyan
        Write-Host "   docker-compose stop" -ForegroundColor Gray
        Write-Host ""
        
        # Ask if user wants to open browser
        $openBrowser = Read-Host "Open frontend in browser? (y/n)"
        if ($openBrowser -eq "y") {
            Start-Process "http://localhost:3000"
        }
        
        # Ask if user wants to generate test data
        $generateTest = Read-Host "Generate test data with live prices? (y/n)"
        if ($generateTest -eq "y") {
            Write-Host "🧪 Generating test data..." -ForegroundColor Yellow
            Start-Sleep -Seconds 2
            try {
                Invoke-RestMethod -Uri "http://localhost:8080/test/generate" -Method Post | Out-Null
                Write-Host "  ✅ Test data generated!" -ForegroundColor Green
            } catch {
                Write-Host "  ⚠️  Failed to generate test data. Try again in a few seconds." -ForegroundColor Yellow
            }
        }
        
        Write-Host ""
        Write-Host "✨ LadyBug is now running and trading! Watch the logs to see activity." -ForegroundColor Green
        
    } else {
        Write-Host ""
        Write-Host "❌ Failed to start containers" -ForegroundColor Red
        Write-Host "   Check the error messages above" -ForegroundColor Yellow
        Write-Host "   Run: docker-compose logs" -ForegroundColor Gray
    }
}
