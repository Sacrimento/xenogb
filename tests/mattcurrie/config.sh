#!/usr/bin/env bash
# mattcurrie/config.sh — Matt Currie test suite configuration

TIMEOUT=60
SNAPSHOT_FN=vbuf_snapshot

run_test() {
    local rom="$1"

    timeout --foreground "$TIMEOUT" "$EXEC" --headless --stop-condition LDBB --cpu-speed custom -c "$rom" 2>&1
}

check_test() {
    local rom="$1"
    local _output_dir="$2"

    local rom_base="${rom%.gb*}"
    local reference="${rom_base}.png"

    local snapshot_path="/tmp/$SNAPSHOT_FN"

    cmp_snapshot "$snapshot_path" "$reference"
}
