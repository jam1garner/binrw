#!/usr/bin/env bash
export CARGO_INCREMENTAL=0
export RUSTFLAGS="-Zprofile -Ccodegen-units=1 -Copt-level=0 -Clink-dead-code -Coverflow-checks=off -Zpanic_abort_tests"
cargo +nightly test -p binread -p binread_derive
grcov . -s . -t html --branch --ignore-not-existing -o ./target/debug/coverage/
