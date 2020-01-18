#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test without features --------------------------------------------------

log Testing code without default features
cargo test --release --no-default-features

# --- Test with coverage -----------------------------------------------------

if cargo tarpaulin -V >/dev/null 2>&1; then
	log Measuring code coverage with Tarpaulin
	cargo tarpaulin -v --release --out Xml --ciserver travis-ci
else
	log Testing code without coverage
	cargo test --release
fi
