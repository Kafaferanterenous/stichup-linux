use anyhow::Result;
use std::process::Command;

pub fn merge_pdfs(paths: &[String]) -> Result<lopdf::Document> {
    if paths.is_empty() {
        return Ok(lopdf::Document::new());
    }
    let out = "/tmp/stichup_merged.pdf";
    let mut cmd = Command::new("gs");
    cmd.args(["-dNOPAUSE", "-dBATCH", "-sDEVICE=pdfwrite", &format!("-sOutputFile={}", out)]);
    for p in paths {
        cmd.arg(p);
    }
    let status = cmd.status()?;
    if !status.success() {
        return Err(anyhow::anyhow!("gs merge failed"));
    }
    Ok(lopdf::Document::load(out)?)
}
