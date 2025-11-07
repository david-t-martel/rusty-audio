# PowerShell script to check compilation
Set-Location "C:\Users\david\rusty-audio"

Write-Host "Starting compilation check..." -ForegroundColor Green

# Try to compile with detailed error output
cargo check --message-format=short --color=always 2>&1 | Tee-Object -FilePath "compile_errors.log"

Write-Host "Compilation check complete. Errors logged to compile_errors.log" -ForegroundColor Green