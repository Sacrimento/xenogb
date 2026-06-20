#!/usr/bin/env bash
# blarggs/config.sh — Blargg test suite configuration

TIMEOUT=25

run_test() {
    local rom="$1"
    local output_dir="$2"

    local output
    output=$(timeout --foreground "$TIMEOUT" "$EXEC" --headless -s -c "$rom" 2>&1) || true
    echo "$output" > "$output_dir/stdout.txt"
    echo "$output" | grep -q "Passed"
}

check_test() {
    return 0
}
