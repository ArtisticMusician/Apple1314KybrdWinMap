@echo off
REM --- run_tests.bat ---
REM Automated test runner for A1314 Daemon

echo ================================
echo A1314 Daemon - Test Suite Runner
echo ================================
echo.

REM Check if cargo is available
where cargo >nul 2>nul
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo not found! Please install Rust from https://rustup.rs/
    exit /b 1
)

echo [1/6] Checking code formatting...
cargo fmt -- --check
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Code formatting issues detected
    echo Run 'cargo fmt' to fix formatting
    exit /b 1
)
echo PASSED: Code formatting OK
echo.

echo [2/6] Running linter (clippy)...
cargo clippy -- -D warnings
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Clippy warnings detected
    exit /b 1
)
echo PASSED: Clippy checks OK
echo.

echo [3/6] Running unit tests...
cargo test --test unit_tests
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Unit tests failed
    exit /b 1
)
echo PASSED: Unit tests OK
echo.

echo [4/6] Running integration tests...
cargo test --test integration_tests
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Integration tests failed
    exit /b 1
)
echo PASSED: Integration tests OK
echo.

echo [5/6] Building debug version...
cargo build
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Debug build failed
    exit /b 1
)
echo PASSED: Debug build OK
echo.

echo [6/6] Building release version...
cargo build --release
if %ERRORLEVEL% NEQ 0 (
    echo FAILED: Release build failed
    exit /b 1
)
echo PASSED: Release build OK
echo.

echo ================================
echo All tests passed successfully! ✓
echo ================================
echo.
echo Next steps:
echo   - Run daemon: target\release\a1314_daemon.exe
echo   - Install: target\release\a1314_daemon.exe --install
echo.
