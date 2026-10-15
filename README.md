# StichUp Linux

Goal: simple PDF stitching (stich/up) tool for Linux. Start minimal and build up.

## Features (current)

- Add multiple PDFs and stitch them into one merged PDF (Ghostscript).
- Page list in UI; Page Manager for reorder, delete, rotate (90°/180°/270°).
- Thumbnails using pdftoppm.
- F8 toggle thumbnails/bookmarks; F9 cycle page views (Full Page/Fit Width/Two Pages).
- Export: Images (PNG), Text (pdftotext), DOCX (pandoc/LibreOffice), ODT (pandoc/LibreOffice).
- Print/Print Preview (via CUPS lp).
- GS-based merge/reorder/rotate operations; export helpers detect external tools (gs, pdftoppm, pdftotext, pandoc, libreoffice, lp).
