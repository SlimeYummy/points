@echo off

if "%1"=="--gd" (
    cd "%~dp0\critical-point\csbridge"
    cargo build %2
    cd ..\..
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll .\critical-point-cs\tests\bin\Debug\net8.0\ /Y
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll ..\G1_\ /Y

) else if "%1"=="--u3d" (
    cd "%~dp0\critical-point\csbridge"
    cargo build %2
    cd ..\..
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll .\critical-point-cs\tests\bin\Debug\net8.0\ /Y
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll ..\G1\ /Y
    xcopy .\critical-point-cs\bridge\*.cs ..\G1\Assets\Scripts\CriticalPoint\ /Y

) else if "%1"=="--pyext" (
    cd "%~dp0\critical-point\pyext"
    maturin build -i python %2
    pip install --force-reinstall ..\target\wheels\critical_point_pyext-0.1.0-cp313-cp313-win_amd64.whl

) else if "%1"=="--debug" (
    cd "%~dp0\critical-point"
    cargo build
    cd ..
    xcopy .\critical-point\target\debug\critical_point_csbridge.dll .\critical-point-cs\tests\bin\Debug\net8.0\ /Y

) else if "%1"=="--release" (
    cd "%~dp0\critical-point"
    cargo build --release

) else (
    echo Usage:
    echo   build.bat --debug
    echo   build.bat --release
    echo   build.bat --gd --release
    echo   build.bat --u3d --release
    echo   build.bat --pyext --release
)
