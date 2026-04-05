# t27 Python Bindings Setup
# TRINITY Ternary Computing Framework - Python Installation
# φ² + 1/φ² = 3 | TRINITY

"""
t27 - TRINITY Ternary Computing Framework Python Bindings

This package provides Python access to TRINITY's ternary computing
capabilities, including ternary logic operations and PyTorch layers
with ternary weights.
"""

from setuptools import setup, find_packages
from pathlib import Path

# Read README for long description
readme_file = Path(__file__).parent / "README.md"
long_description = readme_file.read_text() if readme_file.exists() else ""

setup(
    name="t27-trinity",
    version="0.1.0",
    description="TRINITY Ternary Computing Framework Python Bindings",
    long_description=long_description,
    long_description_content_type="text/markdown",
    author="TRINITY Project",
    author_email="trinity@example.com",
    url="https://github.com/trinity-framework/t27",
    packages=find_packages(),
    python_requires=">=3.8",
    install_requires=[
        "numpy>=1.19.0",
        "torch>=1.9.0",
    ],
    extras_require={
        "dev": [
            "pytest>=6.0",
            "pytest-cov>=2.0",
            "black>=21.0",
            "mypy>=0.900",
        ],
        "examples": [
            "matplotlib>=3.3",
            "torchvision>=0.10",
        ],
    },
    classifiers=[
        "Development Status :: 3 - Alpha",
        "Intended Audience :: Developers",
        "Intended Audience :: Science/Research",
        "License :: OSI Approved :: MIT License",
        "Programming Language :: Python :: 3",
        "Programming Language :: Python :: 3.8",
        "Programming Language :: Python :: 3.9",
        "Programming Language :: Python :: 3.10",
        "Programming Language :: Python :: 3.11",
        "Topic :: Scientific/Engineering :: Artificial Intelligence",
        "Topic :: Software Development :: Libraries :: Python Modules",
    ],
    keywords="ternary computing neural-networks kleene-logic k3 hslm",
    project_urls={
        "Bug Reports": "https://github.com/trinity-framework/t27/issues",
        "Source": "https://github.com/trinity-framework/t27",
        "Documentation": "https://trinity-framework.github.io/t27",
    },
)
