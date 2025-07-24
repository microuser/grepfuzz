# grepfuzz

`grepfuzz` is a modular, layered Rust CLI and library for filtering filenames and images using blurry image detection. It supports Linux-style slash-zero-terminated pipes and can process images from files, stdin, or synthetic sources.

## Project Structure
- **main.rs**: Thin CLI entrypoint. All logic is delegated to the library and helper modules.
- **lib.rs**: Root of the library crate. Re-exports all major modules and contains the main processing pipeline (`process_image`).
- **Modules**:
    - `cli.rs`: CLI argument parsing and mode selection
    - `config.rs`: Configuration and merging logic
    - `image_loader.rs`: Unified image input handling (synthetic, stdin, file) via `ImageInputMode` and `analyze_image_input`
    - `detector_helpers.rs`: Blur detector construction
    - `output_helpers.rs`: Output formatting and printing
    - `image_source_helpers.rs`: Image source selection logic
    - `blur_detector.rs`, `blur_laplacian.rs`, `blur_tenengrad.rs`, `blur_opencv.rs`: Blur detection algorithms
    - `blur_result.rs`: Result struct for detector outputs
    - `metadata.rs`: EXIF and metadata extraction

For a detailed layered architecture, call stack, and function reference, see [ARCHITECTURE.md](ARCHITECTURE.md).

---
This project is modular and maintainable, following best practices for Rust CLI+library applications.
