// lib.rs
// This module will contain the core logic refactored from main.rs

// Export modules for public use
pub mod image_analysis;
pub mod image_loader;
pub mod metadata;
pub mod blur_detector;
pub mod blur_laplacian;
pub mod blur_opencv;
pub mod blur_result;
pub mod blur_tenengrad;
pub mod config;

use std::path::Path;
use blur_detector::BlurDetector;
use blur_laplacian::LaplacianVarianceDetector;
use blur_tenengrad::TenengradDetector;
use blur_opencv::OpenCvLaplacianDetector;
use blur_result::BlurResult;
use image::ImageBuffer;

/// Processes an image at the given path using the provided blur detectors.
pub fn process_image(
    path: &Path,
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
        let name = det.name().to_string();
        let threshold = if let Some(l) = det.as_any().downcast_ref::<LaplacianVarianceDetector>() {
            l.threshold
        } else if let Some(t) = det.as_any().downcast_ref::<TenengradDetector>() {
            t.threshold
        } else if let Some(o) = det.as_any().downcast_ref::<OpenCvLaplacianDetector>() {
            o.threshold
        } else { 0.0 };
        results.push(BlurResult { name, value: val, threshold, is_blurry });
        all_blurry = all_blurry && is_blurry;
    }

    // File size
    let size = std::fs::metadata(path)?.len();
    let focal = crate::metadata::extract_focal_length(path);

    Ok((all_blurry, results, size, width, height, focal))
}
