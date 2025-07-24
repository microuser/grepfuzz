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
}
