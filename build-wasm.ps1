# Build script for multithreaded WASM
# Sets proper RUSTFLAGS for atomics support

$env:RUSTFLAGS = "-C target-feature=+atomics,+bulk-memory,+mutable-globals"

Write-Host "Building multithreaded WASM with atomics support..." -ForegroundColor Green
Write-Host "RUSTFLAGS: $env:RUSTFLAGS" -ForegroundColor Cyan

cargo build --target wasm32-unknown-unknown --profile wasm-release

if ($LASTEXITCODE -eq 0) {
    Write-Host ""
    Write-Host "[SUCCESS] Build successful!" -ForegroundColor Green
    Write-Host "Output: target/wasm32-unknown-unknown/wasm-release/rusty_audio.wasm" -ForegroundColor Cyan
} else {
    Write-Host ""
    Write-Host "[ERROR] Build failed with exit code $LASTEXITCODE" -ForegroundColor Red
    exit $LASTEXITCODE
}
