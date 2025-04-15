#!/usr/bin/env bash

PROJECT=xenogb

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/debug/$PROJECT

failed=0
tests=""

echo "# Run blargg's tests"
for file in $TEST_DIR/roms/*; do
    test_case=$(basename $file)
    (timeout --foreground 10 $EXEC --headless -s -c "$file" 2>&1 | grep -q "Passed") &
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
