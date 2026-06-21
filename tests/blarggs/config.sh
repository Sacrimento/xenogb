#!/usr/bin/env bash
# blarggs/config.sh — Blargg test suite configuration

TIMEOUT=25

run_test() {
    local rom="$1"

    (set +o pipefail; timeout --foreground "$TIMEOUT" "$EXEC" --headless -s -c "$rom" 2>&1 | grep -q "Passed")
}

check_test() {
    return 0
}
