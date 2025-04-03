#!/usr/bin/env sh

set +e

PROJECT=xenogb
SNAPSHOT_FN=vbuf_snapshot

TEST_DIR=$(dirname $(realpath -s $0))
ROOT=$TEST_DIR/../..
EXEC=$ROOT/target/debug/$PROJECT

failed=0

rm -f $TEST_DIR/*.pgm
rm -f $TEST_DIR/$SNAPSHOT_FN.png

fail () {
    echo "$1 --- FAILED";
    failed=1;
}

echo "# Run mattcurrie's tests"
for test_rom in $(find $TEST_DIR -name "*.gb") ; do
    test_case=$(basename $test_rom)
    reference=${test_rom%.gb}.png
    $EXEC --headless -c "$test_rom" &
    sleep 1
    if ! kill -s USR1 $(pidof $PROJECT) > /dev/null 2>&1; then
        fail $test_case
        continue
    fi
    sleep 1
    convert $SNAPSHOT_FN.pgm $SNAPSHOT_FN.png
    if [ $(compare -metric AE $SNAPSHOT_FN.png $reference NULL: 2>&1) = "0" ]; then
        echo "$test_case --- OK"
    else
        fail $test_case
    fi

    rm -f $TEST_DIR/$SNAPSHOT_FN.pgm
    rm -f $TEST_DIR/$SNAPSHOT_FN.png
done;

exit $failed
