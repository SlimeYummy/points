@echo off

cd "%~dp0\critical-point\csbridge"
cargo build

cd ..\..
xcopy .\critical-point\target\debug\critical_point_csbridge.dll .\critical-point-cs\tests\bin\Debug\net8.0\ /Y

if "%1"=="--gd" (
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll ..\g1\ /Y
)

if "%1"=="--u3d" (
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll ..\G1\ /Y
    xcopy .\critical-point-cs\bridge\*.cs ..\G1\Assets\Scripts\CriticalPoint\ /Y
)
