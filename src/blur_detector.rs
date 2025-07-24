use image::{ImageBuffer, Luma};
use std::any::Any;

pub trait BlurDetector {
    /// Returns (metric_value, is_blurry)
    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool);
    fn as_any(&self) -> &dyn Any;
    fn name(&self) -> &'static str;
}
