# Architecture Overview

This document describes the general flow of the `main` function and the architecture of the image blur detection system.

## Main Flow

1. **Parameter Acquisition**
   - The program starts by accepting parameters and a list of filenames via command-line arguments and/or standard input (STDIN).
   - These user-provided parameters and filenames are merged with standard defaults defined in the configuration (`config`).

2. **Configuration Merging**
   - Default configuration values are loaded.
   - User-supplied parameters (from command-line or STDIN) override the defaults as appropriate.

3. **BlurDetector Implementations**
   - The system supports multiple implementations of the `BlurDetector` trait:
     - **LaplacianVarianceDetector** (uses OpenCV's Laplacian operator)
     - **TenengradDetector** (uses the Tenengrad method)
   - Each implementation produces a `Result` struct capturing the outcome of the blur detection for an image.

4. **Processing Filenames**
   - For each filename (image path) provided, the system processes the image using **each** implementation of `BlurDetector`.

5. **Image Loading (ImageSource)**
   - Images are loaded from sources defined by the `ImageSource` trait:
     - **STDIN**: Images can be piped into the program and read from standard input.
     - **File Path**: Images can be loaded from disk using the provided file paths.
   - The system determines the source type for each filename and loads the image accordingly.

6. **Synthetic Image Types**
   - In addition to real images, the system can generate and process synthetic images:
     - **SyntheticWhite**: Produces a white image.
     - **SyntheticCheckerboard**: Produces a checkerboard pattern image.
   - These synthetic images are useful for testing and validation.

7. **Result Storage and Output**
   - After processing, the results from each `BlurDetector` implementation are stored and can be output, logged, or further analyzed as needed.

## Diagram

```mermaid
flowchart TD
    A[Start: Read Params & STDIN] --> B[Merge with Config Defaults]
    B --> C[Load BlurDetector Implementations]
    C --> D{For Each Filename}
    D --> E[Identify ImageSource (STDIN, File, Synthetic)]
    E --> F[Load Image]
    F --> G{For Each BlurDetector}
    G --> H[Process Image]
    H --> I[Store Result]
    I --> J[Output / Next]
```

## Extensibility
- **Adding new BlurDetectors**: Implement the `BlurDetector` trait and register the new detector in the main flow.
- **Adding new ImageSources**: Implement the `ImageSource` trait for new input types.
- **Adding new Synthetic Types**: Implement synthetic generators as additional `ImageSource` variants.

---

This architecture ensures modularity, easy extensibility, and clear separation between configuration, image loading, processing, and result management.

# Detailed ASCII UML Architecture Diagram

Below is a more detailed diagram with key structs, functions, and data types for each layer.

```
+------------------------------------------------+
|           Configuration Layer                  |
|------------------------------------------------|
| Structs:                                      |
|   - GrepfuzzConfig                            |
|   - DetectorConfig                            |
| Functions:                                    |
|   - from_file(path) -> GrepfuzzConfig         |
|   - default() -> GrepfuzzConfig               |
| Data Out:                                     |
|   - GrepfuzzConfig struct                     |
+------------------------------------------------+
                  |
                  v
+------------------------------------------------+
|           Input/Source Layer                   |
|------------------------------------------------|
| Enums/Structs:                                |
|   - ImageSource (enum)                        |
| Functions:                                    |
|   - load_image(source) -> ImageBuffer         |
| Data Out:                                     |
|   - ImageBuffer<Luma<u8>, Vec<u8>>            |
+------------------------------------------------+
                  |
                  v
+------------------------------------------------+
|           Detection Layer                      |
|------------------------------------------------|
| Traits/Structs:                               |
|   - BlurDetector (trait)                      |
|   - LaplacianVarianceDetector                 |
|   - TenengradDetector                         |
|   - OpenCvLaplacianDetector                   |
| Functions/Methods:                            |
|   - detect(&self, img) -> (f64, bool)         |
|   - name(&self) -> &'static str               |
|   - as_any(&self) -> &dyn Any                 |
| Data Out:                                     |
|   - (metric_value: f64, is_blurry: bool)      |
+------------------------------------------------+
                  |
                  v
+------------------------------------------------+
|           Result Layer                         |
|------------------------------------------------|
| Structs:                                      |
|   - BlurResult                                |
| Data Out:                                     |
|   - Vec<BlurResult>                           |
+------------------------------------------------+
                  |
                  v
+------------------------------------------------+
|           Orchestration Layer                  |
|------------------------------------------------|
| Main Function:                                |
|   - main()                                    |
|   - process_image(path, detectors)            |
|   - CLI parsing (Cli struct)                  |
| Data Out:                                     |
|   - Output to stdout/logs                     |
+------------------------------------------------+
```

**Notes:**
- Each box lists the main structs, enums, and functions used in that layer.
- "Data Out" shows the main data type(s) passed to the next layer.
- This diagram gives programmers a quick reference to the key types and functions per layer.


# Elaboration of Detail

## Layered Architecture of `grepfuzz`

### 1. Configuration Layer
- **Purpose:** Handles configuration loading and defaults.
- **Key Structs:**
  ```rust
  pub struct GrepfuzzConfig {
      pub detectors: DetectorConfig,
  }

  pub struct DetectorConfig {
      pub laplacian_threshold: Option<f64>,
      pub tenengrad_threshold: Option<f64>,
      pub opencv_laplacian_threshold: Option<f64>,
      // Add more detector thresholds as needed
  }
  ```
- **Key Functions:**
  ```rust
  pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> { ... }
  fn default() -> Self { ... }
  ```
- **Role in Processing:** Loads config from file or uses defaults; merges with CLI/STDIN parameters in `main`.

---

### 2. Input/Source Layer
- **Purpose:** Abstracts image sources (file, stdin, synthetic).
- **Key Enum:**
  ```rust
  pub enum ImageSource {
      SyntheticCheckerboard { width: u32, height: u32 },
      SyntheticWhite { width: u32, height: u32 },
      File(String),
      Stdin,
  }
  ```
- **Key Functions:**
  ```rust
  pub fn load_image(source: ImageSource) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
      match source {
          ImageSource::SyntheticCheckerboard { width, height } => ...,
          ImageSource::SyntheticWhite { width, height } => ...,
          ImageSource::File(filename) => ...,
          ImageSource::Stdin => ...,
      }
  }
  ```
  - **`match` statements** are used to select the correct image loading strategy based on the source.

---

### 3. Detection Layer
- **Purpose:** Implements the `BlurDetector` trait for different algorithms.
- **Key Trait:**
  ```rust
  pub trait BlurDetector {
      fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool);
      fn as_any(&self) -> &dyn Any;
      fn name(&self) -> &'static str;
  }
  ```
- **Implementations:**
  - **LaplacianVarianceDetector**
    ```rust
    pub struct LaplacianVarianceDetector { pub threshold: f64 }
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) { ... }
    ```
  - **TenengradDetector**
    ```rust
    pub struct TenengradDetector { pub threshold: f64 }
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) { ... }
    ```
  - **OpenCvLaplacianDetector**
    ```rust
    pub struct OpenCvLaplacianDetector { pub threshold: f64 }
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) { ... }
    ```
- **Role in Processing:** Each detector processes the image and returns a metric and blurry status.

---

### 4. Result Layer
- **Purpose:** Stores and communicates the results of blur detection.
- **Key Struct:**
  ```rust
  pub struct BlurResult {
      pub name: String,
      pub value: f64,
      pub threshold: f64,
      pub is_blurry: bool,
  }
  ```
- **Role in Processing:** Each detector's result is stored in a `BlurResult` and collected for output.

---

### 5. Orchestration Layer (main.rs)
- **Purpose:** Coordinates the flow: parameter parsing, config merging, image loading, detection, and output.
- **Key Function:**
  ```rust
  fn main() -> io::Result<()> {
      // Parse CLI, load config, merge parameters
      // For each filename/source:
      //   Load image (match on ImageSource)
      //   For each BlurDetector:
      //     Call detect(), collect BlurResult
      //   Output results
  }
  ```
- **Loops & Match Usage:**
  - Outer `loop` for reading filenames/inputs.
  - Inner `for` loops for iterating over detectors and images.
  - `match` statements for config loading and image source handling.
  - Example:
    ```rust
    for det in detectors {
        let (val, is_blurry) = det.detect(&img);
        // collect results
    }
    ```

---

### 6. Testing & Synthetic Data
- **Purpose:** Provides synthetic images and tests for validation.
- **Example Test:**
  ```rust
  #[test]
  fn test_sharp_on_checkerboard() {
      let checkerboard_img = ImageBuffer::from_fn(...); // synthetic
      let (_variance, is_blurry) = analyze_blur_variance(&checkerboard_img, threshold);
      assert!(!is_blurry, "Checkerboard image should be classified as sharp");
  }
  ```

---

## Processing Flow (with Loops and Match)

1. **Startup:** Parse CLI args and/or read filenames from STDIN.
2. **Config:** Load config from file if present, else use defaults. (`match` on config source)
3. **Image Source:** For each filename, determine source (`match` on `ImageSource`).
4. **Image Loading:** Use `load_image` to get image data.
5. **Detection:** For each detector, call `detect()` and collect `BlurResult`. (`for` loop over detectors)
6. **Result Output:** Print or store results.
7. **Repeat:** Continue until all inputs are processed. (`loop` for input, `for` for detectors/images)

---

### Example: Core Processing Loop (main.rs, simplified)

```rust
loop {
    // Read next filename/source
    match process_image(path, &detectors) {
        Ok((is_blurry, results, size, width, height, focal)) => {
            for res in &results {
                println!("{}: {:.6} (thresh {:.3}) => {}", res.name, res.value, res.threshold, if res.is_blurry { "BLURRY" } else { "SHARP" });
            }
        }
        Err(e) => { eprintln!("Error: {}", e); }
    }
}
```

---

## Summary

- **Layered design:** Config → Input → Detection → Result → Orchestration.
- **Loops**: Used for iterating over inputs and detectors.
- **Match statements**: Used for config, image source, and result handling.
- **Extensible:** New detectors or sources can be added by implementing the relevant trait.

If you want this summary added to your `ARCHITECTURE.md` or want more detailed code examples for a specific layer, let me know!


# ASCII UML Architecture Diagram

```
+-----------------------------+
|      Configuration Layer    |
+-----------------------------+
              |
              v
+-----------------------------+
|      Input/Source Layer     |
+-----------------------------+
              |
              v
+-----------------------------+
|       Detection Layer       |
+-----------------------------+
              |
              v
+-----------------------------+
|        Result Layer         |
+-----------------------------+
              |
              v
+-----------------------------+
|    Orchestration Layer      |
+-----------------------------+
```

Each arrow represents the downward flow of data and control from configuration, through input, detection, result storage, and orchestration in the application.


#Authors
microuser
