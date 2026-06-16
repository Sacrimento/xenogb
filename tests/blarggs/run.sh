#!/usr/bin/env bash

PROJECT=xenogb

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/release/$PROJECT
# Last blarggs test takes ~20s (https://github.com/retrio/gb-test-roms/blob/c240dd7d700e5c0b00a7bbba52b53e4ee67b5f15/cpu_instrs/source/11-op%20a%2C(hl).s#L2)
TIMEOUT=25

failed=0
tests=""

echo "# Run blargg's tests"
for file in $TEST_DIR/roms/*; do
    test_case=$(basename $file)
    (timeout --foreground $TIMEOUT $EXEC --headless -s -c "$file" 2>&1 | grep -q "Passed") &
    tests+=" $test_case:$!"
done;

for t in $tests; do
    c=$(echo $t | cut -d : -f 1)
    p=$(echo $t | cut -d : -f 2)
    if wait $p; then
        echo "$c --- OK"
    else
        failed=1
        echo "$c --- FAILED"
    fi
done

exit $failed
