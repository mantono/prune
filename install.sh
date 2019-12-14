#!/bin/sh
cargo build --release &&
sudo cp -vi target/release/prune /usr/local/bin/prn