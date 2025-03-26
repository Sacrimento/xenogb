#!/usr/bin/env sh

PROJECT=xenogb

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/debug/$PROJECT

failed=0

echo "# Run blargg's tests"
for file in $TEST_DIR/roms/*; do
    cmd="$EXEC -c $file";
    if timeout --foreground 5 $EXEC --headless -s -c "$file" 2>&1 | grep -q "Passed"; then
        echo "$cmd --- OK";
    else
        echo "$cmd --- FAILED";
        failed=1;
    fi
done;

exit $failed
