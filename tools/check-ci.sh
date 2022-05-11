#! /usr/bin/env bash

# Fail fast
set -e

cargo check
cargo check --release
cargo test
