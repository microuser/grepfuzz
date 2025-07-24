use std::path::Path;
use rexif::{parse_file, ExifTag};

pub fn extract_focal_length(path: &Path) -> Option<String> {
    let exif = parse_file(path).ok()?;
    for entry in exif.entries {
        if entry.tag == ExifTag::FocalLength {
            return Some(entry.value_more_readable.to_string());
        }
    }
    None
}
