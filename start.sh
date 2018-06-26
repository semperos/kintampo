#!/bin/bash

set -e -u

type cargo 2>&1 >/dev/null || \
    (echo "You need to install Rust and Cargo to proceed. Try rustup." && exit 1)

cd kintampo
RUST_LOG=kintampo=trace cargo run
