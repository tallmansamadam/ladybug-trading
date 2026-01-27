Get-Process | Where-Object {$_.ProcessName -like "*ladybug*"} | Stop-Process -Force
Write-Host "Ladybug processes terminated" -ForegroundColor Green
Start-Sleep -Seconds 1
