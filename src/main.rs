use grepfuzz::process_image;

use grepfuzz::image_loader::{ImageInputMode, analyze_image_input};
use grepfuzz::detector_helpers;
use grepfuzz::output_helpers;

use clap::Parser;
use clap::CommandFactory;
use std::io;
use std::io::BufRead;
use std::io::Write;
use std::path::Path;






use grepfuzz::config::GrepfuzzConfig;

use grepfuzz::image_source_helpers::select_image_source;

use grepfuzz::cli::Cli;

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    let mut stdout = io::stdout();

    // Load config (now merged with CLI overrides)
    let config = GrepfuzzConfig::from_cli(&cli);
    let laplacian_threshold = config.detectors.laplacian_threshold.unwrap_or(0.1);
    let tenengrad_threshold = config.detectors.tenengrad_threshold.unwrap_or(1000.0);
    let opencv_laplacian_threshold = config.detectors.opencv_laplacian_threshold.unwrap_or(0.1);

    // Unified image input handling
    // use grepfuzz::image_loader::{analyze_image_input, ImageInputMode}; // Already imported at top
    let input_mode = if cli.synthetic_checkerboard {
        Some(ImageInputMode::SyntheticCheckerboard)
    } else if cli.synthetic_white {
        Some(ImageInputMode::SyntheticWhite)
    } else if cli.std_in_bytes {
        Some(ImageInputMode::StdinBytes)
    } else if let Some(ref filename) = cli.file {
        Some(ImageInputMode::File(filename.clone()))
    } else {
        None
    };

    let (input_mode_val, source, img_opt) = match input_mode {
        Some(mode) => match analyze_image_input(mode.clone(), &cli, laplacian_threshold) {
            Some((source, img)) => (Some(mode), source, Some(img)),
            None => {
                eprintln!("Error loading image");
                return Ok(());
            }
        },
        None => {
            // If -h/--help is passed, clap will print help and exit automatically.
            // If no stdin and no file argument, print help and exit.
            let _stdin = io::stdin();
            let is_stdin_tty = atty::is(atty::Stream::Stdin);
            if cli.file.is_none() && is_stdin_tty {
                // No file argument and no piped stdin: print help and exit
                Cli::command().print_help().unwrap();
                println!();
                return Ok(());
            }
            // fallback: use select_image_source
            let source = select_image_source(&cli)?;
            let img = grepfuzz::image_loader::load_image(source.clone())
                .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
            (None, source, None)
        }
    };


    // Handle verbose output for StdinBytes mode
    if let (Some(ImageInputMode::StdinBytes), Some(_source), Some(img)) = (input_mode_val.clone(), Some(source.clone()), img_opt.clone()) {
        let detectors = detector_helpers::build_detectors(laplacian_threshold, tenengrad_threshold, opencv_laplacian_threshold);
        let (is_blurry, results, size, width, height, focal) = grepfuzz::process_image_buffer(&img, detectors.as_slice());
        output_helpers::print_results(
            &mut stdout,
            is_blurry,
            results.as_slice(),
            size,
            width,
            height,
            &focal,
            "<stdin>",
            cli.verbose,
            cli.ascii,
        )?;
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
        let detectors = detector_helpers::build_detectors(laplacian_threshold, tenengrad_threshold, opencv_laplacian_threshold);

        match process_image(path, detectors.as_slice()) {
            Ok((is_blurry, results, size, width, height, focal)) => {
                output_helpers::print_results(
                    &mut stdout,
                    is_blurry,
                    results.as_slice(),
                    size,
                    width,
                    height,
                    &focal,
                    filename,
                    cli.verbose,
                    cli.ascii,
                )?;
                
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
        let mut detectors: Vec<Box<dyn grepfuzz::blur_detector::BlurDetector>> = Vec::new();
        detectors.push(Box::new(grepfuzz::blur_laplacian::LaplacianVarianceDetector::new(laplacian_threshold)));
        detectors.push(Box::new(grepfuzz::blur_tenengrad::TenengradDetector::new(tenengrad_threshold)));
        detectors.push(Box::new(grepfuzz::blur_opencv::OpenCvLaplacianDetector::new(opencv_laplacian_threshold)));
        match process_image(path, detectors.as_slice()) {
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

