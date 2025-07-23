# Makefile for grepfuzz

# Usage:
#   make testImages

# The expected output file should be created manually by the user and placed as test/expected_output.txt

.PHONY: testImages clean

testImages:
	@mkdir -p test
	@./run_grepfuzz.sh > test/actual_output.txt
	@if [ ! -f test/expected_output.txt ]; then \
		echo "No expected output found. Please create test/expected_output.txt."; \
		exit 1; \
	 fi
	@diff -u test/expected_output.txt test/actual_output.txt && echo "PASS: Output matches expected." || (echo "FAIL: Output differs from expected." && exit 1)

clean:
	rm -rf test target
