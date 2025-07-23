use std::io::{self, BufRead, Write};

use std::path::Path;

use clap::{Parser, CommandFactory};
use rexif::{parse_file, ExifTag};
use image::imageops;
use image::{ImageBuffer, Luma};

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
#[derive(clap::ValueEnum, Clone, Debug)]
enum Mode {
    Blur,
    Sharp,
}

struct Cli {
    /// Input file to analyze
    #[arg(short, long)]
    file: Option<String>,


    /// Run in debug mode (synthetic images)
    #[arg(long)]
    debug: bool,

    /// Blur threshold
    #[arg(short = 't', long = "threshold")]
    threshold: Option<f64>,

    /// Filter mode: -b (blur-pass, default) or -s (sharp-pass)
    #[arg(short = 'b', long = "blur", default_value_t = true, conflicts_with = "sharp")]
    blur: bool,

    #[arg(short = 's', long = "sharp", default_value_t = false, conflicts_with = "blur")]
    sharp: bool,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut stdout = io::stdout();

    // Use threshold from CLI or default
    let threshold = cli.threshold.unwrap_or(0.1);

    // Debug mode: synthetic images
    if cli.debug {
        use image::{ImageBuffer, Luma};
        println!("[DEBUG] Running in debug mode: generating synthetic images");
        // Static (random noise) image
        let width = 256;
        let height = 256;
        let mut rng = rand::thread_rng();
        let static_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_fn(width, height, |_x, _y| {
            Luma([rand::Rng::gen::<u8>(&mut rng)])
        });
        let white_img: ImageBuffer<Luma<u8>, Vec<u8>> = ImageBuffer::from_pixel(width, height, Luma([255]));
        println!("[DEBUG] Analyzing static noise image...");
        debug_blur_analysis(&static_img, threshold);
        println!("[DEBUG] Analyzing pure white image...");
        debug_blur_analysis(&white_img, threshold);
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
        let path = Path::new(&filename);
        match process_image(path, threshold) {
            Ok((is_blurry, variance, size, width, height, focal)) => {
                println!("File: {}\n  Size: {} bytes\n  Dimensions: {}x{}\n  Blurry: {}\n  Variance: {:.6}\n  Focal Length: {}\n", path.display(), size, width, height, is_blurry, variance, focal.unwrap_or("N/A".to_string()));
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", filename, e);
            }
        }
        return Ok(());
    }
    

    // Otherwise, process stdin as before
    let mut reader = stdin.lock();
    let mut buffer = Vec::new();
    let blur_mode = cli.blur || (!cli.blur && !cli.sharp); // default to blur if neither specified
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

        match process_image(path, threshold) {
            Ok((is_blurry, _variance, _size, _width, _height, _focal)) => {
                if (blur_mode && is_blurry) || (!blur_mode && !is_blurry) {
                    stdout.write_all(path_str.as_bytes())?;
                    stdout.write_all(&[0])?;
                }
            }
            Err(e) => {
                eprintln!("Error processing {}: {}", path_str, e);
            }
        }
    }
    Ok(())
}

fn process_image(
    path: &Path,
    threshold: f64,
) -> Result<(bool, f64, u64, u32, u32, Option<String>), Box<dyn std::error::Error>> {
    // Load image and convert to grayscale u8
    let img = image::open(path)?.grayscale().to_luma8();
    let width = img.width();
    let height = img.height();

    // Convert to f32 for filtering (without normalization)
    let img_f32: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_fn(width, height, |x, y| {
        Luma([img.get_pixel(x, y)[0] as f32])
    });

    // Apply Laplacian kernel
    let kernel = [0f32, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
    let lap: ImageBuffer<Luma<f32>, Vec<f32>> = imageops::filter3x3(&img_f32, &kernel);

    // Compute variance
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

    // File size
    let size = std::fs::metadata(path)?.len();

    // EXIF focal length
    let focal = extract_focal_length(path);

    Ok((is_blurry, variance, size, width, height, focal))
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

    // Helper for testing: like debug_blur_analysis but returns values
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