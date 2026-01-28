# Free ladybug processes (gracefully release them)
# Because killing ladybugs is bad luck! 🐞
$processes = Get-Process | Where-Object {$_.ProcessName -like "*ladybug*"}

if ($processes) {
    Write-Host "Found $($processes.Count) ladybug process(es) to free" -ForegroundColor Yellow
    
    foreach ($proc in $processes) {
        Write-Host "Gently releasing ladybug PID $($proc.Id)..." -ForegroundColor Cyan
        
        # Try graceful release first
        try {
            $proc.CloseMainWindow() | Out-Null
            Start-Sleep -Milliseconds 500
            
            # Check if it flew away
            if (!$proc.HasExited) {
                Write-Host "Ladybug needs encouragement to fly away..." -ForegroundColor Yellow
                Stop-Process -Id $proc.Id -Force
            } else {
                Write-Host "Ladybug flew away gracefully 🐞✨" -ForegroundColor Green
            }
        } catch {
            Write-Host "Giving ladybug a gentle nudge..." -ForegroundColor Yellow
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }
    
    Write-Host "`nAll ladybugs have been freed! Good luck ahead! 🐞🍀" -ForegroundColor Green
} else {
    Write-Host "No ladybugs found to free" -ForegroundColor Yellow
}

Start-Sleep -Seconds 1
