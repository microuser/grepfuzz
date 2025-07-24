use crate::blur_result::BlurResult;
use std::io::{self, Write};
use ansi_term::Colour::{Green, Red};

pub fn print_results<W: Write>(
    writer: &mut W,
    is_blurry: bool,
    results: &[BlurResult],
    size: u64,
    width: u32,
    height: u32,
    focal: &Option<String>,
    filename: &str,
    verbose: bool,
    ascii: bool,
) -> io::Result<()> {
    if ascii {
        writeln!(writer, "{}\t{}\t{}\t{}\t{}\t{}\t{}", filename, is_blurry, size, width, height, focal.as_deref().unwrap_or("-"),
            results.iter().map(|r| format!("{}:{}:{}", r.name, r.value, r.is_blurry)).collect::<Vec<_>>().join(","))?;
    } else if verbose {
        writeln!(writer, "File: {}", filename)?;
        writeln!(writer, "  Size: {} bytes", size)?;
        writeln!(writer, "  Dimensions: {}x{}", width, height)?;
        writeln!(writer, "  Focal Length: {}", focal.as_deref().unwrap_or("-"))?;
        for r in results {
            let blur_str = if r.is_blurry {
                Red.paint("BLURRY")
            } else {
                Green.paint("SHARP")
            };
            writeln!(writer, "  {}: value = {:.3}, blurry = {} (threshold: {:.3})", r.name, r.value, blur_str, r.threshold)?;
        }
        let overall_str = if is_blurry {
            Red.paint("BLURRY")
        } else {
            Green.paint("SHARP")
        };
        writeln!(writer, "  Overall blurry: {}", overall_str)?;
    } else {
        writeln!(writer, "{}\t{}", filename, is_blurry)?;
    }
    Ok(())
}
