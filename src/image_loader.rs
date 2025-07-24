use image::{ImageBuffer, Luma, ImageReader};
use std::io::{self, Read};

pub enum ImageSource {
    SyntheticCheckerboard { width: u32, height: u32 },
    SyntheticWhite { width: u32, height: u32 },
    File(String),
    Stdin,
}

pub fn load_image(source: ImageSource) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
    match source {
        ImageSource::SyntheticCheckerboard { width, height } => ImageSource::from_checkerboard(width, height),
        ImageSource::SyntheticWhite { width, height } => ImageSource::from_white(width, height),
        ImageSource::File(filename) => ImageSource::from_file(&filename),
        ImageSource::Stdin => ImageSource::from_stdin_bytes(),
    }
}

impl ImageSource {
    pub fn from_file(filename: &str) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
        let img = ImageReader::open(filename)
            .map_err(|e| format!("Failed to open file {}: {}", filename, e))?
            .decode()
            .map_err(|e| format!("Failed to decode image {}: {}", filename, e))?
            .to_luma8();
        Ok(img)
    }
    pub fn from_stdin_bytes() -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
        let mut buf = Vec::new();
        io::stdin().read_to_end(&mut buf).map_err(|e| format!("Failed to read stdin: {}", e))?;
        let img = image::load_from_memory(&buf)
            .map_err(|e| format!("Failed to decode image from stdin: {}", e))?
            .to_luma8();
        Ok(img)
    }
    pub fn from_checkerboard(width: u32, height: u32) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
        Ok(ImageBuffer::from_fn(width, height, |x, y| {
            if (x + y) % 2 == 0 { Luma([0]) } else { Luma([255]) }
        }))
    }
    pub fn from_white(width: u32, height: u32) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
        Ok(ImageBuffer::from_pixel(width, height, Luma([255])))
    }
}

