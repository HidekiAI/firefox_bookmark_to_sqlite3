#!/bin/bash

_PROJ=$(basename $(pwd))
cargo build
cargo build --release
cargo test

if ! [ -e /dev/shm/$_PROJ ]; then
    mkdir -p /dev/shm/$_PROJ
fi

touch test_D.sqlite3 
./target/debug/firefox_bookmark_to_csv -d test_D.sqlite3 -i tests/input.json -c tests/v1.csv -o /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv

touch test_R.sqlite3 
./target/release/firefox_bookmark_to_csv -d test_R.sqlite3 -i tests/bookmarks.json -c tests/v1.csv -o /dev/shm/$_PROJ/bookmarks.csv
tail /dev/shm/$_PROJ/bookmarks.csv
