use std::fs;
use std::path::Path;
use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct DetectorConfig {
    pub laplacian_threshold: Option<f64>,
    pub tenengrad_threshold: Option<f64>,
    pub opencv_laplacian_threshold: Option<f64>,
    // Add more detector thresholds as needed
}

#[derive(Debug, Deserialize, Clone)]
pub struct GrepfuzzConfig {
    pub detectors: DetectorConfig,
}

impl Default for GrepfuzzConfig {
    fn default() -> Self {
        GrepfuzzConfig {
            detectors: DetectorConfig {
                laplacian_threshold: Some(0.2),
                tenengrad_threshold: Some(100.0),
                opencv_laplacian_threshold: Some(55.0),
            },
        }
    }
}

impl GrepfuzzConfig {
    pub fn from_file<P: AsRef<Path>>(path: P) -> Result<Self, String> {
        let content = fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read config file: {}", e))?;
        toml::from_str(&content)
            .map_err(|e| format!("Failed to parse config file: {}", e))
    }
}
