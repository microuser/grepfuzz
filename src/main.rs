mod blur_detector;
mod blur_laplacian;
mod blur_tenengrad;
mod blur_opencv;
mod config;
use clap::Parser;
use std::io;
use std::path::Path;
use image::imageops;
use crate::image_loader::{load_image, ImageSource};
use crate::blur_result::BlurResult;
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
        let img = load_image(ImageSource::SyntheticCheckerboard { width: 256, height: 256 })?;
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
        let img = load_image(ImageSource::SyntheticWhite { width: 256, height: 256 })?;
        if cli.verbose {
            println!("[VERBOSE] Analyzing synthetic white image...");
            debug_blur_analysis(&img, laplacian_threshold);
        } else {
            let (variance, is_blurry) = analyze_blur_variance(&img, laplacian_threshold);
            println!("White: blurry={} variance={:.6}", is_blurry, variance);
        }
        return Ok(());
    }

    // File or stdin loader
    let img = if let Some(filename) = cli.file {
        load_image(ImageSource::File(filename))?
    } else {
        load_image(ImageSource::Stdin)?
    };
    if cli.synthetic_white {
        let width = 256;
        let height = 256;
        let white_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Luma([255]));
        if cli.verbose {
            println!("[VERBOSE] Analyzing synthetic white image...");
            debug_blur_analysis(&white_img, threshold);
        } else {
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
    if let Some(filename) = cli.file {
        let path = std::path::Path::new(&filename);
        let mut detectors: Vec<Box<dyn BlurDetector>> = vec![
            Box::new(LaplacianVarianceDetector { threshold: laplacian_threshold }),
            Box::new(blur_tenengrad::TenengradDetector { threshold: tenengrad_threshold }),
            Box::new(blur_opencv::OpenCvLaplacianDetector { threshold: opencv_laplacian_threshold }),
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
                    println!("[VERBOSE] Blurry (all detectors): {}", is_blurry);
                    println!("[VERBOSE] Focal Length: {}", focal.clone().unwrap_or("N/A".to_string()));
                } else {
                    println!("File: {}\n  Size: {} bytes\n  Dimensions: {}x{}\n  Blurry: {}\n  Focal Length: {}\n", path.display(), size, width, height, is_blurry, focal.unwrap_or("N/A".to_string()));
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
        match process_image(path, threshold, cli.tenengrad_threshold) {
            Ok((is_blurry, variance, tenengrad, size, width, height, focal)) => {
                if (blur_mode && is_blurry) || (!blur_mode && !is_blurry) {
                    if cli.ascii {
                        let sharpness = if is_blurry { "blurry" } else { "sharp" };
                        println!(
                            "File: {}\n  Size: {} bytes\n  Dimensions: {}x{}\n  Blurry: {}\n  Laplacian Variance: {:.6}\n  Tenengrad: {:.2}\n  Focal Length: {}\n",
                            path.display(), size, width, height, sharpness, variance, tenengrad, focal.unwrap_or("N/A".to_string())
                        );
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

fn tenengrad_sharpness(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> f64 {
    let sobel_x = imageops::filter3x3(img, &[-1.0, 0.0, 1.0,
                                             -2.0, 0.0, 2.0,
                                             -1.0, 0.0, 1.0]);
    let sobel_y = imageops::filter3x3(img, &[-1.0, -2.0, -1.0,
                                              0.0,  0.0,  0.0,
                                              1.0,  2.0,  1.0]);
    let mut sum = 0.0;
    for (x, y, pixel) in sobel_x.enumerate_pixels() {
        let gx = pixel[0] as f64;
        let gy = sobel_y.get_pixel(x, y)[0] as f64;
        sum += gx * gx + gy * gy;
    }
    sum / (img.width() as f64 * img.height() as f64)
}

fn process_image(
    path: &std::path::Path,
    detectors: &[Box<dyn BlurDetector>],
) -> Result<(bool, Vec<BlurResult>, u64, u32, u32, Option<String>), Box<dyn std::error::Error>> {
    // Load image and convert to grayscale u8
    let img = image::open(path)?.grayscale().to_luma8();
    let width = img.width();
    let height = img.height();

    let mut results = Vec::new();
    let mut all_blurry = true;
    for det in detectors {
        let (val, is_blurry) = det.detect(&img);
        let name = std::any::type_name::<&Box<dyn BlurDetector>>().to_string(); // Placeholder for now
        let threshold = if let Some(l) = det.as_any().downcast_ref::<LaplacianVarianceDetector>() {
            l.threshold
        } else if let Some(t) = det.as_any().downcast_ref::<TenengradDetector>() {
            t.threshold
        } else { 0.0 };
        results.push(BlurResult { name, value: val, threshold, is_blurry });
        all_blurry = all_blurry && is_blurry;
    }

    // File size
    let size = std::fs::metadata(path)?.len();
    let focal = extract_focal_length(path);

    Ok((all_blurry, results, size, width, height, focal))
}

fn debug_blur_analysis(img: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) {
    println!("[DEBUG] Image size: {}x{}", img.width(), img.height());
    let width = img.width();
    let height = img.height();
    // Convert to f32 for filtering (without normalization)
    let img_f32: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_fn(width, height, |x, y| {
        Luma([img.get_pixel(x, y)[0] as f32])
    });
    println!("[DEBUG] Converted to f32 grayscale");
    // Apply Laplacian kernel
    let kernel = [0f32, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
    let lap: ImageBuffer<Luma<f32>, Vec<f32>> = imageops::filter3x3(&img_f32, &kernel);
    println!("[DEBUG] Applied Laplacian kernel");
    // Compute variance
    let pixels = lap.into_vec();
    let n = pixels.len() as f64;
    let mut mean = 0.0f64;
    for &p in &pixels {
        mean += p as f64;
    }
    mean /= n;
    println!("[DEBUG] Mean: {:.6}", mean);
    let mut variance = 0.0f64;
    for &p in &pixels {
        variance += (p as f64 - mean).powi(2);
    }
    variance /= n;
    println!("[DEBUG] Variance: {:.6}", variance);
    println!("[DEBUG] Threshold: {:.2}", threshold);
    let is_blurry = variance < threshold;
    println!("[DEBUG] Result: {}", if is_blurry {"BLURRY"} else {"SHARP"});
}

// Helper for testing and main: like debug_blur_analysis but returns values
fn analyze_blur_variance(img: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) -> (f64, bool) {
    let width = img.width();
    let height = img.height();
    let img_f32: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_fn(width, height, |x, y| {
        Luma([img.get_pixel(x, y)[0] as f32])
    });
    let kernel = [0f32, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
    let lap: ImageBuffer<Luma<f32>, Vec<f32>> = imageops::filter3x3(&img_f32, &kernel);
    let pixels = lap.into_vec();
    let n = pixels.len() as f64;
    let mut mean = 0.0f64;
    for &p in &pixels {
        mean += p as f64;
    }
    mean /= n;
    let mut variance = 0.0f64;
    for &p in &pixels {
        variance += (p as f64 - mean).powi(2);
    }
    variance /= n;
    let is_blurry = variance < threshold;
    (variance, is_blurry)
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{ImageBuffer, Luma};

    #[test]
    fn test_blur_on_solid_white() {
        // Create a 100x100 solid white image
        let width = 100;
        let height = 100;
        let white_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Luma([255]));
        let threshold = 100.0;
        let (variance, is_blurry) = analyze_blur_variance(&white_img, threshold);
        assert!(variance.abs() < 1e-6, "Variance should be near zero for solid white, got {}", variance);
        assert!(is_blurry, "Solid white image should be classified as blurry");
    }

    #[test]
    fn test_sharp_on_checkerboard() {
        // Create a 100x100 checkerboard image
        let width = 100;
        let height = 100;
        let checkerboard_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
            if (x + y) % 2 == 0 {
                Luma([0]) // black
            } else {
                Luma([255]) // white
            }
        });
        let threshold = 0.1;
        let (_variance, is_blurry) = analyze_blur_variance(&checkerboard_img, threshold);
        assert!(!is_blurry, "Checkerboard image should be classified as sharp");
    }

    #[test]
    fn test_sharp_on_large_checkerboard() {
        // Create a 100x100 checkerboard with 10x10 pixel squares
        let width = 100;
        let height = 100;
        let block = 10;
        let checkerboard_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |x, y| {
            if ((x / block) + (y / block)) % 2 == 0 {
                Luma([0]) // black
            } else {
                Luma([255]) // white
            }
        });
        let threshold = 0.1;
        let (_variance, is_blurry) = analyze_blur_variance(&checkerboard_img, threshold);
        assert!(!is_blurry, "Large-block checkerboard should be classified as sharp");
    }

// Helper for testing and main: like debug_blur_analysis but returns values
fn analyze_blur_variance(img: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) -> (f64, bool) {
    let width = img.width();
    let height = img.height();
    let img_f32: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_fn(width, height, |x, y| {
        Luma([img.get_pixel(x, y)[0] as f32])
    });
    let kernel = [0f32, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
    let lap: ImageBuffer<Luma<f32>, Vec<f32>> = imageops::filter3x3(&img_f32, &kernel);
    let pixels = lap.into_vec();
    let n = pixels.len() as f64;
    let mut mean = 0.0f64;
    for &p in &pixels {
        mean += p as f64;
    }
    mean /= n;
    let mut variance = 0.0f64;
    for &p in &pixels {
        variance += (p as f64 - mean).powi(2);
    }
    variance /= n;
    let is_blurry = variance < threshold;
    (variance, is_blurry)
}

}

fn extract_focal_length(path: &Path) -> Option<String> {
    let exif = parse_file(path).ok()?;
    for entry in exif.entries {
        if entry.tag == ExifTag::FocalLength {
            return Some(entry.value_more_readable.to_string());
        }
    }
    None
}