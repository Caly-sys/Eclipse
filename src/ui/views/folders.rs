use crate::audio::PlayerController;
use crate::library::model::Track;
use crate::ui::icons::{icon_button, paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Layout, Rect, RichText, Rounding, ScrollArea, Stroke, Ui};
use std::path::Path;

#[derive(Default)]
pub struct FoldersViewState {
    pub selected_folder: Option<String>,
}

pub fn render_folders(
    ui: &mut Ui,
    folders: &[String],
    tracks: &[Track],
    state: &mut FoldersViewState,
    player: &mut PlayerController,
    theme: &Theme,
    on_add_folder: &mut bool,
) {
    ui.vertical(|ui| {
        ui.horizontal(|ui| {
            render_icon(ui, Icon::Folder, 18.0, theme.accent);
            ui.add_space(4.0);
            ui.label(RichText::new("Music Folders").size(18.0).color(theme.text_primary).strong());

            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                let (btn_rect, btn_resp) = ui.allocate_exact_size(vec2(106.0, 28.0), egui::Sense::click());
                let btn_bg = if btn_resp.hovered() { theme.surface_hover } else { theme.card };
                let btn_stroke = if btn_resp.hovered() { Stroke::new(1.0_f32, theme.border_accent) } else { Stroke::new(1.0_f32, theme.border) };
                ui.painter().rect_filled(btn_rect, Rounding::same(6.0), btn_bg);
                ui.painter().rect_stroke(btn_rect, Rounding::same(6.0), btn_stroke);

                let icon_r = Rect::from_center_size(egui::pos2(btn_rect.min.x + 16.0, btn_rect.center().y), vec2(12.0, 12.0));
                paint_icon(ui.painter(), Icon::Plus, icon_r, theme.accent);

                ui.painter().text(
                    egui::pos2(btn_rect.min.x + 28.0, btn_rect.center().y),
                    egui::Align2::LEFT_CENTER,
                    "Add Folder",
                    egui::FontId::proportional(12.0),
                    theme.accent,
                );

                if btn_resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }
                if btn_resp.clicked() {
                    *on_add_folder = true;
                }
            });
        });

        ui.add_space(8.0);

        ScrollArea::vertical()
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.spacing_mut().item_spacing = vec2(0.0, 10.0);

                for folder in folders {
                    let folder_path = Path::new(folder);
                    let folder_name = folder_path
                        .file_name()
                        .and_then(|n| n.to_str())
                        .unwrap_or(folder);

                    let folder_tracks: Vec<&Track> = tracks
                        .iter()
                        .filter(|t| t.path.starts_with(folder))
                        .collect();

                    let is_expanded = state.selected_folder.as_deref() == Some(folder);

                    let mut frame = if is_expanded {
                        let mut f = theme.glass_frame(10.0);
                        f.stroke = Stroke::new(1.0_f32, theme.border_accent);
                        f
                    } else {
                        theme.card_frame(10.0)
                    };
                    frame.inner_margin = egui::Margin::same(12.0);

                    frame.show(ui, |ui| {
                        ui.horizontal(|ui| {
                            render_icon(ui, Icon::Folder, 20.0, theme.accent);
                            ui.add_space(6.0);
                            ui.vertical(|ui| {
                                ui.label(RichText::new(folder_name).size(14.0).color(theme.text_primary).strong());
                                ui.label(RichText::new(folder).size(11.0).color(theme.text_muted));
                            });

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                let arrow_icon = if is_expanded { Icon::ChevronDown } else { Icon::ChevronRight };
                                if icon_button(ui, arrow_icon, 14.0, theme.text_secondary).clicked() {
                                    if is_expanded {
                                        state.selected_folder = None;
                                    } else {
                                        state.selected_folder = Some(folder.clone());
                                    }
                                }

                                ui.label(RichText::new(format!("{} Tracks", folder_tracks.len())).size(12.0).color(theme.text_muted));
                            });
                        });

                        // Show tracks if expanded
                        if is_expanded {
                            ui.add_space(10.0);
                            ui.separator();
                            ui.add_space(8.0);

                            for (i, t) in folder_tracks.iter().enumerate() {
                                ui.horizontal(|ui| {
                                    ui.label(RichText::new(format!("{}", i + 1)).size(11.0).color(theme.text_muted));
                                    ui.add_space(6.0);
                                    let btn = ui.add(
                                        egui::Button::new(
                                            RichText::new(t.display_title()).size(12.5).color(theme.text_primary),
                                        )
                                        .fill(egui::Color32::TRANSPARENT)
                                        .stroke(Stroke::NONE),
                                    );

                                    if btn.clicked() {
                                        let list: Vec<Track> = folder_tracks.iter().map(|&tr| tr.clone()).collect();
                                        player.load_queue(list, i);
                                    }

                                    ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                        ui.label(RichText::new(t.formatted_duration()).size(11.5).color(theme.text_muted));
                                    });
                                });
                            }
                        }
                    });
                }
            });
    });
}
