# Makefile for grepfuzz

# Usage:
#   make testImages

# The expected output file should be created manually by the user and placed as test/expected_output.txt

.PHONY: testImages clean run-synthetic-checkerboard run-synthetic-white run-file-find run-file-verbose run-null-filelist

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

run-synthetic-checkerboard:
	cargo run -- --synthetic-checkerboard --verbose

run-synthetic-white:
	cargo run -- --synthetic-white -v

run-find-std:
	pwd
	find ./images -type f -iname '*.jpg'
	echo verbose
	find ./images -type f -iname '*.jpg' -print0 | cargo run --release -- --verbose --blur --ascii
	echo normal
	find ./images -type f -iname '*.jpg' -print0 | cargo run --release -- --blur --ascii
	echo normal
	find ./images -type f -iname '*.jpg' -print0 | cargo run --release -- 

run-find-file:
	pwd
	ls -la .
	ls -la ./images
	echo verbose
	find ./images -type f -iname '*.jpg' -print0 | xargs -0 -I {} -n 1 cargo run --release -- -f {} --verbose
	echo normal
	find ./images -type f -iname '*.jpg' -print0 | xargs -0 -I {} -n 1 cargo run --release -- -f {}


run-file-verbose:
	@if [ -z "./images/image-26.jpg" ]; then echo "Usage: make run-file-verbose FILE=./images/image-26.jpg"; exit 1; fi
	cargo run -- -f "./images/image-26.jpg" --verbose

run-photoboothblack:
	cargo run -- --ascii -f ./images/photoboothblack.jpg

test-find-std:
	@mkdir -p test
	@$(MAKE) run-find-std > test/actual_run_find_std.txt 2>&1
	@if [ ! -f test/expected_run_find_std.txt ]; then \
		echo "No expected output found. Please create test/expected_run_find_std.txt."; \
		exit 1; \
	fi
	@diff -u test/expected_run_find_std.txt test/actual_run_find_std.txt && echo "PASS: run-find-std output matches expected." || (echo "FAIL: run-find-std output differs from expected." && exit 1)

install:
	@if [ "$(shell uname)" = "Darwin" ]; then \
		command -v brew >/dev/null 2>&1 || { echo >&2 "Homebrew not found. Please install Homebrew first."; exit 1; }; \
		brew install opencv; \
	elif [ -f /etc/debian_version ]; then \
		sudo apt-get update && sudo apt-get install -y libopencv-dev; \
	else \
		echo "Unsupported OS. Please install OpenCV manually."; \
		exit 1; \
	fi
	
build-opencv:
	LDFLAGS="-L/opt/homebrew/opt/llvm/lib" CPPFLAGS="-I/opt/homebrew/opt/llvm/include" cargo build

# Test passthrough mode with null-terminated filelist
test-null-filelist: build
	@echo 'file1.jpg\0file2.jpg\0file3.jpg\0' > test/filelist_in.txt
	@./target/debug/grepfuzz -p < test/filelist_in.txt > test/filelist_out.txt
	@diff --text test/filelist_in.txt test/filelist_out.txt
	@tr '\0' '|' < test/filelist_in.txt
	@tr '\0' '|' < test/filelist_out.txt

git-update:
	git fetch
	git pull

build:
	cargo build

run-stdin-bytes-cup:
	cat images/cup.jpg | cargo run -- --std_in_bytes --verbose
