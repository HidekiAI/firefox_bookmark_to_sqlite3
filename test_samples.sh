#!/bin/bash
_TEST_DIR=./samples

_PROJ=$(basename $(pwd))
cargo build
cargo build --release && cp target/release/firefox_bookmark_to_csv .
echo "########################################## cargo test"
cargo test

# NOTE: ${_TMPDIR} is ONLY for Linux, will fail on Windows (unless you're running
# memory-hogging WSL2)!  But because we're running this as BASH
# one thing we can assume is that if on Windows, it's running in MinGW, so
# you can use /tmp instead which should work on Windows as well
_TMPDIR="/tmp/$(whoami)/"
if ! [ -e ${_TMPDIR}/$_PROJ ]; then
    mkdir -p ${_TMPDIR}/$_PROJ
fi

echo "########################################## ${_TEST_DIR}/test_D.sqlite3 (v1) - assumes DB does NOT preexist"
touch ${_TEST_DIR}/test_D.sqlite3 
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
./target/debug/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_D.sqlite3 -i ${_TEST_DIR}/input.json -c ${_TEST_DIR}/v1.csv -o ${_TMPDIR}/$_PROJ/input.csv
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
touch ${_TMPDIR}/$_PROJ/input.csv
tail ${_TMPDIR}/$_PROJ/input.csv
rm ${_TMPDIR}/$_PROJ/input.csv
# NOTE: Follow through to next test without deleting DB

echo "########################################## ${_TEST_DIR}/test_D.sqlite3 (v2) - assumes DB exists"
touch ${_TEST_DIR}/test_D.sqlite3 
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
./target/debug/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_D.sqlite3 -i ${_TEST_DIR}/input.json -c ${_TEST_DIR}/v2.csv -o ${_TMPDIR}/$_PROJ/input.csv
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
touch ${_TMPDIR}/$_PROJ/input.csv
tail ${_TMPDIR}/$_PROJ/input.csv
rm ${_TMPDIR}/$_PROJ/input.csv
rm ${_TEST_DIR}/test_D.sqlite3 

echo "########################################## ${_TEST_DIR}/test_R.sqlite3 (v2) - assumes DB does NOT preexist"
touch ${_TEST_DIR}/test_R.sqlite3 
sqlite3 ${_TEST_DIR}/test_R.sqlite3 ".schema"
./target/release/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_R.sqlite3 -i ${_TEST_DIR}/bookmarks.json -c ${_TEST_DIR}/v2.csv -o ${_TMPDIR}/$_PROJ/bookmarks.csv
sqlite3 ${_TEST_DIR}/test_R.sqlite3 ".schema"
touch ${_TMPDIR}/$_PROJ/bookmarks.csv
tail ${_TMPDIR}/$_PROJ/bookmarks.csv
rm ${_TMPDIR}/$_PROJ/bookmarks.csv
rm ${_TEST_DIR}/test_R.sqlite3 
