#!/usr/bin/env bash

PROJECT=xenogb
SNAPSHOT_FN=vbuf_snapshot

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/debug/$PROJECT

source $TEST_DIR/../utils.sh

failed=0

fail () {
    echo "$1 --- FAILED";
    failed=1;
}

echo "# Run mattcurrie's tests"
for test_rom in $(find $TEST_DIR -name "*.gb") ; do
    test_case=$(basename $test_rom)
    reference=${test_rom%.gb}.png
    $EXEC --headless --stop-condition LDBB -c "$test_rom" &

    wait $!

    SNAPSHOT_PATH=/tmp/"$!"_"$SNAPSHOT_FN"

    if cmp_snapshot $SNAPSHOT_PATH $reference; then
        echo "$test_case --- OK"
    else
        fail $test_case
    fi
done;

exit $failed
