#!/bin/bash

_PROJ=$(basename $(pwd))
cargo build
cargo build --release
cargo test

if ! [ -e /dev/shm/$_PROJ ]; then
    mkdir -p /dev/shm/$_PROJ
fi
target/debug/json_to_csv -i tests/input.json -c tests/v1.csv -o /dev/shm/$_PROJ/input.csv
tail /dev/shm/$_PROJ/input.csv

target/release/json_to_csv -i tests/bookmarks.json -c tests/v1.csv -o /dev/shm/$_PROJ/bookmarks.csv
tail /dev/shm/$_PROJ/bookmarks.csv
