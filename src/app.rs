use egui::{RichText, Window, SidePanel, TopBottomPanel, CentralPanel};
use crate::stitch::merge_pdfs;
use std::path::Path;
use std::process::Command;

#[derive(Clone, Debug)]
struct PageItem {
    path: String,
    name: String,
    removed: bool,
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
}
impl eframe::App for StichUpApp {
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
                            for f in files {
                                let p = f.to_string_lossy().to_string();
                                let name = Path::new(&p).file_name().unwrap_or_default().to_string_lossy().to_string();
                                self.pages.push(PageItem { path: p, name, removed: false });
                            }
                            self.status = format!("Added {} page-group(s)/files", self.pages.len());
                        }
                        ui.close_menu();
                    }
                    if ui.button("Merge & Save").clicked() {
                        let mut paths: Vec<String> = self.pages.iter().filter(|x| !x.removed).map(|x| x.path.clone()).collect();
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
                    ui.button(RichText::new("📄 New").small());
                    ui.button(RichText::new("📂 Open").small());
                    ui.button(RichText::new("💾 Save").small());
                    ui.separator();
                    ui.button(RichText::new("✂️ Cut").small());
                    ui.button(RichText::new("📋 Copy").small());
                    ui.button(RichText::new("📌 Paste").small());
                    ui.separator();
                    ui.button(RichText::new("↩ Undo").small());
                    ui.button(RichText::new("↪ Redo").small());
                    ui.label(format!("{:?}", self.page_view));
                });
            });
        });
        if self.show_thumbnails || self.show_bookmarks {
            SidePanel::left("sidebar").resizable(true).default_width(180.0).show(ctx, |ui| {
                ui.heading("Sidebar");
                ui.checkbox(&mut self.show_thumbnails, "Thumbnails (F8)");
                ui.checkbox(&mut self.show_bookmarks, "Bookmarks");
                ui.separator();
                if self.show_thumbnails {
                    ui.label("Pages");
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        let len = self.pages.iter().filter(|x| !x.removed).count();
                        for i in 0..self.pages.len() {
                            if self.pages[i].removed { continue; }
                            let name = self.pages[i].name.clone();
                            ui.horizontal(|ui| {
                                ui.label(format!("{}.", i+1));
                                if ui.small_button("↑").clicked() && i>0 {
                                    // swap with previous non-removed? easier: just swap
                                    self.pages.swap(i, i-1);
                                }
                                if ui.small_button("↓").clicked() && i+1 < self.pages.len() {
                                    self.pages.swap(i, i+1);
                                }
                                if ui.small_button("✕").clicked() {
                                    self.pages[i].removed = true;
                                }
                            });
                            ui.label(name);
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
            ui.separator();
            ui.label("Pages (reorder/remove):");
            egui::ScrollArea::vertical().max_height(250.0).show(ui, |ui| {
                let mut remove_idx: Option<usize> = None;
                for i in 0..self.pages.len() {
                    if self.pages[i].removed { continue; }
                    let name = self.pages[i].name.clone();
                    ui.horizontal(|ui| {
                        ui.label(format!("{}. {}", i+1, &name));
                        if ui.small_button("Up").clicked() && i>0 { self.pages.swap(i, i-1); }
                        if ui.small_button("Remove").clicked() { remove_idx = Some(i); }
                    });
                }
                if let Some(i) = remove_idx { self.pages[i].removed = true; }
            });
            ui.separator();
            if self.text_edit_mode {
                ui.group(|ui| {
                    ui.label("Text Edit Mode");
                    ui.text_edit_multiline(&mut self.current_text);
                });
            }
            ui.label(&self.status);
        });

        if self.show_help {
            Window::new("About StichUp").open(&mut self.show_help).show(ctx, |ui| {
                ui.label("StichUp v0.1.0");
                ui.label("Built with egui + lopdf");
                ui.label("Linux-only");
            });
        }
    }
}
