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
                    ui.label(RichText::new("Requires pandoc/pdftoppm").small());
                    if ui.button("DOCX (stub)").clicked() { self.status = "Export stub".to_string(); ui.close_menu(); }
                    if ui.button("ODT (stub)").clicked() { self.status = "Export stub".to_string(); ui.close_menu(); }
                    if ui.button("Images (stub)").clicked() { self.status = "Export stub".to_string(); ui.close_menu(); }
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

        if self.show_help {
            Window::new("About StichUp").open(&mut self.show_help).show(ctx, |ui| {
                ui.label("StichUp v0.1.0");
                ui.label("Linux-only");
            });
        }
    }
}
