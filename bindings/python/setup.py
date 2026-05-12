from setuptools import setup, find_packages
from pyo3 import RustExtension, setup

# Get build directory from Cargo.toml
import sys
import os

with open(os.path.join(os.path.dirname(__file__), 'Cargo.toml')) as f:
    for line in f:
        if line.startswith('directory = "'):
            build_dir = line.split('"')[1]
            break

# Ensure we're in the right directory
os.chdir(os.path.join(os.path.dirname(__file__), build_dir))

print(f"Build directory: {build_dir}")

setup(
    name='t27-python-bindings',
    version='0.1.0' if sys.argv[-1] == '0.1.0' else '0.1.0',
    packages=find_packages(),
    rust_extensions=[
        RustExtension('t27_core', 't27/bindings/python/t27_core'),
    ],
)
