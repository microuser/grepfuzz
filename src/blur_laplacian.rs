use image::{ImageBuffer, Luma, imageops};
use crate::BlurDetector;

pub struct LaplacianVarianceDetector {
    pub threshold: f64,
}

impl BlurDetector for LaplacianVarianceDetector {
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) {
        let width = img.width();
        let height = img.height();
        let img_f32: ImageBuffer<Luma<f32>, Vec<f32>> = ImageBuffer::from_fn(width, height, |x, y| {
            Luma([img.get_pixel(x, y)[0] as f32])
        });
        let kernel = [0f32, 1.0, 0.0, 1.0, -4.0, 1.0, 0.0, 1.0, 0.0];
        let lap = imageops::filter3x3(&img_f32, &kernel);
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
        let is_blurry = variance < self.threshold;
        (variance, is_blurry)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
