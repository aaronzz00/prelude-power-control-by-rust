@echo off
setlocal

:: Initialize Visual Studio environment
call "C:\Program Files\Microsoft Visual Studio\2022\Community\VC\Auxiliary\Build\vcvars64.bat"

if %ERRORLEVEL% NEQ 0 (
    echo Failed to initialize Visual Studio environment
    exit /b 1
)

echo Environment initialized successfully
echo.
echo Running cargo check...
cargo check

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo Cargo check failed
    exit /b 1
)

echo.
echo Running cargo test...
cargo test

endlocal
