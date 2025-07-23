use image::{ImageBuffer, Luma, ImageReader};
use std::fs::File;
use std::io::{self, Read};

pub enum ImageSource {
    SyntheticCheckerboard { width: u32, height: u32 },
    SyntheticWhite { width: u32, height: u32 },
    File(String),
    Stdin,
}

pub fn load_image(source: ImageSource) -> Result<ImageBuffer<Luma<u8>, Vec<u8>>, String> {
    match source {
        ImageSource::SyntheticCheckerboard { width, height } => {
            Ok(ImageBuffer::from_fn(width, height, |x, y| {
                if (x + y) % 2 == 0 { Luma([0]) } else { Luma([255]) }
            }))
        }
        ImageSource::SyntheticWhite { width, height } => {
            Ok(ImageBuffer::from_pixel(width, height, Luma([255])))
        }
        ImageSource::File(filename) => {
            let img = ImageReader::open(&filename)
                .map_err(|e| format!("Failed to open file {}: {}", filename, e))?
                .decode()
                .map_err(|e| format!("Failed to decode image {}: {}", filename, e))?
                .to_luma8();
            Ok(img)
        }
        ImageSource::Stdin => {
            let mut buf = Vec::new();
            io::stdin().read_to_end(&mut buf).map_err(|e| format!("Failed to read stdin: {}", e))?;
            let img = image::load_from_memory(&buf)
                .map_err(|e| format!("Failed to decode image from stdin: {}", e))?
                .to_luma8();
            Ok(img)
        }
    }
}
