[Console]::OutputEncoding = [System.Text.Encoding]::UTF8
$OutputEncoding = [System.Text.Encoding]::UTF8

Write-Host "Checking port 9999..." -ForegroundColor Cyan

nportx check 9999
$code = $LASTEXITCODE

if ($code -eq 0) {
    Write-Host "[ok] Port 9999 is free. Nothing to do." -ForegroundColor Green
    exit 0
}

Write-Host "[!] Port 9999 is busy (exit code $code)." -ForegroundColor Yellow
Write-Host "Killing the process on 9999..." -ForegroundColor Yellow
nportx kill 9999