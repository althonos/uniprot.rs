#!/bin/sh -e

. $(dirname $0)/functions.sh

# --- Test without features --------------------------------------------------

log Testing code without default features
cargo tarpaulin -v --release --no-default-features

# --- Test with features -----------------------------------------------------

log Measuring code coverage with Tarpaulin
cargo tarpaulin -v --release --out Xml --ciserver travis-ci
