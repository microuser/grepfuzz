use image::{ImageBuffer, Luma, imageops};

pub fn debug_blur_analysis(img: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) {
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
    let lap = imageops::filter3x3(&img_f32, &kernel);
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

pub fn analyze_blur_variance(img: &ImageBuffer<Luma<u8>, Vec<u8>>, threshold: f64) -> (f64, bool) {
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

pub fn tenengrad_sharpness(img: &ImageBuffer<Luma<u8>, Vec<u8>>) -> f64 {
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
