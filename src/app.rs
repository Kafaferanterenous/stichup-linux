use egui::{RichText, Window, SidePanel, TopBottomPanel, CentralPanel};
use eframe::App;
use crate::stitch::merge_pdfs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
struct PageItem {
    path: String,
    name: String,
    removed: bool,
    thumb: Option<String>,
}

#[derive(Default)]
pub struct StichUpApp {
    pages: Vec<PageItem>,
    output_path: String,
    status: String,
    show_help: bool,
    text_edit_mode: bool,
    current_text: String,
    show_thumbnails: bool,
    show_bookmarks: bool,
    page_view: PageView,
    show_print_preview: bool,
    preview_imgs: Vec<String>,
    preview_page: usize,
    printers: Vec<String>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PageView {
    #[default]
    FullPage,
    FitWidth,
    TwoPages,
}

impl StichUpApp {
    pub fn new(_cc: &eframe::CreationContext) -> Self {
        Self {
            pages: vec![],
            output_path: "merged.pdf".to_string(),
            status: "Ready".to_string(),
            show_help: false,
            text_edit_mode: false,
            current_text: String::new(),
            show_thumbnails: true,
            show_bookmarks: false,
            page_view: PageView::FullPage,
            show_print_preview: false,
            preview_imgs: vec![],
            preview_page: 0,
            printers: vec![],
        }
    }

    fn make_thumb(&mut self, idx: usize) {
        if idx >= self.pages.len() { return; }
        let pdf = self.pages[idx].path.clone();
        let prefix = format!("/tmp/stichup_thumb_{}_{}", std::process::id(), idx);
        let _ = Command::new("pdftoppm").args(["-png", "-f", "1", "-l", "1", "-scale-to", "96", &pdf, &prefix]).output();
        let candidate1 = format!("{}-1.png", prefix);
        if Path::new(&candidate1).exists() {
            self.pages[idx].thumb = Some(candidate1);
        }
    }

    fn active_paths(&self) -> Vec<String> {
        self.pages.iter().filter(|x| !x.removed).map(|x| x.path.clone()).collect()
    }

    /// Ensure the merged output exists and return its path.
    fn ensure_output(&mut self) -> Option<String> {
        let paths = self.active_paths();
        if paths.is_empty() {
            // fall back to a single PDF if only one page group exists but marked
            self.status = "No PDFs to process".to_string();
            return None;
        }
        let out = if self.output_path.ends_with(".pdf") {
            self.output_path.clone()
        } else {
            format!("{}.pdf", self.output_path)
        };
        match merge_pdfs(&paths) {
            Ok(mut doc) => match doc.save(&out) {
                Ok(_) => Some(out),
                Err(e) => {
                    self.status = format!("Save failed: {}", e);
                    None
                }
            },
            Err(e) => {
                self.status = format!("Merge failed: {}", e);
                None
            }
        }
    }

    fn do_export_images(&mut self) {
        let Some(pdf) = self.ensure_output() else { return };
        let Some(dir) = rfd::FileDialog::new().pick_folder() else { return };
        let dir = dir.to_string_lossy().to_string();
        let stem = Path::new(&pdf).file_stem().unwrap_or_default().to_string_lossy().to_string();
        match crate::export::export_images(&pdf, &dir, &stem, 150) {
            Ok(files) => self.status = format!("Exported {} page image(s) to {}", files.len(), dir),
            Err(e) => self.status = format!("Export images failed: {}", e),
        }
    }

    fn do_export_text(&mut self) {
        let Some(pdf) = self.ensure_output() else { return };
        let Some(out) = rfd::FileDialog::new()
            .add_filter("Text", &["txt"])
            .set_file_name("output.txt")
            .save_file()
        else {
            return;
        };
        let out = out.to_string_lossy().to_string();
        match crate::export::export_text(&pdf, &out) {
            Ok(_) => self.status = format!("Exported text → {}", out),
            Err(e) => self.status = format!("Export text failed: {}", e),
        }
    }

    fn do_export_doc(&mut self, ext: &str) {
        let Some(pdf) = self.ensure_output() else { return };
        let Some(dir) = rfd::FileDialog::new().pick_folder() else { return };
        let dir = dir.to_string_lossy().to_string();
        match crate::export::export_doc(&pdf, &dir, ext) {
            Ok(p) => self.status = format!("Exported → {}", p),
            Err(e) => self.status = format!("Export {} failed: {}", ext, e),
        }
    }

    fn do_print(&mut self, printer: Option<&str>) {
        let Some(pdf) = self.ensure_output() else { return };
        match crate::export::print_pdf(&pdf, printer) {
            Ok(_) => self.status = format!(
                "Sent to printer{}",
                printer.map(|p| format!(" '{}'", p)).unwrap_or_default()
            ),
            Err(e) => self.status = format!("Print failed: {}", e),
        }
    }

    fn open_print_preview(&mut self) {
        let Some(pdf) = self.ensure_output() else { return };
        let dir = format!("/tmp/stichup_preview_{}", std::process::id());
        let _ = std::fs::remove_dir_all(&dir);
        match crate::export::export_images(&pdf, &dir, "page", 110) {
            Ok(files) if !files.is_empty() => {
                self.preview_imgs = files;
                self.preview_page = 0;
                self.printers = crate::export::list_printers();
                self.show_print_preview = true;
                self.status = "Print preview ready".to_string();
            }
            Ok(_) => self.status = "Print preview: no pages".to_string(),
            Err(e) => self.status = format!("Preview failed: {}", e),
        }
    }
}
impl App for StichUpApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.input(|i| {
            if i.key_pressed(egui::Key::F8) {
                self.show_thumbnails = !self.show_thumbnails;
            }
            if i.key_pressed(egui::Key::F9) {
                self.page_view = match self.page_view {
                    PageView::FullPage => PageView::FitWidth,
                    PageView::FitWidth => PageView::TwoPages,
                    PageView::TwoPages => PageView::FullPage,
                };
            }
        });

        TopBottomPanel::top("top_panel").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("File", |ui| {
                    if ui.button("Add PDF(s)").clicked() {
                        if let Some(files) = rfd::FileDialog::new().add_filter("PDF", &["pdf"]).pick_files() {
                            let fcount = files.len();
                            let paths: Vec<String> = files.iter().map(|f| f.to_string_lossy().to_string()).collect();
                            for p in &paths {
                                let name = Path::new(p).file_name().unwrap_or_default().to_string_lossy().to_string();
                                self.pages.push(PageItem { path: p.clone(), name, removed: false, thumb: None });
                            }
                            let start = self.pages.len().saturating_sub(fcount);
                            for i in start..self.pages.len() {
                                self.make_thumb(i);
                            }
                            self.status = format!("Added {} file(s)", self.pages.len());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Merge & Save").clicked() {
                        let paths: Vec<String> = self.pages.iter().filter(|x| !x.removed).map(|x| x.path.clone()).collect();
                        if paths.is_empty() {
                            self.status = "No PDFs to merge".to_string();
                        } else {
                            match merge_pdfs(&paths) {
                                Ok(mut doc) => {
                                    let out = if self.output_path.ends_with(".pdf") { self.output_path.clone() } else { format!("{}.pdf", self.output_path) };
                                    match doc.save(&out) {
                                        Ok(_) => self.status = format!("Merged {} PDFs → {}", paths.len(), out),
                                        Err(e) => self.status = format!("Save failed: {}", e),
                                    }
                                }
                                Err(e) => self.status = format!("Merge failed: {}", e),
                            }
                        }
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Clear").clicked() {
                        self.pages.clear();
                        self.status = "Cleared".to_string();
                        ui.close_menu();
                    }
                    ui.separator();
                    if ui.button("Print Preview…").clicked() {
                        self.open_print_preview();
                        ui.close_menu();
                    }
                    if ui.button("Print (default printer)").clicked() {
                        self.do_print(None);
                        ui.close_menu();
                    }
                    ui.menu_button("Print to…", |ui| {
                        let printers = crate::export::list_printers();
                        if printers.is_empty() {
                            ui.label(RichText::new("No printers found").italics());
                        }
                        for p in printers {
                            if ui.button(&p).clicked() {
                                self.do_print(Some(&p));
                                ui.close_menu();
                            }
                        }
                    });
                });
                ui.menu_button("View", |ui| {
                    ui.checkbox(&mut self.show_thumbnails, "Thumbnails (F8)");
                    ui.checkbox(&mut self.show_bookmarks, "Bookmarks");
                    ui.separator();
                    ui.radio_value(&mut self.page_view, PageView::FullPage, "Full Page");
                    ui.radio_value(&mut self.page_view, PageView::FitWidth, "Fit Width (F9)");
                    ui.radio_value(&mut self.page_view, PageView::TwoPages, "Two Pages (F9)");
                });
                ui.menu_button("Edit", |ui| {
                    if ui.button("Toggle Text Edit Mode").clicked() {
                        self.text_edit_mode = !self.text_edit_mode;
                        self.status = if self.text_edit_mode { "Text edit mode enabled".to_string() } else { "Text edit mode disabled".to_string() };
                        ui.close_menu();
                    }
                });
                ui.menu_button("Export", |ui| {
                    let hints = format!(
                        "pdftoppm:{} pdftotext:{} pandoc:{} libreoffice:{}",
                        crate::export::have("pdftoppm"),
                        crate::export::have("pdftotext"),
                        crate::export::have("pandoc"),
                        crate::export::have("libreoffice"),
                    );
                    ui.label(RichText::new(hints).small());
                    ui.separator();
                    if ui.button("Pages as Images (PNG)…").clicked() {
                        self.do_export_images();
                        ui.close_menu();
                    }
                    if ui.button("Text (.txt)…").clicked() {
                        self.do_export_text();
                        ui.close_menu();
                    }
                    if ui.button("Word (.docx)…").clicked() {
                        self.do_export_doc("docx");
                        ui.close_menu();
                    }
                    if ui.button("OpenDocument (.odt)…").clicked() {
                        self.do_export_doc("odt");
                        ui.close_menu();
                    }
                });
                ui.menu_button("Help", |ui| {
                    if ui.button("About").clicked() { self.show_help = true; ui.close_menu(); }
                });
            });
            egui::Frame::none().fill(ui.visuals().extreme_bg_color).show(ui, |ui| {
                ui.horizontal(|ui| {
                    ui.button(RichText::new("New").small());
                    ui.button(RichText::new("Open").small());
                    ui.button(RichText::new("Save").small());
                    ui.separator();
                    ui.button(RichText::new("Cut").small());
                    ui.button(RichText::new("Copy").small());
                    ui.button(RichText::new("Paste").small());
                    ui.separator();
                    ui.label(format!("{:?}", self.page_view));
                });
            });
        });

        if self.show_thumbnails || self.show_bookmarks {
            SidePanel::left("sidebar").resizable(true).default_width(200.0).show(ctx, |ui| {
                ui.heading("Sidebar");
                ui.checkbox(&mut self.show_thumbnails, "Thumbnails (F8)");
                ui.checkbox(&mut self.show_bookmarks, "Bookmarks");
                ui.separator();
                if self.show_thumbnails {
                    ui.label("Pages");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for i in 0..self.pages.len() {
                            if self.pages[i].removed { continue; }
                            let thumb = self.pages[i].thumb.clone();
                            let name = self.pages[i].name.clone();
                            let mut up = false;
                            let mut down = false;
                            let mut remove = false;
                            ui.vertical(|ui| {
                                if let Some(t) = &thumb {
                                    if let Ok(img) = image::open(t) {
                                        let rgba = img.to_rgba8();
                                        let size = [rgba.width() as usize, rgba.height() as usize];
                                        let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_flat_samples().as_slice());
                                        let tex = ui.ctx().load_texture(t, color_image, egui::TextureOptions::LINEAR);
                                        ui.image((tex.id(), egui::vec2(96.0, 128.0)));
                                    }
                                }
                                ui.horizontal(|ui| {
                                    ui.label(format!("{}.", i+1));
                                    if ui.small_button("↑").clicked() && i>0 { up = true; }
                                    if ui.small_button("↓").clicked() && i+1 < self.pages.len() { down = true; }
                                    if ui.small_button("✕").clicked() { remove = true; }
                                });
                                ui.label(&name);
                            });
                            if up { self.pages.swap(i, i-1); }
                            if down { self.pages.swap(i, i+1); }
                            if remove { self.pages[i].removed = true; }
                        }
                    });
                }
                if self.show_bookmarks {
                    ui.label("Bookmarks");
                    ui.label(RichText::new("No bookmarks loaded").italics());
                }
            });
        }

        CentralPanel::default().show(ctx, |ui| {
            ui.heading("StichUp");
            ui.label("PDF stitching + editing (Linux-only)");
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Output:");
                ui.text_edit_singleline(&mut self.output_path);
            });
            ui.label(&self.status);
        });

        if self.show_print_preview {
            let mut open = true;
            Window::new("Print Preview")
                .open(&mut open)
                .default_width(560.0)
                .show(ctx, |ui| {
                    let total = self.preview_imgs.len();
                    ui.horizontal(|ui| {
                        if ui.button("◀ Prev").clicked() && self.preview_page > 0 {
                            self.preview_page -= 1;
                        }
                        if total > 0 {
                            ui.label(format!("Page {} / {}", self.preview_page + 1, total));
                        }
                        if ui.button("Next ▶").clicked() && self.preview_page + 1 < total {
                            self.preview_page += 1;
                        }
                    });
                    ui.separator();
                    if let Some(p) = self.preview_imgs.get(self.preview_page) {
                        if let Ok(img) = image::open(p) {
                            let rgba = img.to_rgba8();
                            let (w, h) = (rgba.width() as usize, rgba.height() as usize);
                            let ci = egui::ColorImage::from_rgba_unmultiplied(
                                [w, h],
                                rgba.as_flat_samples().as_slice(),
                            );
                            let tex = ui.ctx().load_texture(
                                format!("preview_{}", self.preview_page),
                                ci,
                                egui::TextureOptions::LINEAR,
                            );
                            ui.image((tex.id(), egui::vec2(w as f32 * 0.5, h as f32 * 0.5)));
                        }
                    } else {
                        ui.label(RichText::new("No preview pages").italics());
                    }
                    ui.separator();
                    if ui.button("Print").clicked() {
                        self.do_print(None);
                    }
                });
            if !open {
                self.show_print_preview = false;
            }
        }

        if self.show_help {
            Window::new("About StichUp").open(&mut self.show_help).show(ctx, |ui| {
                ui.label("StichUp v0.1.0");
                ui.label("Linux-only");
            });
        }
    }
}
