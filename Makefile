# Makefile for grepfuzz

# Usage:
#   make testImages

# The expected output file should be created manually by the user and placed as test/expected_output.txt

.PHONY: testImages clean

testImages:
	@mkdir -p test
	@find ./images -type f -iname '*.jpg' -print0 |./run_grepfuzz.sh -a > test/actual_output.txt
	@if [ ! -f test/expected_output.txt ]; then \
		echo "No expected output found. Please create test/expected_output.txt."; \
		exit 1; \
	 fi
	@diff -u test/expected_output.txt test/actual_output.txt && echo "PASS: Output matches expected." || (echo "FAIL: Output differs from expected." && exit 1)

clean:
	rm -rf test target

build:
	cargo build

# Test passthrough mode with null-terminated filelist
test-null-filelist: build
	@echo 'file1.jpg\0file2.jpg\0file3.jpg\0' > test/filelist_in.txt
	@./target/debug/grepfuzz -p < test/filelist_in.txt > test/filelist_out.txt
	@diff --text test/filelist_in.txt test/filelist_out.txt
	@tr '\0' '|' < test/filelist_in.txt
	@tr '\0' '|' < test/filelist_out.txt
