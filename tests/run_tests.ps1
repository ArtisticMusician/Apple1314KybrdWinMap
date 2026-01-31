# --- run_tests.ps1 ---
# PowerShell test runner for A1314 Daemon

Write-Host "================================" -ForegroundColor Cyan
Write-Host "A1314 Daemon - Test Suite Runner" -ForegroundColor Cyan
Write-Host "================================" -ForegroundColor Cyan
Write-Host ""

# Check if cargo is available
$cargoPath = Get-Command cargo -ErrorAction SilentlyContinue
if (-not $cargoPath) {
    Write-Host "ERROR: Cargo not found! Please install Rust from https://rustup.rs/" -ForegroundColor Red
    exit 1
}

$totalTests = 6
$currentTest = 0
$failed = $false

function Run-Test {
    param(
        [string]$Name,
        [string]$Command
    )
    
    $script:currentTest++
    Write-Host "[$script:currentTest/$totalTests] $Name..." -ForegroundColor Yellow
    
    $output = Invoke-Expression $Command 2>&1
    
    if ($LASTEXITCODE -eq 0) {
        Write-Host "PASSED: $Name" -ForegroundColor Green
        Write-Host ""
        return $true
    } else {
        Write-Host "FAILED: $Name" -ForegroundColor Red
        Write-Host $output -ForegroundColor Red
        Write-Host ""
        return $false
    }
}

# Run tests
if (-not (Run-Test "Checking code formatting" "cargo fmt -- --check")) {
    Write-Host "Run 'cargo fmt' to fix formatting issues" -ForegroundColor Yellow
    $failed = $true
}

if (-not (Run-Test "Running linter (clippy)" "cargo clippy -- -D warnings")) {
    $failed = $true
}

if (-not (Run-Test "Running unit tests" "cargo test --test unit_tests")) {
    $failed = $true
}

if (-not (Run-Test "Running integration tests" "cargo test --test integration_tests")) {
    $failed = $true
}

if (-not (Run-Test "Building debug version" "cargo build")) {
    $failed = $true
}

if (-not (Run-Test "Building release version" "cargo build --release")) {
    $failed = $true
}

# Summary
Write-Host "================================" -ForegroundColor Cyan
if (-not $failed) {
    Write-Host "All tests passed successfully! ✓" -ForegroundColor Green
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Next steps:" -ForegroundColor Yellow
    Write-Host "  - Run daemon: target\release\a1314_daemon.exe"
    Write-Host "  - Install: target\release\a1314_daemon.exe --install"
    Write-Host ""
    exit 0
} else {
    Write-Host "Some tests failed! ✗" -ForegroundColor Red
    Write-Host "================================" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "Please fix the errors above and try again." -ForegroundColor Yellow
    Write-Host ""
    exit 1
}
