use anyhow::{anyhow, Result};
use std::path::Path;
use std::process::Command;

pub fn have(tool: &str) -> bool {
    Command::new("sh")
        .args(["-c", &format!("command -v {}", tool)])
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

pub fn count_pages(pdf: &str) -> usize {
    let out = Command::new("pdfinfo").arg(pdf).output();
    if let Ok(o) = out {
        let s = String::from_utf8_lossy(&o.stdout);
        for line in s.lines() {
            if let Some(rest) = line.strip_prefix("Pages:") {
                if let Ok(n) = rest.trim().parse::<usize>() {
                    return n;
                }
            }
        }
    }
    0
}

/// Render every page of `pdf` to `<out_dir>/<prefix>-NNN.png` using pdftoppm.
pub fn export_images(pdf: &str, out_dir: &str, prefix: &str, dpi: u32) -> Result<Vec<String>> {
    if !have("pdftoppm") {
        return Err(anyhow!("pdftoppm not installed (poppler-utils)"));
    }
    std::fs::create_dir_all(out_dir)?;
    let out_prefix = Path::new(out_dir).join(prefix);
    let status = Command::new("pdftoppm")
        .args(["-png", "-r", &dpi.to_string(), pdf])
        .arg(&out_prefix)
        .status()?;
    if !status.success() {
        return Err(anyhow!("pdftoppm failed"));
    }
    let mut files: Vec<String> = std::fs::read_dir(out_dir)?
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| {
            p.file_name()
                .and_then(|n| n.to_str())
                .map(|n| n.starts_with(&format!("{}-", prefix)) && n.ends_with(".png"))
                .unwrap_or(false)
        })
        .map(|p| p.to_string_lossy().to_string())
        .collect();
    files.sort();
    Ok(files)
}

pub fn export_text(pdf: &str, out_txt: &str) -> Result<()> {
    if !have("pdftotext") {
        return Err(anyhow!("pdftotext not installed (poppler-utils)"));
    }
    let status = Command::new("pdftotext").args(["-layout", pdf, out_txt]).status()?;
    if !status.success() {
        return Err(anyhow!("pdftotext failed"));
    }
    Ok(())
}

/// Convert to a document format. Tries pandoc first, then LibreOffice.
/// `ext` is e.g. "docx" or "odt".
pub fn export_doc(pdf: &str, out_dir: &str, ext: &str) -> Result<String> {
    let out_path = Path::new(out_dir).join(
        Path::new(pdf)
            .file_stem()
            .map(|s| format!("{}.{}", s.to_string_lossy(), ext))
            .unwrap_or_else(|| format!("output.{}", ext)),
    );
    let out_str = out_path.to_string_lossy().to_string();

    if have("pandoc") {
        let status = Command::new("pandoc").args([pdf, "-o", &out_str]).status()?;
        if status.success() && out_path.exists() {
            return Ok(out_str);
        }
    }
    if have("libreoffice") {
        let status = Command::new("libreoffice")
            .args(["--headless", "--convert-to", ext, "--outdir", out_dir, pdf])
            .status()?;
        if status.success() {
            // LibreOffice names the output after the input stem.
            let stem = Path::new(pdf).file_stem().unwrap_or_default().to_string_lossy().to_string();
            let produced = Path::new(out_dir).join(format!("{}.{}", stem, ext));
            if produced.exists() {
                return Ok(produced.to_string_lossy().to_string());
            }
        }
    }
    Err(anyhow!("no converter available (need pandoc or libreoffice)"))
}

/// Send a PDF to the default printer via CUPS `lp`.
pub fn print_pdf(pdf: &str, printer: Option<&str>) -> Result<()> {
    if !have("lp") {
        return Err(anyhow!("lp not installed (cups-client)"));
    }
    let mut cmd = Command::new("lp");
    if let Some(p) = printer {
        cmd.args(["-d", p]);
    }
    let out = cmd.arg(pdf).output()?;
    if !out.status.success() {
        return Err(anyhow!("lp failed: {}", String::from_utf8_lossy(&out.stderr).trim()));
    }
    Ok(())
}

pub fn list_printers() -> Vec<String> {
    let out = Command::new("sh")
        .args(["-c", "lpstat -a 2>/dev/null | awk '{print $1}'"])
        .output();
    match out {
        Ok(o) => String::from_utf8_lossy(&o.stdout)
            .lines()
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect(),
        Err(_) => vec![],
    }
}
