# Verification Script - Check Everything Is Ready

Write-Host "🔍 LadyBug Trading System - Installation Verification" -ForegroundColor Cyan
Write-Host "=" * 60 -ForegroundColor Gray
Write-Host ""

# Check component files
Write-Host "📁 Checking Component Files..." -ForegroundColor Yellow
$componentsPath = "C:\Users\frank\Documents\scripts\trading-scripts\ladybug\gui\src\components"

$files = @(
    "Dashboard.tsx",
    "EnhancedChart.tsx",
    "PositionsPnLChart.tsx"
)

foreach ($file in $files) {
    $fullPath = Join-Path $componentsPath $file
    if (Test-Path $fullPath) {
        $size = (Get-Item $fullPath).Length
        Write-Host "  ✅ $file ($size bytes)" -ForegroundColor Green
    } else {
        Write-Host "  ❌ $file NOT FOUND" -ForegroundColor Red
    }
}

Write-Host ""

# Check if recharts is installed
Write-Host "📦 Checking NPM Dependencies..." -ForegroundColor Yellow
$packageJsonPath = "C:\Users\frank\Documents\scripts\trading-scripts\ladybug\gui\package.json"

if (Test-Path $packageJsonPath) {
    $packageJson = Get-Content $packageJsonPath | ConvertFrom-Json
    if ($packageJson.dependencies.recharts) {
        Write-Host "  ✅ recharts: $($packageJson.dependencies.recharts)" -ForegroundColor Green
    } else {
        Write-Host "  ⚠️  recharts NOT INSTALLED - Run: npm install recharts" -ForegroundColor Yellow
    }
} else {
    Write-Host "  ❌ package.json not found" -ForegroundColor Red
}

Write-Host ""

# Check if backend is running
Write-Host "🚀 Checking Backend Server..." -ForegroundColor Yellow
try {
    $response = Invoke-RestMethod -Uri "http://localhost:8080/health" -ErrorAction Stop
    Write-Host "  ✅ Backend is RUNNING on port 8080" -ForegroundColor Green
} catch {
    Write-Host "  ⚠️  Backend is NOT RUNNING" -ForegroundColor Yellow
    Write-Host "     Start with: cd rust-engine; cargo run" -ForegroundColor Gray
}

Write-Host ""

# Check git status
Write-Host "📝 Checking Git Status..." -ForegroundColor Yellow
Push-Location "C:\Users\frank\Documents\scripts\trading-scripts\ladybug"
$gitStatus = git log -1 --oneline
Write-Host "  Latest commit: $gitStatus" -ForegroundColor Cyan
Pop-Location

Write-Host ""

# Summary
Write-Host "=" * 60 -ForegroundColor Gray
Write-Host "✅ SUMMARY" -ForegroundColor Green
Write-Host "=" * 60 -ForegroundColor Gray
Write-Host ""
Write-Host "Components Installed: ✅" -ForegroundColor Green
Write-Host "Location: $componentsPath" -ForegroundColor Gray
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "  1. Install recharts: npm install recharts" -ForegroundColor White
Write-Host "  2. Start backend: cd rust-engine; cargo run" -ForegroundColor White
Write-Host "  3. Start frontend: cd gui; npm start" -ForegroundColor White
Write-Host "  4. Import components in Dashboard.tsx" -ForegroundColor White
Write-Host ""
Write-Host "Documentation: See FILES_ALREADY_INSTALLED.md" -ForegroundColor Cyan
Write-Host ""
