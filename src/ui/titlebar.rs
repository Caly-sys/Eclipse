use crate::ui::icons::{paint_icon, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{vec2, Color32, Layout, Rect, RichText, Rounding, Stroke, Ui, UiBuilder};

#[allow(clippy::too_many_arguments)]
pub fn render_titlebar(
    ui: &mut Ui,
    theme: &Theme,
    search_query: &mut String,
    on_scan: &mut bool,
    focus_search: &mut bool,
    can_go_back: bool,
    can_go_forward: bool,
    on_back: &mut bool,
    on_forward: &mut bool,
) {
    let titlebar_height = 36.0;
    let titlebar_rect = {
        let mut r = ui.available_rect_before_wrap();
        r.max.y = r.min.y + titlebar_height;
        r
    };

    // Cache search box bounding rect to prevent search clicks from triggering window drag/double-click
    let search_rect_id = ui.id().with("titlebar_search_rect_cache");
    let last_search_rect: Option<egui::Rect> = ui.data(|d| d.get_temp(search_rect_id));
    let pointer_over_search = ui.input(|i| {
        if let (Some(pos), Some(sr)) = (i.pointer.hover_pos(), last_search_rect) {
            sr.expand(3.0).contains(pos)
        } else {
            false
        }
    });

    // 1. Draggable background hit-testing across entire titlebar
    let titlebar_id = ui.id().with("titlebar_drag_bg");
    let titlebar_resp = ui.interact(titlebar_rect, titlebar_id, egui::Sense::click_and_drag());

    if titlebar_resp.double_clicked() && !pointer_over_search {
        let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
    } else if titlebar_resp.drag_started_by(egui::PointerButton::Primary) && !pointer_over_search {
        ui.ctx().send_viewport_cmd(egui::ViewportCommand::StartDrag);
    }

    // 2. Child widgets layout (rendered on top of the titlebar hit-test rect)
    // 3-Column space-between layout:
    // Left: Brand icon + wordmark + history navigation chevrons
    // Right: Quick Scan button + Window controls (Minimize, Maximize, Close)
    // Center: Centered pill search bar (max-width: 380px)

    // Left Column
    let left_w = 210.0_f32;
    let left_rect = Rect::from_min_size(titlebar_rect.min, vec2(left_w, titlebar_height));
    ui.allocate_new_ui(UiBuilder::new().max_rect(left_rect), |ui| {
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing = vec2(8.0, 0.0);

            // App Logo Icon Badge: Clean monochrome / dark glass
            ui.add_space(2.0);
            let (logo_rect, _) = ui.allocate_exact_size(vec2(24.0, 24.0), egui::Sense::hover());
            ui.painter().circle_filled(
                logo_rect.center(),
                11.0,
                Color32::from_rgb(20, 20, 24),
            );
            ui.painter().circle_stroke(
                logo_rect.center(),
                11.0,
                Stroke::new(1.0_f32, theme.border),
            );
            let logo_icon_rect = Rect::from_center_size(logo_rect.center(), vec2(13.0, 13.0));
            paint_icon(ui.painter(), Icon::Sparkle, logo_icon_rect, Color32::WHITE);

            // Bold Uppercase Spaced Wordmark
            ui.label(
                RichText::new("E C L I P S E D")
                    .color(theme.text_primary)
                    .size(13.5)
                    .strong(),
            );

            // History navigation buttons (ChevronLeft / ChevronRight)
            let history_btn = |ui: &mut Ui, icon: Icon, enabled: bool| -> bool {
                let (fill, stroke, icon_color, sense) = if enabled {
                    (
                        theme.card,
                        Stroke::new(1.0_f32, theme.border),
                        theme.text_primary,
                        egui::Sense::click(),
                    )
                } else {
                    (
                        Color32::from_rgb(14, 14, 16),
                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 10)),
                        theme.text_muted,
                        egui::Sense::hover(),
                    )
                };

                let (rect, resp) = ui.allocate_exact_size(vec2(22.0, 22.0), sense);
                if enabled && resp.hovered() {
                    ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
                }

                ui.painter().rect_filled(rect, Rounding::same(6.0), fill);
                ui.painter().rect_stroke(rect, Rounding::same(6.0), stroke);
                let icon_rect = Rect::from_center_size(rect.center(), vec2(11.0, 11.0));
                paint_icon(ui.painter(), icon, icon_rect, icon_color);

                enabled && resp.clicked()
            };

            if history_btn(ui, Icon::ChevronLeft, can_go_back) {
                *on_back = true;
            }
            if history_btn(ui, Icon::ChevronRight, can_go_forward) {
                *on_forward = true;
            }
        });
    });

    // Right Column: Window controls & Quick Scan
    ui.allocate_new_ui(UiBuilder::new().max_rect(titlebar_rect), |ui| {
        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            ui.add_space(2.0);

            // Window controls: Close, Maximize/Restore, Minimize
            // 1. Close Button
            let (close_rect, close_resp) = ui.allocate_exact_size(vec2(32.0, 26.0), egui::Sense::click());
            let (close_bg, close_stroke, close_color) = if close_resp.hovered() {
                (
                    Color32::from_rgb(220, 38, 38),
                    Stroke::NONE,
                    Color32::WHITE,
                )
            } else {
                (
                    Color32::TRANSPARENT,
                    Stroke::NONE,
                    theme.text_secondary,
                )
            };
            ui.painter().rect_filled(close_rect, Rounding::same(6.0), close_bg);
            if close_stroke != Stroke::NONE {
                ui.painter().rect_stroke(close_rect, Rounding::same(6.0), close_stroke);
            }
            let close_icon_rect = Rect::from_center_size(close_rect.center(), vec2(11.0, 11.0));
            paint_icon(ui.painter(), Icon::Close, close_icon_rect, close_color);
            if close_resp.clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Close);
            }

            // 2. Maximize / Restore Button
            let is_maximized = ui.input(|i| i.viewport().maximized.unwrap_or(false));
            let max_icon = if is_maximized { Icon::Restore } else { Icon::Maximize };
            let (max_rect, max_resp) = ui.allocate_exact_size(vec2(32.0, 26.0), egui::Sense::click());
            let (max_bg, max_stroke, max_color) = if max_resp.hovered() {
                (
                    theme.surface_hover,
                    Stroke::new(1.0_f32, theme.border),
                    theme.text_primary,
                )
            } else {
                (
                    Color32::TRANSPARENT,
                    Stroke::NONE,
                    theme.text_secondary,
                )
            };
            ui.painter().rect_filled(max_rect, Rounding::same(6.0), max_bg);
            if max_stroke != Stroke::NONE {
                ui.painter().rect_stroke(max_rect, Rounding::same(6.0), max_stroke);
            }
            let max_icon_rect = Rect::from_center_size(max_rect.center(), vec2(11.0, 11.0));
            paint_icon(ui.painter(), max_icon, max_icon_rect, max_color);
            if max_resp.clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Maximized(!is_maximized));
            }

            // 3. Minimize Button
            let (min_rect, min_resp) = ui.allocate_exact_size(vec2(32.0, 26.0), egui::Sense::click());
            let (min_bg, min_stroke, min_color) = if min_resp.hovered() {
                (
                    theme.surface_hover,
                    Stroke::new(1.0_f32, theme.border),
                    theme.text_primary,
                )
            } else {
                (
                    Color32::TRANSPARENT,
                    Stroke::NONE,
                    theme.text_secondary,
                )
            };
            ui.painter().rect_filled(min_rect, Rounding::same(6.0), min_bg);
            if min_stroke != Stroke::NONE {
                ui.painter().rect_stroke(min_rect, Rounding::same(6.0), min_stroke);
            }
            let min_icon_rect = Rect::from_center_size(min_rect.center(), vec2(11.0, 11.0));
            paint_icon(ui.painter(), Icon::Minimize, min_icon_rect, min_color);
            if min_resp.clicked() {
                ui.ctx().send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            }

            ui.add_space(6.0);

            // Quick Scan Button
            let (scan_rect, scan_resp) = ui.allocate_exact_size(vec2(68.0, 24.0), egui::Sense::click());
            let scan_bg = if scan_resp.hovered() {
                theme.surface_hover
            } else {
                theme.surface
            };
            let scan_stroke = if scan_resp.hovered() {
                Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 30))
            } else {
                Stroke::new(1.0_f32, theme.border)
            };
            ui.painter().rect_filled(scan_rect, Rounding::same(8.0), scan_bg);
            ui.painter().rect_stroke(scan_rect, Rounding::same(8.0), scan_stroke);

            let scan_icon_rect = Rect::from_center_size(
                egui::pos2(scan_rect.min.x + 14.0, scan_rect.center().y),
                vec2(12.0, 12.0),
            );
            let scan_color = if scan_resp.hovered() { theme.text_primary } else { theme.text_secondary };
            paint_icon(ui.painter(), Icon::Refresh, scan_icon_rect, scan_color);

            ui.painter().text(
                egui::pos2(scan_rect.min.x + 28.0, scan_rect.center().y),
                egui::Align2::LEFT_CENTER,
                "Scan",
                egui::FontId::proportional(11.5),
                scan_color,
            );

            if scan_resp.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }
            if scan_resp.clicked() {
                *on_scan = true;
            }
        });
    });

    // Center Column: Centered Pill Search Bar (max-width: 380px)
    let search_max_w = 380.0_f32;
    let available_w = titlebar_rect.width();
    let search_w = search_max_w.min((available_w - 440.0).max(180.0));
    let search_rect = Rect::from_center_size(
        egui::pos2(titlebar_rect.center().x, titlebar_rect.center().y),
        vec2(search_w, 28.0),
    );
    ui.data_mut(|d| d.insert_temp(search_rect_id, search_rect));

    ui.allocate_new_ui(UiBuilder::new().max_rect(search_rect), |ui| {
        let search_frame = egui::Frame::none()
            .fill(Color32::from_rgb(20, 20, 24))
            .stroke(Stroke::new(1.0_f32, theme.border))
            .rounding(Rounding::same(14.0))
            .inner_margin(egui::Margin::symmetric(10.0, 4.0));

        search_frame.show(ui, |ui| {
            ui.horizontal(|ui| {
                ui.spacing_mut().item_spacing = vec2(6.0, 0.0);
                render_icon(ui, Icon::Search, 13.0, theme.text_muted);

                let edit_w = (search_w - 68.0).max(80.0);
                let search_id = ui.id().with("titlebar_search_input");
                let edit = egui::TextEdit::singleline(search_query)
                    .id(search_id)
                    .hint_text(RichText::new("Search your library...").color(theme.text_muted))
                    .text_color(theme.text_primary)
                    .frame(false)
                    .desired_width(edit_w);
                let edit_resp = ui.add(edit);

                if *focus_search {
                    edit_resp.request_focus();
                    *focus_search = false;
                }

                if !search_query.is_empty() {
                    let (close_rect, close_resp) = ui.allocate_exact_size(vec2(16.0, 16.0), egui::Sense::click());
                    let close_color = if close_resp.hovered() { theme.text_primary } else { theme.text_muted };
                    paint_icon(ui.painter(), Icon::Close, close_rect, close_color);
                    if close_resp.clicked() {
                        search_query.clear();
                    }
                } else {
                    // Pill shortcut badge Ctrl K
                    let badge_frame = egui::Frame::none()
                        .fill(Color32::from_rgb(26, 26, 30))
                        .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 18)))
                        .rounding(Rounding::same(4.0))
                        .inner_margin(egui::Margin::symmetric(5.0, 1.5));
                    badge_frame.show(ui, |ui| {
                        ui.label(RichText::new("Ctrl K").size(9.5).color(theme.text_secondary));
                    });
                }
            });
        });
    });
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_titlebar_renders_without_panic() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            egui::CentralPanel::default().show(ctx, |ui| {
                let theme = Theme::from_name("Eclipse", None);
                let mut search_query = String::new();
                let mut on_scan = false;
                let mut focus_search = false;
                let mut on_back = false;
                let mut on_forward = false;

                render_titlebar(
                    ui,
                    &theme,
                    &mut search_query,
                    &mut on_scan,
                    &mut focus_search,
                    true,
                    false,
                    &mut on_back,
                    &mut on_forward,
                );
            });
        });
    }
}
