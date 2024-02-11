#!/bin/bash
_TEST_DIR=./samples

_PROJ=$(basename $(pwd))
cargo build
cargo build --release && cp target/release/firefox_bookmark_to_csv .
echo "########################################## cargo test"
cargo test

if ! [ -e /dev/shm/$_PROJ ]; then
    mkdir -p /dev/shm/$_PROJ
fi

echo "########################################## ${_TEST_DIR}/test_D.sqlite3 (v1) - assumes DB does NOT preexist"
touch ${_TEST_DIR}/test_D.sqlite3 
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
./target/debug/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_D.sqlite3 -i ${_TEST_DIR}/input.json -c ${_TEST_DIR}/v1.csv -o /dev/shm/$_PROJ/input.csv
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
touch /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv
rm /dev/shm/$_PROJ/input.csv
# NOTE: Follow through to next test without deleting DB

echo "########################################## ${_TEST_DIR}/test_D.sqlite3 (v2) - assumes DB exists"
touch ${_TEST_DIR}/test_D.sqlite3 
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
./target/debug/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_D.sqlite3 -i ${_TEST_DIR}/input.json -c ${_TEST_DIR}/v2.csv -o /dev/shm/$_PROJ/input.csv
sqlite3 ${_TEST_DIR}/test_D.sqlite3 ".schema"
touch /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv
rm /dev/shm/$_PROJ/input.csv
rm ${_TEST_DIR}/test_D.sqlite3 

echo "########################################## ${_TEST_DIR}/test_R.sqlite3 (v2) - assumes DB does NOT preexist"
touch ${_TEST_DIR}/test_R.sqlite3 
sqlite3 ${_TEST_DIR}/test_R.sqlite3 ".schema"
./target/release/firefox_bookmark_to_csv -d ${_TEST_DIR}/test_R.sqlite3 -i ${_TEST_DIR}/bookmarks.json -c ${_TEST_DIR}/v2.csv -o /dev/shm/$_PROJ/bookmarks.csv
sqlite3 ${_TEST_DIR}/test_R.sqlite3 ".schema"
touch /dev/shm/$_PROJ/bookmarks.csv
tail /dev/shm/$_PROJ/bookmarks.csv
rm /dev/shm/$_PROJ/bookmarks.csv
rm ${_TEST_DIR}/test_R.sqlite3 
