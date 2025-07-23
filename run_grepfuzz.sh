#!/bin/bash
mkdir -p test
find ./images/ -iname '*.jpg' -print0 | tee test/find_pipe.txt | ./target/debug/grepfuzz | tee test/grepfuzz_output.txt
if [ ! -f test/expected_output.txt ]; then
    cp test/grepfuzz_output.txt test/expected_output.txt
    echo "[INFO] test/expected_output.txt created from current output."
fi
