mod cargo;
mod gem;
mod mdbook;
mod npm;
mod pip;
mod sphinx;

pub use cargo::CargoProcessor;
pub use gem::GemProcessor;
pub use mdbook::MdbookProcessor;
pub use npm::NpmProcessor;
pub use pip::PipProcessor;
pub use sphinx::SphinxProcessor;

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

use crate::graph::Product;

/// Compute the stamp file path for an anchor file within the given output directory.
pub(super) fn stamp_path(output_dir: &Path, anchor: &Path) -> PathBuf {
    let anchor_dir = anchor.parent().unwrap_or(Path::new(""));
    let name = if anchor_dir.as_os_str().is_empty() {
        "root".to_string()
    } else {
        anchor_dir.display().to_string().replace(['/', '\\'], "_")
    };
    output_dir.join(format!("{}.stamp", name))
}

/// Create the stamp file for a product, ensuring its parent directory exists.
pub(super) fn write_stamp(product: &Product, processor_name: &str) -> Result<()> {
    let stamp = product.outputs.first()
        .with_context(|| format!("{} product has no output stamp", processor_name))?;
    if let Some(parent) = stamp.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create {} output directory: {}", processor_name, parent.display()))?;
    }
    fs::write(stamp, "")
        .with_context(|| format!("Failed to write {} stamp file: {}", processor_name, stamp.display()))?;
    Ok(())
}

/// Remove stamp outputs and optional output directory. Returns number of items removed.
pub(super) fn clean_stamp_and_output_dir(product: &Product, processor_name: &str, verbose: bool) -> Result<usize> {
    let mut count = 0;
    for output in &product.outputs {
        match fs::remove_file(output) {
            Ok(()) => {
                count += 1;
                if verbose {
                    println!("Removed {} output: {}", processor_name, output.display());
                }
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => return Err(e.into()),
        }
    }
    if let Some(ref output_dir) = product.output_dir
        && output_dir.exists()
    {
        if verbose {
            println!("Removing {} output directory: {}", processor_name, output_dir.display());
        }
        fs::remove_dir_all(output_dir.as_ref())?;
        count += 1;
    }
    Ok(count)
}
