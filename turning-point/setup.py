from setuptools import setup, find_packages

setup(
    name="turning_point",
    version="0.1.0",
    author="SlimeYummy",
    author_email="zzzcccnnn@outlook.com",
    description="Generate game objects from Python templates",
    url="https://github.com/SlimeYummy/points",
    packages=find_packages(exclude=["tests"]),
    # package_dir={"": "lib"},
    classifiers=[],
    python_requires=">=3.8",
    install_requires=["critical-point-pyext"]
)
