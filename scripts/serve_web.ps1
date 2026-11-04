# OPURE OS — Local WebAssembly Playground Server
# Serves E:\opure_os over HTTP on port 8080 with proper WASM MIME-types

param (
    [int]$Port = 8080
)

$webRoot = Join-Path $PSScriptRoot "..\web"
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "  LAUNCHING OPURE OS CYBER-BRUTALIST WASM PLAYGROUND       " -ForegroundColor Cyan
Write-Host "==========================================================" -ForegroundColor Cyan
Write-Host "Serving path: $webRoot on http://localhost:$Port" -ForegroundColor Yellow

# Check if Python is available
if (Get-Command python -ErrorAction SilentlyContinue) {
    Start-Process "http://localhost:$Port/index.html"
    Set-Location -Path $webRoot
    python -m http.server $Port
} else {
    Write-Host "Python not found. Opening index.html directly..." -ForegroundColor Yellow
    Start-Process (Join-Path $webRoot "index.html")
}
