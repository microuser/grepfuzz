use crate::BlurDetector;
use image::{ImageBuffer, Luma};
use std::any::Any;

// Requires the opencv crate in Cargo.toml and OpenCV installed on system.
// [dependencies]
// opencv = "0.87"

pub struct OpenCvLaplacianDetector {
    pub threshold: f64,
}

impl BlurDetector for OpenCvLaplacianDetector {
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) {
        use opencv::{core, imgproc, prelude::*};
        // Convert image::ImageBuffer to OpenCV Mat
        let (width, height) = (img.width() as i32, img.height() as i32);
        // Convert image::ImageBuffer to OpenCV Mat
        let (width, height) = (img.width() as i32, img.height() as i32);
        let mut mat = core::Mat::new_rows_cols(height, width, core::CV_8UC1).expect("Mat allocation failed");
        for y in 0..height {
            for x in 0..width {
                *mat.at_2d_mut::<u8>(y, x).unwrap() = img.get_pixel(x as u32, y as u32)[0];
            }
        }
        // Apply Laplacian
        let mut dst = core::Mat::default();
        imgproc::laplacian(
            &mat,
            &mut dst,
            core::CV_64F,
            3,
            1.0,
            0.0,
            core::BORDER_DEFAULT,
        ).expect("Laplacian failed");
        // Compute variance
        let mean = core::mean(&dst, &core::no_array()).unwrap().0[0];
        let mut sum_sq = 0.0;
        let mut count = 0;
        for val in dst.data_typed::<f64>().unwrap() {
            sum_sq += (*val - mean).powi(2);
            count += 1;
        }
        let variance = if count > 0 { sum_sq / count as f64 } else { 0.0 };
        let is_blurry = variance < self.threshold;
        (variance, is_blurry)
    }
    fn as_any(&self) -> &dyn Any {
        self
    }
}
