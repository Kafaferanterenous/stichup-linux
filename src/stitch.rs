use lopdf::{Document};
use anyhow::Result;

pub fn merge_pdfs(paths: &[String]) -> Result<Document> {
    let mut result = Document::new();
    let mut first = true;
    for path in paths {
        let src = Document::load(path)?;
        if first {
            result = src;
            first = false;
        } else {
            result = lopdf::Document::with_version("1.5");
        }
    }
    result.compress();
    Ok(result)
}
