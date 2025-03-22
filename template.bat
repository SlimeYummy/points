@echo off

if "%1"=="--gen" (
    cd "%~dp0\turning-point"
    python tests.py

) else if "%1"=="--install" (
    cd "%~dp0\turning-point"
    pip install -e .

) else (
    echo Usage:
    echo   template.bat --gen
    echo   template.bat --install
)
