@echo off

@REM if "%1"=="--gen" (
@REM     cd "%~dp0\turning-point"
@REM     python tests.py

@REM ) else if "%1"=="--install" (
@REM     cd "%~dp0\turning-point"
@REM     pip install -e .

@REM ) else (
@REM     echo Usage:
@REM     echo   template.bat --gen
@REM     echo   template.bat --install
@REM )

if "%1"=="--gen-test" (
    cd "%~dp0\turning-point"
    npm run gen-test

) else if "%1"=="--gen-demo" (
    cd "%~dp0\turning-point"
    npm run gen-demo

) else (
    echo Usage:
    echo   template.bat --gen
)
