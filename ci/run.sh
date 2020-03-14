#!/bin/bash

set -euo pipefail

rustup self update

rustup toolchain install --component clippy --profile minimal nightly
rustup default nightly

case "$OP" in
	'build')
		cargo build -p "$CRATE"
		;;

	'clippy')
		cargo clippy -p "$CRATE"
		;;

	'clippy-tests')
		cargo clippy -p "$CRATE" --tests
		;;

	'test')
		cargo test -p "$CRATE"
		;;

	*)
		exit 1
		;;
esac
