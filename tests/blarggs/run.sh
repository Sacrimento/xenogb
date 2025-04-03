#!/usr/bin/env sh

PROJECT=xenogb

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/debug/$PROJECT

failed=0

echo "# Run blargg's tests"
for file in $TEST_DIR/roms/*; do
    test_case=$(basename $file)
    if timeout --foreground 5 $EXEC --headless -s -c "$file" 2>&1 | grep -q "Passed"; then
        echo "$test_case --- OK";
    else
        echo "$test_case --- FAILED";
        failed=1;
    fi
done;

exit $failed
