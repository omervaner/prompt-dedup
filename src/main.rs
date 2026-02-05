#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use eframe::egui::{self, Color32, Visuals, Stroke};
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

mod db;
mod export;
mod similarity;

use db::{Database, Prompt};
use similarity::SimilarPair;

// Catppuccin Macchiato colors
mod colors {
    use super::Color32;
    pub const BASE: Color32 = Color32::from_rgb(36, 39, 58);
    pub const MANTLE: Color32 = Color32::from_rgb(30, 32, 48);
    pub const CRUST: Color32 = Color32::from_rgb(24, 25, 38);
    pub const SURFACE0: Color32 = Color32::from_rgb(54, 58, 79);
    pub const SURFACE1: Color32 = Color32::from_rgb(73, 77, 100);
    pub const TEXT: Color32 = Color32::from_rgb(202, 211, 245);
    pub const SUBTEXT: Color32 = Color32::from_rgb(165, 173, 206);
    pub const GREEN: Color32 = Color32::from_rgb(166, 218, 149);
    pub const BLUE: Color32 = Color32::from_rgb(138, 173, 244);
    pub const YELLOW: Color32 = Color32::from_rgb(238, 212, 159);
    pub const RED: Color32 = Color32::from_rgb(237, 135, 150);
    pub const PEACH: Color32 = Color32::from_rgb(245, 169, 127);
}

fn setup_catppuccin_theme(ctx: &egui::Context) {
    let mut visuals = Visuals::dark();

    visuals.panel_fill = colors::CRUST;
    visuals.window_fill = colors::BASE;
    visuals.extreme_bg_color = colors::CRUST;
    visuals.faint_bg_color = colors::SURFACE0;

    visuals.widgets.noninteractive.bg_fill = colors::SURFACE0;
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, colors::TEXT);

    visuals.widgets.inactive.bg_fill = colors::SURFACE0;
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, colors::TEXT);

    visuals.widgets.hovered.bg_fill = colors::SURFACE1;
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, colors::TEXT);

    visuals.widgets.active.bg_fill = colors::SURFACE1;
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, colors::TEXT);

    visuals.selection.bg_fill = colors::BLUE;
    visuals.selection.stroke = Stroke::new(1.0, colors::TEXT);

    ctx.set_visuals(visuals);
}

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1000.0, 750.0]),
        ..Default::default()
    };

    eframe::run_native(
        "Prompt Deduplicator",
        options,
        Box::new(|cc| {
            setup_catppuccin_theme(&cc.egui_ctx);
            Ok(Box::new(PromptDedupApp::new()))
        }),
    )
}

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Browse,
    Deduplicate,
}

struct PromptDedupApp {
    db: Database,
    prompt_count: i64,
    last_import_result: Option<ImportResult>,
    search_query: String,
    displayed_prompts: Vec<Prompt>,

    // Tab state
    active_tab: Tab,

    // Deduplicate state
    similarity_threshold: f32,
    similar_pairs: Vec<SimilarPair>,
    current_pair_index: usize,
    is_scanning: bool,

    // Find & Replace state
    show_find_replace: bool,
    find_text: String,
    replace_text: String,
    case_sensitive: bool,
    replace_preview: Vec<ReplacePreview>,

    // Status message
    status_message: Option<(String, bool)>, // (message, is_error)
    status_time: Option<Instant>,
}

struct ReplacePreview {
    id: i64,
    original: String,
    replaced: String,
}

struct ImportResult {
    file_name: String,
    added: usize,
    skipped: usize,
}

impl PromptDedupApp {
    fn new() -> Self {
        let db = Database::open("prompts.db").expect("Failed to open database");
        let prompt_count = db.count().unwrap_or(0);
        let displayed_prompts = db.get_all().unwrap_or_default();
        Self {
            db,
            prompt_count,
            last_import_result: None,
            search_query: String::new(),
            displayed_prompts,
            active_tab: Tab::Browse,
            similarity_threshold: 0.80,
            similar_pairs: Vec::new(),
            current_pair_index: 0,
            is_scanning: false,
            show_find_replace: false,
            find_text: String::new(),
            replace_text: String::new(),
            case_sensitive: false,
            replace_preview: Vec::new(),
            status_message: None,
            status_time: None,
        }
    }

    fn set_status(&mut self, message: String, is_error: bool) {
        self.status_message = Some((message, is_error));
        self.status_time = Some(Instant::now());
    }

    fn clear_old_status(&mut self) {
        if let Some(time) = self.status_time {
            if time.elapsed().as_secs() >= 5 {
                self.status_message = None;
                self.status_time = None;
            }
        }
    }

    fn import_file(&mut self, path: PathBuf) {
        let file_name = path.file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string());

        match fs::read_to_string(&path) {
            Ok(contents) => {
                let prompts: Vec<(String, Option<String>)> = contents
                    .lines()
                    .map(|line| line.trim())
                    .filter(|line| !line.is_empty())
                    .map(|line| (line.to_string(), Some(file_name.clone())))
                    .collect();

                let total = prompts.len();
                match self.db.insert_prompts(&prompts) {
                    Ok(added) => {
                        let skipped = total - added;

                        self.last_import_result = Some(ImportResult {
                            file_name: file_name.clone(),
                            added,
                            skipped,
                        });

                        self.prompt_count = self.db.count().unwrap_or(0);
                        self.refresh_displayed_prompts();
                        self.set_status(format!("Imported {} from {} ({} duplicates skipped)", added, file_name, skipped), false);
                    }
                    Err(e) => {
                        self.set_status(format!("Database error: {}", e), true);
                    }
                }
            }
            Err(e) => {
                self.set_status(format!("Failed to read file: {}", e), true);
            }
        }
    }

    fn export_prompts(&mut self) {
        if self.displayed_prompts.is_empty() {
            self.set_status("Nothing to export".to_string(), true);
            return;
        }

        if let Some(path) = rfd::FileDialog::new()
            .add_filter("Text files", &["txt"])
            .set_file_name("prompts.txt")
            .save_file()
        {
            match export::export_to_txt(&self.displayed_prompts, &path) {
                Ok(count) => {
                    let filter_info = if !self.search_query.is_empty() {
                        " (filtered)"
                    } else {
                        ""
                    };
                    self.set_status(format!("Exported {} prompts{}", count, filter_info), false);
                }
                Err(e) => {
                    self.set_status(format!("Failed to export: {}", e), true);
                }
            }
        }
    }

    fn refresh_displayed_prompts(&mut self) {
        self.displayed_prompts = if self.search_query.is_empty() {
            self.db.get_all().unwrap_or_default()
        } else {
            self.db.search(&self.search_query).unwrap_or_default()
        };
    }

    fn refresh_counts(&mut self) {
        self.prompt_count = self.db.count().unwrap_or(0);
        self.refresh_displayed_prompts();
    }

    fn scan_for_duplicates(&mut self) {
        let all_prompts = self.db.get_all().unwrap_or_default();
        let prompts_for_scan: Vec<(i64, String)> = all_prompts
            .iter()
            .map(|p| (p.id, p.text.clone()))
            .collect();

        self.similar_pairs = similarity::find_similar_pairs(&prompts_for_scan, self.similarity_threshold);
        self.current_pair_index = 0;
        self.is_scanning = false;
    }

    fn delete_prompt(&mut self, id: i64) {
        let _ = self.db.delete_prompt(id);
        self.refresh_counts();

        // Remove pairs containing this ID
        self.similar_pairs.retain(|p| p.id_a != id && p.id_b != id);

        // Adjust index if needed
        if self.current_pair_index >= self.similar_pairs.len() && !self.similar_pairs.is_empty() {
            self.current_pair_index = self.similar_pairs.len() - 1;
        }
    }

    fn remove_all_duplicates(&mut self) {
        // Collect IDs to delete (keep first, delete second from each pair)
        let ids_to_delete: Vec<i64> = self.similar_pairs.iter()
            .map(|p| p.id_b)
            .collect();

        let count = ids_to_delete.len();
        for id in ids_to_delete {
            let _ = self.db.delete_prompt(id);
        }

        self.similar_pairs.clear();
        self.current_pair_index = 0;
        self.refresh_counts();
        self.set_status(format!("Removed {} duplicate prompts", count), false);
    }

    fn render_highlighted_text(&self, ui: &mut egui::Ui, text: &str) {
        if self.search_query.is_empty() {
            ui.label(text);
            return;
        }

        let query_lower = self.search_query.to_lowercase();
        let text_lower = text.to_lowercase();

        let mut job = egui::text::LayoutJob::default();
        let mut last_end = 0;

        for (start, _) in text_lower.match_indices(&query_lower) {
            if start > last_end {
                job.append(
                    &text[last_end..start],
                    0.0,
                    egui::TextFormat::simple(egui::FontId::default(), colors::TEXT),
                );
            }

            let end = start + self.search_query.len();
            job.append(
                &text[start..end],
                0.0,
                egui::TextFormat {
                    color: colors::CRUST,
                    background: colors::YELLOW,
                    ..egui::TextFormat::simple(egui::FontId::default(), colors::CRUST)
                },
            );

            last_end = end;
        }

        if last_end < text.len() {
            job.append(
                &text[last_end..],
                0.0,
                egui::TextFormat::simple(egui::FontId::default(), colors::TEXT),
            );
        }

        ui.label(job);
    }

    fn render_browse_tab(&mut self, ui: &mut egui::Ui) {
        // Toolbar card
        egui::Frame::new()
            .fill(colors::SURFACE0)
            .inner_margin(16.0)
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // First row: Import, Export, and actions
                ui.horizontal(|ui| {
                    let import_btn = egui::Button::new(
                        egui::RichText::new("Import File").color(colors::CRUST)
                    ).fill(colors::GREEN);

                    if ui.add(import_btn).clicked() {
                        if let Some(path) = rfd::FileDialog::new()
                            .add_filter("Text files", &["txt"])
                            .pick_file()
                        {
                            self.import_file(path);
                        }
                    }

                    ui.add_space(8.0);

                    let export_btn = egui::Button::new(
                        egui::RichText::new("Export").color(colors::CRUST)
                    ).fill(colors::BLUE);

                    if ui.add(export_btn).clicked() {
                        self.export_prompts();
                    }

                    ui.add_space(16.0);

                    if let Some(result) = &self.last_import_result {
                        ui.label(
                            egui::RichText::new(format!(
                                "Last: {} (+{}, -{} dupes)",
                                result.file_name, result.added, result.skipped
                            ))
                            .color(colors::SUBTEXT)
                        );
                    }
                });

                ui.add_space(12.0);

                // Second row: Search
                ui.horizontal(|ui| {
                    ui.label("Search:");
                    let response = ui.add(
                        egui::TextEdit::singleline(&mut self.search_query)
                            .hint_text("Type to filter prompts...")
                            .desired_width(400.0)
                    );
                    if response.changed() {
                        self.refresh_displayed_prompts();
                    }

                    if ui.button("Clear").clicked() {
                        self.search_query.clear();
                        self.refresh_displayed_prompts();
                    }

                    ui.add_space(16.0);
                    ui.label(format!("Showing: {}", self.displayed_prompts.len()));
                });
            });

        ui.add_space(16.0);

        // Prompts table card
        egui::Frame::new()
            .fill(colors::BASE)
            .inner_margin(0.0)
            .corner_radius(8.0)
            .stroke(Stroke::new(1.0, colors::SURFACE0))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                // Table header
                egui::Frame::new()
                    .fill(colors::SURFACE0)
                    .inner_margin(egui::Margin::symmetric(12, 8))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label(egui::RichText::new("Prompts").strong());
                    });

                // Table rows
                let prompts_clone: Vec<String> = self.displayed_prompts.iter()
                    .map(|p| p.text.clone())
                    .collect();

                egui::ScrollArea::vertical()
                    .max_height(ui.available_height() - 10.0)
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());

                        for (i, text) in prompts_clone.iter().enumerate() {
                            let bg_color = if i % 2 == 0 {
                                colors::BASE
                            } else {
                                colors::MANTLE
                            };

                            egui::Frame::new()
                                .fill(bg_color)
                                .inner_margin(egui::Margin::symmetric(12, 8))
                                .show(ui, |ui| {
                                    ui.horizontal(|ui| {
                                        ui.with_layout(
                                            egui::Layout::left_to_right(egui::Align::Center)
                                                .with_main_wrap(true),
                                            |ui| {
                                                ui.set_width(ui.available_width() - 60.0);
                                                self.render_highlighted_text(ui, text);
                                            }
                                        );

                                        ui.with_layout(
                                            egui::Layout::right_to_left(egui::Align::Center),
                                            |ui| {
                                                if ui.small_button("Copy").clicked() {
                                                    ui.ctx().copy_text(text.clone());
                                                }
                                            }
                                        );
                                    });
                                });
                        }
                    });
            });
    }

    fn render_deduplicate_tab(&mut self, ui: &mut egui::Ui) {
        // Toolbar card
        egui::Frame::new()
            .fill(colors::SURFACE0)
            .inner_margin(16.0)
            .corner_radius(8.0)
            .show(ui, |ui| {
                ui.set_width(ui.available_width());

                ui.horizontal(|ui| {
                    ui.label("Similarity threshold:");

                    let slider = egui::Slider::new(&mut self.similarity_threshold, 0.5..=0.99)
                        .show_value(false);
                    ui.add(slider);

                    // Read-only percentage display
                    ui.label(egui::RichText::new(format!("{:.0}%", self.similarity_threshold * 100.0))
                        .color(colors::TEXT));

                    ui.add_space(20.0);

                    let scan_button = egui::Button::new(
                        egui::RichText::new("Find Duplicates").color(colors::CRUST)
                    ).fill(colors::PEACH);

                    if ui.add(scan_button).clicked() {
                        self.is_scanning = true;
                        self.scan_for_duplicates();
                    }

                    ui.add_space(10.0);

                    // Remove All button - only show when pairs exist
                    if !self.similar_pairs.is_empty() {
                        let remove_all_btn = egui::Button::new(
                            egui::RichText::new("Remove All").color(colors::CRUST)
                        ).fill(colors::RED);

                        if ui.add(remove_all_btn).clicked() {
                            self.remove_all_duplicates();
                        }

                        ui.add_space(10.0);
                        ui.label(format!("{} pairs found", self.similar_pairs.len()));
                    }
                });
            });

        ui.add_space(16.0);

        // Comparison view
        if self.similar_pairs.is_empty() {
            egui::Frame::new()
                .fill(colors::BASE)
                .inner_margin(40.0)
                .corner_radius(8.0)
                .stroke(Stroke::new(1.0, colors::SURFACE0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());
                    ui.vertical_centered(|ui| {
                        ui.add_space(40.0);
                        ui.label(egui::RichText::new("No duplicate pairs found").size(18.0).color(colors::SUBTEXT));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Click 'Find Duplicates' to scan your prompts").color(colors::SUBTEXT));
                        ui.add_space(10.0);
                        ui.label(egui::RichText::new("Try lowering the threshold to find more matches").color(colors::SUBTEXT));
                        ui.add_space(40.0);
                    });
                });
        } else {
            let pair = self.similar_pairs[self.current_pair_index].clone();
            let total_pairs = self.similar_pairs.len();

            // Main comparison card
            egui::Frame::new()
                .fill(colors::BASE)
                .inner_margin(20.0)
                .corner_radius(8.0)
                .stroke(Stroke::new(1.0, colors::SURFACE0))
                .show(ui, |ui| {
                    ui.set_width(ui.available_width());

                    // Header
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new(format!(
                            "Pair {} of {}",
                            self.current_pair_index + 1,
                            total_pairs
                        )).strong());

                        ui.add_space(20.0);

                        ui.label(egui::RichText::new(format!(
                            "{:.0}% similar",
                            pair.similarity * 100.0
                        )).color(colors::YELLOW));
                    });

                    ui.add_space(16.0);

                    // Side by side comparison - centered
                    let gap = 20.0;
                    let available_width = ui.available_width();
                    // Account for gap and frame inner margins (16px each side = 32px per card)
                    let card_width = (available_width - gap) / 2.0 - 32.0;

                    ui.columns(2, |columns| {
                        // Left prompt
                        columns[0].vertical_centered(|ui| {
                            egui::Frame::new()
                                .fill(colors::MANTLE)
                                .inner_margin(16.0)
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    ui.set_width(card_width);
                                    ui.set_min_height(150.0);

                                    ui.vertical(|ui| {
                                        ui.label(&pair.text_a);

                                        ui.add_space(16.0);

                                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                            ui.horizontal(|ui| {
                                                let delete_btn = egui::Button::new(
                                                    egui::RichText::new("Delete").color(colors::CRUST)
                                                ).fill(colors::RED);

                                                if ui.add(delete_btn).clicked() {
                                                    self.delete_prompt(pair.id_a);
                                                }

                                                if ui.small_button("Copy").clicked() {
                                                    ui.ctx().copy_text(pair.text_a.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                        });

                        // Right prompt
                        columns[1].vertical_centered(|ui| {
                            egui::Frame::new()
                                .fill(colors::MANTLE)
                                .inner_margin(16.0)
                                .corner_radius(6.0)
                                .show(ui, |ui| {
                                    ui.set_width(card_width);
                                    ui.set_min_height(150.0);

                                    ui.vertical(|ui| {
                                        ui.label(&pair.text_b);

                                        ui.add_space(16.0);

                                        ui.with_layout(egui::Layout::bottom_up(egui::Align::LEFT), |ui| {
                                            ui.horizontal(|ui| {
                                                let delete_btn = egui::Button::new(
                                                    egui::RichText::new("Delete").color(colors::CRUST)
                                                ).fill(colors::RED);

                                                if ui.add(delete_btn).clicked() {
                                                    self.delete_prompt(pair.id_b);
                                                }

                                                if ui.small_button("Copy").clicked() {
                                                    ui.ctx().copy_text(pair.text_b.clone());
                                                }
                                            });
                                        });
                                    });
                                });
                        });
                    });

                    ui.add_space(20.0);

                    // Navigation
                    ui.vertical_centered(|ui| {
                        // Skip button
                        let skip_btn = egui::Button::new(
                            egui::RichText::new("Skip (keep both)")
                        );
                        if ui.add(skip_btn).clicked() {
                            if self.current_pair_index < self.similar_pairs.len() - 1 {
                                self.current_pair_index += 1;
                            }
                        }

                        ui.add_space(12.0);

                        // Previous / Next
                        ui.horizontal(|ui| {
                            let can_prev = self.current_pair_index > 0;
                            let can_next = self.current_pair_index < self.similar_pairs.len() - 1;

                            if ui.add_enabled(can_prev, egui::Button::new("◀ Previous")).clicked() {
                                self.current_pair_index -= 1;
                            }

                            ui.add_space(20.0);

                            if ui.add_enabled(can_next, egui::Button::new("Next ▶")).clicked() {
                                self.current_pair_index += 1;
                            }
                        });
                    });
                });
        }
    }

    fn update_replace_preview(&mut self) {
        self.replace_preview.clear();

        if self.find_text.is_empty() {
            return;
        }

        let all_prompts = self.db.get_all().unwrap_or_default();

        for prompt in all_prompts {
            let contains_match = if self.case_sensitive {
                prompt.text.contains(&self.find_text)
            } else {
                prompt.text.to_lowercase().contains(&self.find_text.to_lowercase())
            };

            if contains_match {
                let replaced = if self.case_sensitive {
                    prompt.text.replace(&self.find_text, &self.replace_text)
                } else {
                    // Case-insensitive replace
                    let lower_find = self.find_text.to_lowercase();
                    let text_lower = prompt.text.to_lowercase();
                    let mut result = String::new();
                    let mut last_end = 0;

                    for (start, _) in text_lower.match_indices(&lower_find) {
                        result.push_str(&prompt.text[last_end..start]);
                        result.push_str(&self.replace_text);
                        last_end = start + self.find_text.len();
                    }
                    result.push_str(&prompt.text[last_end..]);
                    result
                };

                self.replace_preview.push(ReplacePreview {
                    id: prompt.id,
                    original: prompt.text,
                    replaced,
                });
            }
        }
    }

    fn apply_replacements(&mut self) {
        let count = self.replace_preview.len();
        for preview in &self.replace_preview {
            let _ = self.db.update_prompt(preview.id, &preview.replaced);
        }

        self.replace_preview.clear();
        self.find_text.clear();
        self.replace_text.clear();
        self.show_find_replace = false;
        self.refresh_counts();
        self.set_status(format!("Replaced text in {} prompts", count), false);
    }

    fn render_find_replace_popup(&mut self, ctx: &egui::Context) {
        let mut open = self.show_find_replace;

        egui::Window::new("Find & Replace")
            .open(&mut open)
            .collapsible(false)
            .resizable(true)
            .default_width(500.0)
            .default_height(400.0)
            .show(ctx, |ui| {
                // Grid for aligned Find/Replace fields
                egui::Grid::new("find_replace_grid")
                    .num_columns(2)
                    .spacing([10.0, 8.0])
                    .show(ui, |ui| {
                        ui.label("Find:");
                        let find_response = ui.add(
                            egui::TextEdit::singleline(&mut self.find_text)
                                .desired_width(350.0)
                        );
                        if find_response.changed() {
                            self.update_replace_preview();
                        }
                        ui.end_row();

                        ui.label("Replace:");
                        let replace_response = ui.add(
                            egui::TextEdit::singleline(&mut self.replace_text)
                                .desired_width(350.0)
                        );
                        if replace_response.changed() {
                            self.update_replace_preview();
                        }
                        ui.end_row();
                    });

                ui.add_space(8.0);

                let checkbox_response = ui.checkbox(&mut self.case_sensitive, "Case sensitive");
                if checkbox_response.changed() {
                    self.update_replace_preview();
                }

                ui.add_space(16.0);

                // Match count - centered
                ui.vertical_centered(|ui| {
                    ui.label(egui::RichText::new(format!(
                        "Matches: {} prompts",
                        self.replace_preview.len()
                    )).color(colors::YELLOW));
                });

                ui.add_space(8.0);

                // Preview list - centered container
                ui.vertical_centered(|ui| {
                    egui::Frame::new()
                        .fill(colors::BASE)
                        .inner_margin(8.0)
                        .corner_radius(6.0)
                        .stroke(egui::Stroke::new(1.0, colors::SURFACE0))
                        .show(ui, |ui| {
                            ui.set_width(ui.available_width());
                            egui::ScrollArea::vertical()
                                .max_height(200.0)
                                .show(ui, |ui| {
                                    if self.replace_preview.is_empty() {
                                        ui.vertical_centered(|ui| {
                                            ui.label(egui::RichText::new("No matches found").color(colors::SUBTEXT));
                                        });
                                    } else {
                                        for preview in &self.replace_preview {
                                            egui::Frame::new()
                                                .fill(colors::MANTLE)
                                                .inner_margin(8.0)
                                                .corner_radius(4.0)
                                                .show(ui, |ui| {
                                                    // Show truncated versions for long prompts
                                                    let orig_display: String = preview.original.chars().take(80).collect();
                                                    let repl_display: String = preview.replaced.chars().take(80).collect();

                                                    let orig_suffix = if preview.original.len() > 80 { "..." } else { "" };
                                                    let repl_suffix = if preview.replaced.len() > 80 { "..." } else { "" };

                                                    ui.label(egui::RichText::new(format!("{}{}", orig_display, orig_suffix))
                                                        .color(colors::RED));
                                                    ui.label(egui::RichText::new(format!("→ {}{}", repl_display, repl_suffix))
                                                        .color(colors::GREEN));
                                                });
                                            ui.add_space(4.0);
                                        }
                                    }
                                });
                        });
                });

                ui.add_space(16.0);

                // Buttons - centered
                ui.vertical_centered(|ui| {
                    ui.horizontal(|ui| {
                        let can_apply = !self.replace_preview.is_empty();

                        let apply_btn = egui::Button::new(
                            egui::RichText::new("Apply All").color(colors::CRUST)
                        ).fill(colors::GREEN);

                        if ui.add_enabled(can_apply, apply_btn).clicked() {
                            self.apply_replacements();
                        }

                        if ui.button("Cancel").clicked() {
                            self.show_find_replace = false;
                        }
                    });
                });
            });

        self.show_find_replace = open;
    }
}

impl eframe::App for PromptDedupApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Clear old status messages
        self.clear_old_status();

        // Keyboard shortcut: Cmd+R (Mac) / Ctrl+R (others)
        ctx.input(|i| {
            let modifier = if cfg!(target_os = "macos") {
                i.modifiers.mac_cmd
            } else {
                i.modifiers.ctrl
            };
            if modifier && i.key_pressed(egui::Key::R) {
                self.show_find_replace = !self.show_find_replace;
                if self.show_find_replace {
                    self.find_text.clear();
                    self.replace_text.clear();
                    self.replace_preview.clear();
                }
            }
            // Escape to close
            if i.key_pressed(egui::Key::Escape) && self.show_find_replace {
                self.show_find_replace = false;
            }
        });

        // Find & Replace popup
        if self.show_find_replace {
            self.render_find_replace_popup(ctx);
        }

        // Status bar at bottom
        if let Some((message, is_error)) = &self.status_message {
            egui::TopBottomPanel::bottom("status_bar")
                .frame(egui::Frame::new()
                    .fill(if *is_error { colors::RED } else { colors::GREEN })
                    .inner_margin(8.0))
                .show(ctx, |ui| {
                    ui.horizontal_centered(|ui| {
                        ui.label(egui::RichText::new(message).color(colors::CRUST).strong());
                    });
                });
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::new().fill(colors::CRUST).inner_margin(20.0))
            .show(ctx, |ui| {
                // Header with tabs
                ui.horizontal(|ui| {
                    ui.heading("Prompt Deduplicator");

                    ui.add_space(30.0);

                    // Tab buttons
                    let browse_selected = self.active_tab == Tab::Browse;
                    let dedup_selected = self.active_tab == Tab::Deduplicate;

                    if ui.add(egui::SelectableLabel::new(browse_selected, "Browse")).clicked() {
                        self.active_tab = Tab::Browse;
                    }

                    if ui.add(egui::SelectableLabel::new(dedup_selected, "Deduplicate")).clicked() {
                        self.active_tab = Tab::Deduplicate;
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        ui.label(format!("Total: {}", self.prompt_count));
                    });
                });

                ui.add_space(16.0);

                // Render active tab
                match self.active_tab {
                    Tab::Browse => self.render_browse_tab(ui),
                    Tab::Deduplicate => self.render_deduplicate_tab(ui),
                }
            });
    }
}
