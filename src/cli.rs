use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// Input file to analyze
    #[arg(short, long, conflicts_with_all = ["synthetic_checkerboard", "synthetic_white", "passthrough"])]
    pub file: Option<String>,

    /// Generate and analyze a synthetic checkerboard image
    #[arg(long = "synthetic-checkerboard", conflicts_with_all = ["file", "synthetic_white", "passthrough"])]
    pub synthetic_checkerboard: bool,

    /// Generate and analyze a synthetic solid white image
    #[arg(long = "synthetic-white", conflicts_with_all = ["file", "synthetic_checkerboard", "passthrough"])]
    pub synthetic_white: bool,

    /// Verbose (human-readable debug) output
    #[arg(short = 'v', long = "verbose", default_value_t = false)]
    pub verbose: bool,

    /// Blur threshold
    #[arg(short = 't', long = "threshold")]
    pub threshold: Option<f64>,

    /// Filter mode: -b (blur-pass, default) or -s (sharp-pass)
    #[arg(short = 'b', long = "blur", default_value_t = true, conflicts_with = "sharp")]
    pub blur: bool,

    #[arg(short = 's', long = "sharp", default_value_t = false, conflicts_with = "blur")]
    pub sharp: bool,

    /// ASCII output: print all details for each file in a human-readable format
    #[arg(short = 'a', long = "ascii", default_value_t = false)]
    pub ascii: bool,

    /// Passthrough mode: output stdin to stdout with zero-terminated records
    #[arg(short = 'p', long = "passthrough", default_value_t = false, conflicts_with_all = ["file", "synthetic_checkerboard", "synthetic_white"])]
    pub passthrough: bool,

    /// Read a single image from stdin as bytes
    #[arg(short = 'B', long = "std_in_bytes", default_value_t = false, conflicts_with_all = ["file", "synthetic_checkerboard", "synthetic_white", "passthrough"])]
    pub std_in_bytes: bool,

    /// Tenengrad (Sobel) sharpness threshold
    #[arg(long = "tenengrad-threshold")]
    pub tenengrad_threshold: Option<f64>,

    /// OpenCV Laplacian threshold
    #[arg(long = "opencv-laplacian-threshold")]
    pub opencv_laplacian_threshold: Option<f64>,

    /// Config file path
    #[arg(long = "config")]
    pub config: Option<String>,
}

#[derive(clap::ValueEnum, Clone, Debug)]
pub enum Mode {
    Blur,
    Sharp,
}
