This is not the main project repo, but rather an auxillary workspace for developing and testing data analysis and processing tools for use throughout the main project.

---

# Self-similarity analysis (ss_analysis)

# EvalPy

Python bindings for Rust based evaluation library.

## Setup guide

**Prerequisites:** A working python installation, and a stable rust toolchain for your desired platform.

The first step is to setup a virtual evironment

```sh
python -m venv ./venv
chmod +x ./venv/bin/activate # only on unix systems
```

To activate the virtual environment (this needs to be redone everytime the shell is closed):
```sh
./venv/bin/activate
```

Install dependencies and build the bindings:
```sh
pip install -r ./evalpy/requirements.txt
cd ./evalpy
python ./evalpy/setup.py build
```
