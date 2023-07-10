#!/usr/bin/env bash

git clone https://github.com/lambdaclass/cairo_native.git
brew install llvm@16 tmux
cd cairo_native && scripts/fetch-corelibs.sh && MLIR_SYS_160_PREFIX=/opt/homebrew/opt/llvm@16 cargo build --release
