use std::io::{self, BufRead, Write};
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use clap::Parser;
use exif::{In, Reader, Tag};
use image::imageops;
use image::{ImageBuffer, Luma};

#[derive(Parser)]
#[command(name = "blurdetect")]
#[command(about = "Detect blurry images from zero-terminated stdin input")]
struct Cli {
    /// Enable human-readable output
    #[arg(short = 'h', long = "human-readable")]
    human: bool,

    /// Blur threshold (variance below this is blurry)
    #[arg(short = 't', long, default_value_t = 100.0)]
    threshold: f64,
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();

    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut buffer = Vec::new();
    let mut stdout = io::stdout();

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

        match process_image(path, cli.threshold) {
            Ok((is_blurry, variance, size, width, height, focal)) => {
                if is_blurry {
                    if cli.human {
                        let focal_str = focal.unwrap_or_else(|| "N/A".to_string());
                        writeln!(
                            stdout,
                            "{}: BLURRY | Variance: {:.2} | Size: {} bytes | Resolution: {}x{} | Focal Length: {}",
                            path_str, variance, size, width, height, focal_str
                        )?;
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

fn extract_focal_length(path: &Path) -> Option<String> {
    let file = File::open(path).ok()?;
    let mut bufreader = BufReader::new(file);
    let exifreader = Reader::new();
    let exif = exifreader.read_from_container(&mut bufreader).ok()?;
    exif.get_field(Tag::FocalLength, In::PRIMARY)
        .map(|field| field.display_value().to_string())
}