#!/usr/bin/env bash

PROJECT=xenogb
CPU_REGISTERS_FN=cpu_registers.txt

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/release/$PROJECT

TIMEOUT=2

EXPECTED_REGS=("A:*" "F:*" "B:03" "C:05" "D:08" "E:0D" "H:15" "L:22")

source $TEST_DIR/../utils.sh

failed=0

tests=""

echo "# Run mooneye's tests"
for test_rom in $(find $TEST_DIR/roms -type f -regex ".*\.gbc?"); do
    test_case=$(basename $test_rom)
    timeout --foreground $TIMEOUT $EXEC --test-out-dir /tmp/${test_case} --headless --stop-condition LDBB -c "$test_rom" 2>&1 &
    tests+=" $test_case:$!"
done;

for t in $tests; do
    c=$(echo $t | cut -d : -f 1)
    p=$(echo $t | cut -d : -f 2)
    if wait $p && cmp_regs "/tmp/${c}/${CPU_REGISTERS_FN}" "${EXPECTED_REGS[@]}"; then
        echo "$c --- OK"
    else
        failed=1
        echo "$c --- FAILED"
    fi
done

exit $failed
