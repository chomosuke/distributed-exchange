#!/usr/bin/bash
port=$((8000 + $1))
loc=$("./node${1}/")
cargo run -p node -- -a 127.0.0.1:$port -c 127.0.0.1:8000 -p $loc
