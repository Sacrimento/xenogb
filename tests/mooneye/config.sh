#!/usr/bin/env bash
# mooneye/config.sh — Mooneye test suite configuration

TIMEOUT=2
CPU_REGISTERS_FN=cpu_registers.txt

EXPECTED_REGS=("A:*" "F:*" "B:03" "C:05" "D:08" "E:0D" "H:15" "L:22")

run_test() {
    local rom="$1"
    local output_dir="$2"

    timeout --foreground "$TIMEOUT" "$EXEC" --test-out-dir "$output_dir" --headless --stop-condition LDBB -c "$rom" 2>&1
}

check_test() {
    local rom="$1"
    local output_dir="$2"

    cmp_regs "$output_dir/$CPU_REGISTERS_FN" "${EXPECTED_REGS[@]}"
}
