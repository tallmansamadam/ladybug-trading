# Find ladybug processes
$processes = Get-Process | Where-Object {$_.ProcessName -like "*ladybug*"}

if ($processes) {
    Write-Host "Found $($processes.Count) ladybug process(es)" -ForegroundColor Yellow
    
    foreach ($proc in $processes) {
        Write-Host "Attempting graceful shutdown of PID $($proc.Id)..." -ForegroundColor Cyan
        
        # Try graceful close first
        try {
            $proc.CloseMainWindow() | Out-Null
            Start-Sleep -Milliseconds 500
            
            # Check if still running
            if (!$proc.HasExited) {
                Write-Host "Process still running, forcing termination..." -ForegroundColor Yellow
                Stop-Process -Id $proc.Id -Force
            } else {
                Write-Host "Process terminated gracefully" -ForegroundColor Green
            }
        } catch {
            Write-Host "Forcing termination..." -ForegroundColor Yellow
            Stop-Process -Id $proc.Id -Force -ErrorAction SilentlyContinue
        }
    }
    
    Write-Host "`nAll ladybug processes terminated" -ForegroundColor Green
} else {
    Write-Host "No ladybug processes found" -ForegroundColor Yellow
}

Start-Sleep -Seconds 1
