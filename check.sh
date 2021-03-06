#!/bin/sh
set -ex
# Deny all warnings here, becomes a pain to scroll back otherwise
cargo hack clippy --feature-powerset -- -D warnings
# Same as CI runs
cargo clippy --all-targets --all-features -- -D warnings
# Running all modules like this causes a lot of rebuilds which take a lot of time
cargo hack test --feature-powerset
# Make sure dependencies don't have any advisories or weird licensing
cargo deny --all-features --frozen --locked check
