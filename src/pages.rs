use anyhow::{anyhow, Result};
use lopdf::Document;
use std::process::Command;

/// Compute maximal contiguous ascending runs for a page order, e.g.
/// `[3,1,2,5] -> [[3],[1,2],[5]]`. Used to batch gs extractions.
fn plan_runs(order: &[usize]) -> Vec<Vec<usize>> {
    let mut runs: Vec<Vec<usize>> = Vec::new();
    for &p in order {
        match runs.last_mut() {
            Some(run) if run.last().map(|&l| l + 1 == p).unwrap_or(false) => run.push(p),
            _ => runs.push(vec![p]),
        }
    }
    runs
}

/// Produce `out` containing exactly the pages of `pdf` in the given 1-based
/// `order` (reorder, delete and duplicate all supported) using Ghostscript.
pub fn reorder_pages(pdf: &str, order: &[usize], out: &str) -> Result<()> {
    if order.is_empty() {
        return Err(anyhow!("no pages selected"));
    }
    let runs = plan_runs(order);
    let tmpdir = std::env::temp_dir().join(format!("stichup_pages_{}", std::process::id()));
    std::fs::create_dir_all(&tmpdir)?;

    let mut parts: Vec<String> = Vec::new();
    for (i, run) in runs.iter().enumerate() {
        let first = *run.first().unwrap();
        let last = *run.last().unwrap();
        let part = tmpdir.join(format!("part{}.pdf", i));
        let status = Command::new("gs")
            .args(["-q", "-dNOPAUSE", "-dBATCH", "-sDEVICE=pdfwrite"])
            .arg(format!("-dFirstPage={}", first))
            .arg(format!("-dLastPage={}", last))
            .arg(format!("-sOutputFile={}", part.display()))
            .arg(pdf)
            .status()?;
        if !status.success() {
            return Err(anyhow!("gs page extract failed ({}..{})", first, last));
        }
        parts.push(part.to_string_lossy().to_string());
    }

    let mut cmd = Command::new("gs");
    cmd.args(["-q", "-dNOPAUSE", "-dBATCH", "-sDEVICE=pdfwrite"])
        .arg(format!("-sOutputFile={}", out));
    for p in &parts {
        cmd.arg(p);
    }
    let status = cmd.status()?;
    let _ = std::fs::remove_dir_all(&tmpdir);
    if !status.success() {
        return Err(anyhow!("gs merge of page parts failed"));
    }
    Ok(())
}

/// Rotate the given 1-based pages by `degrees` (multiple of 90), writing `out`.
pub fn rotate_pages(pdf: &str, pages: &[usize], degrees: i64, out: &str) -> Result<()> {
    let mut doc = Document::load(pdf)?;
    let ids: Vec<lopdf::ObjectId> = doc.get_pages().values().cloned().collect();
    for &p in pages {
        if let Some(id) = ids.get(p.saturating_sub(1)) {
            let obj = doc.get_object_mut(*id)?;
            let dict = obj.as_dict_mut()?;
            let cur = dict.get(b"Rotate").ok().and_then(|o| o.as_i64().ok()).unwrap_or(0);
            let next = ((cur + degrees) % 360 + 360) % 360;
            dict.set("Rotate", next);
        }
    }
    doc.save(out)?;
    Ok(())
}

pub fn page_count(pdf: &str) -> usize {
    match Document::load(pdf) {
        Ok(d) => d.get_pages().len(),
        Err(_) => 0,
    }
}
