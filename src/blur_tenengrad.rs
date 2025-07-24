use image::{ImageBuffer, Luma, imageops};
use crate::BlurDetector;

pub struct TenengradDetector {
    pub threshold: f64,
}

impl TenengradDetector {
    pub fn new(threshold: f64) -> Self {
        Self { threshold }
    }
}


impl BlurDetector for TenengradDetector {
    fn name(&self) -> &'static str {
        "Tenengrad"
    }

    fn detect(&self, img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> (f64, bool) {
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
        let val = sum / (img.width() as f64 * img.height() as f64);
        let is_blurry = val < self.threshold;
        (val, is_blurry)
    }
    fn as_any(&self) -> &dyn std::any::Any {
        self
    }
}
