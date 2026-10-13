use anyhow::Result;
use std::process::Command;

pub fn export_doc(path: &str, out: &str) -> Result<()> {
    // Placeholder: use pandoc if available
    let _ = Command::new("pandoc").args([path, "-o", out]).output();
    Ok(())
}

pub fn export_images(path: &str, prefix: &str) -> Result<()> {
    // Placeholder: use pdftoppm if available
    let _ = Command::new("pdftoppm").args(["-png", path, prefix]).output();
    Ok(())
}
