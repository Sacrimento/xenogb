#!/usr/bin/env bash
# run_tests.sh — Single entry point for XenoGB test framework

set -euo pipefail

FRAMEWORK_DIR="$(dirname "$(realpath -s "$0")")"
export EXEC="$FRAMEWORK_DIR/../target/release/xenogb"
export SUITE_FILTERS=()
export SKIP_SUITES=()
export SKIP_SUBDIRS=()

# shellcheck disable=SC1091
source "$FRAMEWORK_DIR/framework.sh"
# shellcheck disable=SC1091
source "$FRAMEWORK_DIR/utils.sh"

parse_args "$@"
load_skip_list

ROOT_DIR="$FRAMEWORK_DIR/.."

echo "Building xenogb..."
cargo build --release --manifest-path "$ROOT_DIR/Cargo.toml"

run_framework
