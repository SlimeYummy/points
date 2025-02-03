cd "%~dp0\critical-point\pyext"
maturin build -i python
pip install --force-reinstall ..\target\wheels\critical_point_pyext-0.1.0-cp312-none-win_amd64.whl
