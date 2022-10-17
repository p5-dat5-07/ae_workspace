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
```

Once created the environment must be activated. This has to be done everytime your shell is restarted:
```sh
source ./venv/bin/activate # POSIX/unix only
venv\Scripts\activate.bat # Windows only
```

Install dependencies and build the bindings:
```sh
pip install -r ./evalpy/requirements.txt
pip install maturin
maturin develop -r -m ./evalpy/Cargo.toml
```
