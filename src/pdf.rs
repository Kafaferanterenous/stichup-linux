use anyhow::Result;
use lopdf::{Document, Bookmark, Object};

#[derive(Clone, Debug)]
pub struct PdfDoc {
    pub doc: Document,
    pub filename: String,
}

impl PdfDoc {
    pub fn load(path: &str) -> Result<Self> {
        let doc = Document::load(path)?;
        Ok(Self {
            doc,
            filename: path.to_string(),
        })
    }
}
