# BlurDetect HOWTO

## Installation

### From crates.io (recommended, once published)
cargo install blurdetect

### Build from source
1. Clone the repository: `git clone https://github.com/microuser/blurdetect.git`
2. Navigate to the project: `cd blurdetect`
3. Build the release binary: `cargo build --release`
4. The binary will be available at `target/release/blurdetect`

## Usage
BlurDetect reads zero-terminated file paths from stdin (e.g., from `find . -print0`) and outputs the paths of blurry images in the same format.

Example:
find . -name "*.jpg" -print0 | blurdetect

### Flags
- `-h` or `--human-readable`: Enable human-readable output with additional metadata (file size, resolution, focal length from EXIF).
- `-t <VALUE>` or `--threshold <VALUE>`: Set the blur threshold (default: 100.0). Images with Laplacian variance below this are considered blurry.

For human-readable output:
find . -name "*.jpg" -print0 | blurdetect -h

This tool supports common image formats like JPEG and PNG. Errors (e.g., invalid images) are logged to stderr, and the image is skipped.

## Usage examples

**Blurry only (default):**
```sh
find ./images/ -iname '*.jpg' -print0 | ./target/debug/grepfuzz
```

**Sharp only:**
```sh
find ./images/ -iname '*.jpg' -print0 | ./target/debug/grepfuzz -s
```