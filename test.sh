#!/bin/bash

_PROJ=$(basename $(pwd))
cargo build
cargo build --release
echo "########################################## cargo test"
cargo test

if ! [ -e /dev/shm/$_PROJ ]; then
    mkdir -p /dev/shm/$_PROJ
fi

echo "########################################## tests/test_D.sqlite3 (v1)"
touch tests/test_D.sqlite3 
./target/debug/firefox_bookmark_to_csv -d tests/test_D.sqlite3 -i tests/input.json -c tests/v1.csv -o /dev/shm/$_PROJ/input.csv
touch /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv
rm /dev/shm/$_PROJ/input.csv

echo "########################################## tests/test_D.sqlite3 (v2)"
touch tests/test_D.sqlite3 
./target/debug/firefox_bookmark_to_csv -d tests/test_D.sqlite3 -i tests/input.json -c tests/v2.csv -o /dev/shm/$_PROJ/input.csv
touch /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv
rm /dev/shm/$_PROJ/input.csv


echo "########################################## tests/test_R.sqlite3 (v2)"
touch tests/test_R.sqlite3 
./target/release/firefox_bookmark_to_csv -d tests/test_R.sqlite3 -i tests/bookmarks.json -c tests/v2.csv -o /dev/shm/$_PROJ/bookmarks.csv
touch /dev/shm/$_PROJ/bookmarks.csv
tail /dev/shm/$_PROJ/bookmarks.csv
rm /dev/shm/$_PROJ/bookmarks.csv
