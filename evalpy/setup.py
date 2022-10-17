#!/usr/bin/env python
import sys

from setuptools import setup

from setuptools_rust import Binding, RustExtension

setup(
       name = "evalpy",
       version = "1.0",
       rust_extensions = [RustExtension("evalpy.evalpy", binding=Binding.PyO3)],
       zip_safe = False
)
