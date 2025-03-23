#!/bin/bash

# Build sandbox
cd sandbox && cargo build --release && cd -

# Copy sandbox so to release directory.
mkdir -p target/release && cp sandbox/libportfolio_sandbox_sandbox.so target/release/
cargo build --release

# Run api and then core tests.
export PORTFOLIO_DATABASE_URL="mysql://root:@127.0.0.1/"
cd api && cargo test --release -- --test-threads=1 && cd - 
cd core && cargo test --release -- --test-threads=1 && cd -

