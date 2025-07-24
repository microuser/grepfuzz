use crate::cli::Cli;
use crate::image_loader::ImageSource;
use std::io;

/// Selects the image source based on CLI arguments.
pub fn select_image_source(cli: &Cli) -> Result<ImageSource, io::Error> {
    if cli.synthetic_checkerboard {
        Ok(ImageSource::SyntheticCheckerboard { width: 256, height: 256 })
    } else if cli.synthetic_white {
        Ok(ImageSource::SyntheticWhite { width: 256, height: 256 })
    } else if let Some(ref filename) = cli.file {
        Ok(ImageSource::File(filename.clone()))
    } else if cli.std_in_bytes {
        Ok(ImageSource::Stdin)
    } else {
        // Default to stdin if nothing else specified
        Ok(ImageSource::Stdin)
    }
}
