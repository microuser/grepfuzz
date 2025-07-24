use crate::blur_detector::BlurDetector;
use crate::blur_laplacian::LaplacianVarianceDetector;
use crate::blur_tenengrad::TenengradDetector;
use crate::blur_opencv::OpenCvLaplacianDetector;

pub fn build_detectors(laplacian_threshold: f64, tenengrad_threshold: f64, opencv_laplacian_threshold: f64) -> Vec<Box<dyn BlurDetector>> {
    vec![
        Box::new(LaplacianVarianceDetector { threshold: laplacian_threshold }),
        Box::new(TenengradDetector { threshold: tenengrad_threshold }),
        Box::new(OpenCvLaplacianDetector::new(opencv_laplacian_threshold)),
    ]
}
