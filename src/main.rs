mod blur_detector;
mod blur_laplacian;
mod blur_tenengrad;
mod blur_opencv;
mod config;
mod image_loader;
mod blur_result;

use grepfuzz::process_image; // Use process_image from lib.rs
use clap::Parser;
use clap::CommandFactory;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;
use image::imageops;
use blur_result::BlurResult;
use rexif::{parse_file, ExifTag};

use blur_detector::BlurDetector;
use blur_laplacian::LaplacianVarianceDetector;
use blur_tenengrad::TenengradDetector;
use blur_opencv::OpenCvLaplacianDetector;

use image::{ImageBuffer, Luma};
use config::GrepfuzzConfig;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Input file to analyze
    #[arg(short, long, conflicts_with_all = ["synthetic_checkerboard", "synthetic_white", "passthrough"])]
    file: Option<String>,

    /// Generate and analyze a synthetic checkerboard image
    #[arg(long = "synthetic-checkerboard", conflicts_with_all = ["file", "synthetic_white", "passthrough"])]
    synthetic_checkerboard: bool,

    /// Generate and analyze a synthetic solid white image
    #[arg(long = "synthetic-white", conflicts_with_all = ["file", "synthetic_checkerboard", "passthrough"])]
    synthetic_white: bool,

    /// Verbose (human-readable debug) output
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    verbose: bool,

    /// Blur threshold
    #[arg(short = 't', long = "threshold")]
    threshold: Option<f64>,

    /// Filter mode: -b (blur-pass, default) or -s (sharp-pass)
    #[arg(short = 'b', long = "blur", default_value_t = true, conflicts_with = "sharp")]
    blur: bool,

    #[arg(short = 's', long = "sharp", default_value_t = false, conflicts_with = "blur")]
    sharp: bool,

    /// ASCII output: print all details for each file in a human-readable format
    #[arg(short = 'a', long = "ascii", default_value_t = false)]
    ascii: bool,

    /// Passthrough mode: output stdin to stdout with zero-terminated records
    #[arg(short = 'p', long = "passthrough", default_value_t = false, conflicts_with_all = ["file", "synthetic_checkerboard", "synthetic_white"])]
    passthrough: bool,

    /// Read a single image from stdin as bytes
    #[arg(short = 'B', long = "std_in_bytes", default_value_t = false, conflicts_with_all = ["file", "synthetic_checkerboard", "synthetic_white", "passthrough"])]
    std_in_bytes: bool,

    /// Tenengrad (Sobel) sharpness threshold
    #[arg(long = "tenengrad-threshold")]
    tenengrad_threshold: Option<f64>,

    /// OpenCV Laplacian threshold
    #[arg(long = "opencv-laplacian-threshold")]
    opencv_laplacian_threshold: Option<f64>,

    /// Config file path
    #[arg(long = "config")]
    config: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Blur,
    Sharp,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut stdout = io::stdout();

    // Load config
    let config = if let Some(ref path) = cli.config {
        match GrepfuzzConfig::from_file(path) {
            Ok(cfg) => cfg,
            Err(e) => {
                eprintln!("Failed to load config: {}. Using defaults.", e);
                GrepfuzzConfig::default()
            }
        }
    } else {
        GrepfuzzConfig::default()
    };

    // Use config values unless overridden by CLI
    let laplacian_threshold = cli.threshold.or(config.detectors.laplacian_threshold).unwrap_or(0.1);
    let tenengrad_threshold = cli.tenengrad_threshold.or(config.detectors.tenengrad_threshold).unwrap_or(1000.0);
    let opencv_laplacian_threshold = cli.opencv_laplacian_threshold.or(config.detectors.opencv_laplacian_threshold).unwrap_or(0.1);

    // Synthetic image: checkerboard
    if cli.synthetic_checkerboard {
        let img = image_loader::load_image(image_loader::ImageSource::SyntheticCheckerboard { width: 256, height: 256 })
    .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if cli.verbose {
            println!("[VERBOSE] Analyzing synthetic checkerboard image...");
            debug_blur_analysis(&img, laplacian_threshold);
        } else {
            let (variance, is_blurry) = analyze_blur_variance(&img, laplacian_threshold);
            println!("Checkerboard: blurry={} variance={:.6}", is_blurry, variance);
        }
        return Ok(());
    }

    // Synthetic image: solid white
    if cli.synthetic_white {
        let img = image_loader::ImageSource::from_white(256, 256)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if cli.verbose {
            println!("[VERBOSE] Analyzing synthetic white image...");
            debug_blur_analysis(&img, laplacian_threshold);
        } else {
            let (variance, is_blurry) = analyze_blur_variance(&img, laplacian_threshold);
            println!("White: blurry={} variance={:.6}", is_blurry, variance);
        }
        return Ok(());
    }

    // --std_in_bytes: Read a single image from stdin as bytes
    if cli.std_in_bytes {
        let img = image_loader::ImageSource::from_stdin_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        if cli.verbose {
            println!("[VERBOSE] Analyzing image from stdin (bytes mode)...");
            debug_blur_analysis(&img, cli.threshold.unwrap_or(0.1));
        } else {
            let (variance, is_blurry) = analyze_blur_variance(&img, cli.threshold.unwrap_or(0.1));
            println!("Stdin image: blurry={} variance={:.6}", is_blurry, variance);
        }
        return Ok(());
    }

    // File or stdin loader
    let img = if let Some(ref filename) = cli.file {
        image_loader::ImageSource::from_file(filename)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    } else {
        // Remove or repurpose this block if not using ImageSource::Stdin for null-terminated filenames
        image_loader::ImageSource::from_stdin_bytes()
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?
    };

    if cli.synthetic_white {
        let width = 256;
        let height = 256;
        let white_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Luma([255]));
        if cli.verbose {
            println!("[VERBOSE] Analyzing synthetic white image...");
            let threshold = cli.threshold.unwrap_or(0.1);
            debug_blur_analysis(&white_img, threshold);
        } else {
            let threshold = cli.threshold.unwrap_or(0.1);
            let (variance, is_blurry) = analyze_blur_variance(&white_img, threshold);
            println!("White: blurry={} variance={:.6}", is_blurry, variance);
        }
        return Ok(());
    }

    // If -h/--help is passed, clap will print help and exit automatically.
    // If no stdin and no file argument, print help and exit.
    let stdin = io::stdin();
    let is_stdin_tty = atty::is(atty::Stream::Stdin);

    if cli.file.is_none() && is_stdin_tty {
        // No file argument and no piped stdin: print help and exit
        Cli::command().print_help().unwrap();
        println!();
        return Ok(());
    }

    // If file argument is provided, process that file
    if let Some(ref filename) = cli.file {
        let path = std::path::Path::new(&filename);
        let detectors: Vec<Box<dyn BlurDetector>> = vec![
            Box::new(LaplacianVarianceDetector { threshold: laplacian_threshold }),
            Box::new(TenengradDetector { threshold: tenengrad_threshold }),
            Box::new(OpenCvLaplacianDetector::new(55.0)),
        ];

        match process_image(path, &detectors) {
            Ok((is_blurry, results, size, width, height, focal)) => {
                if cli.verbose || cli.ascii {
                    println!("[VERBOSE] File: {}", path.display());
                    println!("[VERBOSE] Size: {} bytes", size);
                    println!("[VERBOSE] Dimensions: {}x{}", width, height);
                    for res in &results {
                        println!("[VERBOSE] {}: {:.6} (thresh {:.3}) => {}", res.name, res.value, res.threshold, if res.is_blurry { "BLURRY" } else { "SHARP" });
                    }
                    let tenengrad_val = tenengrad_sharpness(&img);
                    println!("[VERBOSE] Tenengrad sharpness: {:.6}", tenengrad_val);
                    println!("[VERBOSE] Blurry (all detectors): {}", is_blurry);
                    println!("[VERBOSE] Focal Length: {}", focal.clone().unwrap_or("N/A".to_string()));
                } else {
                    let tenengrad_val = tenengrad_sharpness(&img);
                    println!("File: {}\n  Size: {} bytes\n  Dimensions: {}x{}\n  Blurry: {}\n  Tenengrad: {:.6}\n  Focal Length: {}", path.display(), size, width, height, if is_blurry { "BLURRY" } else { "SHARP" }, tenengrad_val, focal.clone().unwrap_or("N/A".to_string()));
for res in &results {
    println!("  Detector: {} | Value: {:.6} | Threshold: {:.3} | Result: {}", res.name, res.value, res.threshold, if res.is_blurry { "BLURRY" } else { "SHARP" });
}

                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", filename, e);
            }
        }
        return Ok(());
    }

    // Passthrough mode: copy stdin to stdout, zero-terminated, then print newline and clear buffer
    if cli.passthrough {
        let mut reader = stdin.lock();
        let mut buffer = Vec::new();
        loop {
            buffer.clear();
            let bytes_read = reader.read_until(b'\0', &mut buffer)?;
            if bytes_read == 0 {
                break;
            }
            if !buffer.is_empty() {
                stdout.write_all(&buffer)?;
            }
        }
        buffer.clear();
        return Ok(());
    }

    // Otherwise, process stdin as before
    let mut reader = stdin.lock();
    let mut buffer = Vec::new();
    let blur_mode = cli.blur || (!cli.blur && !cli.sharp); // default to blur if neither specified
    // ... (rest unchanged)
    loop {
        buffer.clear();
        let bytes_read = reader.read_until(b'\0', &mut buffer)?;
        if bytes_read == 0 {
            break;
        }
        if buffer.last() == Some(&b'\0') {
            buffer.pop();
        }
        let path_str = match String::from_utf8(buffer.clone()) {
            Ok(s) => s,
            Err(_) => continue,
        };
        let path = Path::new(&path_str);
        // Recreate detectors for each file if needed (or reuse from above)
        let mut detectors: Vec<Box<dyn blur_detector::BlurDetector>> = Vec::new();
        detectors.push(Box::new(blur_laplacian::LaplacianVarianceDetector::new(laplacian_threshold)));
        detectors.push(Box::new(blur_tenengrad::TenengradDetector::new(tenengrad_threshold)));
        detectors.push(Box::new(blur_opencv::OpenCvLaplacianDetector::new(opencv_laplacian_threshold)));
        match process_image(path, &detectors) {
            Ok((is_blurry, results, size, width, height, _focal)) => {
                if (blur_mode && is_blurry) || (!blur_mode && !is_blurry) {
                    if cli.ascii {
                        // Print all detector results in ASCII/TSV style
for res in &results {
    println!("{}\t{}\t{}\t{}\t{}\t{:.6}\t{:.3}\t{}", path.display(), size, width, height, res.name, res.value, res.threshold, if res.is_blurry { "BLURRY" } else { "SHARP" });
}

                    } else {
                        stdout.write_all(path_str.as_bytes())?;
                        stdout.write_all(&[0])?;
                    }
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", path_str, e);
            }
        }
    }
    Ok(())
}

