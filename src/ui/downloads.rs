use crate::audio::PlayerController;
use crate::streams::controller::StreamController;
use crate::streams::{DownloadFormat, DownloadJob, DownloadStatus};
use crate::ui::components::{dark_pill_button, pill_badge};
use crate::ui::icons::{icon_button, render_icon, Icon};
use crate::ui::theme::Theme;
use egui::{
    pos2, vec2, Color32, Layout, RichText, Rounding, ScrollArea, Stroke, Ui,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DownloadsTab {
    Queue,
    History,
}

/// Render the floating Download Queue / History drawer popover anchored to the top-right.
pub fn render_downloads_popover(
    ctx: &egui::Context,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
    show_downloads: &mut bool,
) {
    if !*show_downloads {
        return;
    }

    let screen_rect = ctx.screen_rect();
    let popover_w = 380.0_f32;
    let popover_h = (screen_rect.height() - 90.0).clamp(240.0, 540.0);
    let popover_pos = pos2(
        (screen_rect.max.x - popover_w - 14.0).max(10.0),
        42.0,
    );

    egui::Area::new(egui::Id::new("downloads_floating_popover"))
        .order(egui::Order::Foreground)
        .fixed_pos(popover_pos)
        .show(ctx, |ui| {
            let popover_frame = egui::Frame::none()
                .fill(Color32::from_rgb(18, 18, 22))
                .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(124, 77, 255, 80)))
                .rounding(Rounding::same(12.0))
                .inner_margin(egui::Margin::symmetric(14.0, 12.0));

            ui.allocate_ui_with_layout(
                vec2(popover_w, popover_h),
                Layout::top_down(egui::Align::Min),
                |ui| {
                    popover_frame.show(ui, |ui| {
                        ui.set_min_width(popover_w - 28.0);
                        ui.set_max_width(popover_w - 28.0);

                        // Header Bar: Title, Active Badge, Close Button
                        let active_count = stream_controller.active_downloads_count();
                        let total_jobs = stream_controller.download_jobs.len();
                        let history_count = total_jobs.saturating_sub(active_count);

                        ui.horizontal(|ui| {
                            render_icon(ui, Icon::Download, 16.0, theme.accent);
                            ui.add_space(4.0);
                            ui.label(
                                RichText::new("Downloads")
                                    .size(15.0)
                                    .color(theme.text_primary)
                                    .strong(),
                            );

                            if active_count > 0 {
                                pill_badge(
                                    ui,
                                    &format!("{} ACTIVE", active_count),
                                    Color32::from_rgba_premultiplied(124, 77, 255, 35),
                                    theme.accent,
                                );
                            }

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if icon_button(ui, Icon::Close, 12.0, theme.text_muted).clicked() {
                                    *show_downloads = false;
                                }
                            });
                        });

                        ui.add_space(8.0);

                        // Audio Format Segmented Selector
                        ui.horizontal(|ui| {
                            ui.label(
                                RichText::new("FORMAT")
                                    .size(10.5)
                                    .color(theme.text_muted)
                                    .strong(),
                            );
                            ui.add_space(2.0);

                            for fmt in DownloadFormat::ALL {
                                let is_selected = stream_controller.selected_download_format == fmt;
                                let (bg, stroke, text_color) = if is_selected {
                                    (
                                        Color32::from_rgb(124, 77, 255),
                                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(179, 136, 255, 180)),
                                        Color32::WHITE,
                                    )
                                } else {
                                    (
                                        Color32::from_rgba_premultiplied(255, 255, 255, 10),
                                        Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 20)),
                                        theme.text_secondary,
                                    )
                                };

                                let btn = egui::Button::new(
                                    RichText::new(fmt.label())
                                        .size(11.0)
                                        .color(text_color)
                                        .strong(),
                                )
                                .fill(bg)
                                .stroke(stroke)
                                .rounding(Rounding::same(6.0))
                                .min_size(vec2(44.0, 22.0));

                                if ui.add(btn).clicked() {
                                    stream_controller.set_download_format(fmt);
                                }
                            }
                        });

                        ui.add_space(8.0);

                        // Segmented Tab Selector
                        let tab_id = ui.id().with("downloads_active_tab");
                        let mut current_tab: DownloadsTab = ui
                            .data(|d| d.get_temp(tab_id))
                            .unwrap_or(DownloadsTab::Queue);

                        ui.horizontal(|ui| {
                            let queue_label = format!("Queue ({})", active_count);
                            let history_label = format!("History ({})", history_count);

                            if tab_button(ui, &queue_label, current_tab == DownloadsTab::Queue, theme).clicked() {
                                current_tab = DownloadsTab::Queue;
                                ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                            }
                            if tab_button(ui, &history_label, current_tab == DownloadsTab::History, theme).clicked() {
                                current_tab = DownloadsTab::History;
                                ui.data_mut(|d| d.insert_temp(tab_id, current_tab));
                            }
                        });

                        ui.add_space(6.0);
                        ui.separator();
                        ui.add_space(6.0);

                        // Tab Content
                        match current_tab {
                            DownloadsTab::Queue => {
                                render_queue_tab(ui, stream_controller, theme);
                            }
                            DownloadsTab::History => {
                                render_history_tab(ui, stream_controller, player, theme);
                            }
                        }
                    });
                },
            );
        });
}

fn tab_button(ui: &mut Ui, label: &str, is_selected: bool, theme: &Theme) -> egui::Response {
    let (bg, stroke, text_color) = if is_selected {
        (
            Color32::from_rgba_premultiplied(124, 77, 255, 35),
            Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(124, 77, 255, 120)),
            Color32::WHITE,
        )
    } else {
        (
            theme.surface,
            Stroke::new(1.0_f32, theme.border),
            theme.text_secondary,
        )
    };

    let btn = egui::Button::new(RichText::new(label).size(11.5).color(text_color).strong())
        .fill(bg)
        .stroke(stroke)
        .rounding(Rounding::same(8.0))
        .min_size(vec2(80.0, 24.0));

    ui.add(btn)
}

fn render_queue_tab(ui: &mut Ui, stream_controller: &mut StreamController, theme: &Theme) {
    let active_jobs: Vec<DownloadJob> = stream_controller
        .download_jobs
        .iter()
        .filter(|j| j.is_active())
        .cloned()
        .collect();

    if active_jobs.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(36.0);
            render_icon(ui, Icon::Download, 32.0, theme.text_muted);
            ui.add_space(8.0);
            ui.label(
                RichText::new("No active downloads")
                    .size(13.5)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Tracks you download will be resolved and saved in the background.")
                    .size(11.5)
                    .color(theme.text_muted),
            );
            ui.add_space(36.0);
        });
        return;
    }

    // Top action row: Active items count & Cancel All button
    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} active", active_jobs.len()))
                .size(11.5)
                .color(theme.text_muted),
        );

        if stream_controller.active_downloads_count() > 0 {
            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                if dark_pill_button(ui, "Cancel All", Some(Icon::Close)).clicked() {
                    stream_controller.cancel_all_downloads();
                }
            });
        }
    });

    ui.add_space(4.0);

    ScrollArea::vertical()
        .id_salt("downloads_queue_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

            for job in active_jobs {
                let card_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 28))
                    .stroke(Stroke::new(1.0_f32, Color32::from_rgba_premultiplied(255, 255, 255, 12)))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0));

                card_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        render_icon(ui, Icon::Music, 16.0, theme.accent);
                        ui.add_space(4.0);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(job.track.display_title())
                                        .size(12.5)
                                        .color(Color32::WHITE)
                                        .strong(),
                                );
                                pill_badge(
                                    ui,
                                    job.format.label(),
                                    Color32::from_rgba_premultiplied(124, 77, 255, 30),
                                    theme.accent,
                                );
                            });
                            ui.label(
                                RichText::new(job.track.display_artist())
                                    .size(11.0)
                                    .color(theme.text_muted),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if icon_button(ui, Icon::Close, 11.0, theme.text_muted).clicked() {
                                stream_controller.cancel_download(&job.id);
                            }
                        });
                    });

                    ui.add_space(4.0);

                    // Progress bar and status indicator
                    match &job.status {
                        DownloadStatus::Queued => {
                            pill_badge(
                                ui,
                                "QUEUED",
                                Color32::from_rgba_premultiplied(255, 255, 255, 12),
                                theme.text_secondary,
                            );
                        }
                        DownloadStatus::Resolving => {
                            pill_badge(
                                ui,
                                "RESOLVING AUDIO",
                                Color32::from_rgba_premultiplied(124, 77, 255, 30),
                                theme.accent,
                            );
                        }
                        DownloadStatus::Downloading(pct) => {
                            let p = pct.clamp(0.0, 1.0);
                            ui.add(
                                egui::ProgressBar::new(p)
                                    .show_percentage()
                                    .animate(true)
                                    .fill(theme.accent),
                            );
                        }
                        DownloadStatus::Tagging => {
                            pill_badge(
                                ui,
                                "TAGGING ID3 METADATA",
                                Color32::from_rgba_premultiplied(124, 77, 255, 30),
                                theme.accent,
                            );
                        }
                        _ => {}
                    }
                });
            }
        });
}

fn render_history_tab(
    ui: &mut Ui,
    stream_controller: &mut StreamController,
    player: &mut PlayerController,
    theme: &Theme,
) {
    let history_jobs: Vec<DownloadJob> = stream_controller
        .download_jobs
        .iter()
        .filter(|j| !j.is_active())
        .cloned()
        .collect();

    // Top action row: Clear Completed and Retry Failed buttons
    let has_completed = history_jobs.iter().any(|j| j.is_completed());
    let has_failed = history_jobs.iter().any(|j| matches!(j.status, DownloadStatus::Failed(_)));

    ui.horizontal(|ui| {
        ui.label(
            RichText::new(format!("{} items", history_jobs.len()))
                .size(11.5)
                .color(theme.text_muted),
        );

        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
            if has_completed && dark_pill_button(ui, "Clear Completed", Some(Icon::Trash)).clicked() {
                stream_controller.clear_completed_downloads();
            }
            if has_failed {
                ui.add_space(4.0);
                if dark_pill_button(ui, "Retry Failed", Some(Icon::Refresh)).clicked() {
                    stream_controller.retry_all_failed_downloads();
                }
            }
        });
    });

    ui.add_space(4.0);

    if history_jobs.is_empty() {
        ui.vertical_centered(|ui| {
            ui.add_space(36.0);
            render_icon(ui, Icon::Hourglass, 30.0, theme.text_muted);
            ui.add_space(8.0);
            ui.label(
                RichText::new("No download history")
                    .size(13.5)
                    .color(theme.text_primary)
                    .strong(),
            );
            ui.add_space(4.0);
            ui.label(
                RichText::new("Completed and failed downloads will appear here.")
                    .size(11.5)
                    .color(theme.text_muted),
            );
            ui.add_space(36.0);
        });
        return;
    }

    ScrollArea::vertical()
        .id_salt("downloads_history_scroll")
        .auto_shrink([false, false])
        .show(ui, |ui| {
            ui.spacing_mut().item_spacing = vec2(0.0, 8.0);

            for job in history_jobs {
                let is_completed = job.is_completed();
                let border_color = if is_completed {
                    Color32::from_rgba_premultiplied(34, 197, 94, 40)
                } else {
                    Color32::from_rgba_premultiplied(239, 68, 68, 50)
                };

                let card_frame = egui::Frame::none()
                    .fill(Color32::from_rgb(24, 24, 28))
                    .stroke(Stroke::new(1.0_f32, border_color))
                    .rounding(Rounding::same(8.0))
                    .inner_margin(egui::Margin::symmetric(10.0, 8.0));

                card_frame.show(ui, |ui| {
                    ui.horizontal(|ui| {
                        let icon = if is_completed { Icon::Music } else { Icon::Info };
                        let icon_col = if is_completed {
                            Color32::from_rgb(34, 197, 94)
                        } else {
                            Color32::from_rgb(239, 68, 68)
                        };
                        render_icon(ui, icon, 16.0, icon_col);
                        ui.add_space(4.0);

                        ui.vertical(|ui| {
                            ui.spacing_mut().item_spacing = vec2(0.0, 2.0);
                            ui.horizontal(|ui| {
                                ui.label(
                                    RichText::new(job.track.display_title())
                                        .size(12.5)
                                        .color(Color32::WHITE)
                                        .strong(),
                                );
                                pill_badge(
                                    ui,
                                    job.format.label(),
                                    Color32::from_rgba_premultiplied(124, 77, 255, 30),
                                    theme.accent,
                                );
                            });
                            ui.label(
                                RichText::new(job.track.display_artist())
                                    .size(11.0)
                                    .color(theme.text_muted),
                            );
                        });

                        ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                            if is_completed {
                                pill_badge(
                                    ui,
                                    "COMPLETED",
                                    Color32::from_rgba_unmultiplied(34, 197, 94, 25),
                                    Color32::from_rgb(34, 197, 94),
                                );
                            } else {
                                pill_badge(
                                    ui,
                                    "FAILED",
                                    Color32::from_rgba_unmultiplied(239, 68, 68, 25),
                                    Color32::from_rgb(239, 68, 68),
                                );
                            }
                        });
                    });

                    ui.add_space(6.0);

                    // Actions row
                    if let DownloadStatus::Completed(ref path) = job.status {
                        ui.horizontal(|ui| {
                            let path_clone = path.clone();
                            if dark_pill_button(ui, "Play", Some(Icon::Play)).clicked() {
                                let mut local_trk = job.track.to_track();
                                local_trk.path = path_clone.to_string_lossy().to_string();
                                player.load_queue(vec![local_trk], 0);
                                player.play();
                            }

                            ui.add_space(4.0);

                            if dark_pill_button(ui, "Open Folder", Some(Icon::Folder)).clicked() {
                                #[cfg(target_os = "windows")]
                                {
                                    if path_clone.exists() {
                                        let _ = std::process::Command::new("explorer")
                                            .arg(format!("/select,\"{}\"", path_clone.display()))
                                            .spawn();
                                    } else if let Some(parent) = path_clone.parent() {
                                        let _ = std::process::Command::new("explorer")
                                            .arg(parent)
                                            .spawn();
                                    }
                                }
                            }
                        });
                    } else if let DownloadStatus::Failed(ref err) = job.status {
                        ui.horizontal(|ui| {
                            let err_msg = if err.trim().is_empty() {
                                job.error.as_deref().unwrap_or("Download failed")
                            } else {
                                err.as_str()
                            };

                            ui.label(
                                RichText::new(err_msg)
                                    .size(11.0)
                                    .color(Color32::from_rgb(248, 113, 113)),
                            );

                            ui.with_layout(Layout::right_to_left(egui::Align::Center), |ui| {
                                if dark_pill_button(ui, "Retry", Some(Icon::Refresh)).clicked() {
                                    stream_controller.retry_download(&job.id);
                                }
                            });
                        });
                    }
                });
            }
        });
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::AppConfig;
    use crate::streams::provider::StreamTrack;

    fn sample_track(id: &str, title: &str) -> StreamTrack {
        StreamTrack::new(
            id,
            title,
            "Test Artist",
            "Test Album",
            180.0,
            None,
            format!("spotify:track:{}", id),
            "spotify",
        )
    }

    #[test]
    fn test_render_downloads_popover_empty() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            let mut controller = StreamController::default();
            let cfg = AppConfig::default();
            let mut player = PlayerController::new(&cfg);
            let theme = Theme::from_name("Eclipse", None);
            let mut show = true;

            render_downloads_popover(ctx, &mut controller, &mut player, &theme, &mut show);
            assert!(show);
        });
    }

    #[test]
    fn test_render_downloads_popover_with_jobs() {
        let ctx = egui::Context::default();
        let _ = ctx.run(Default::default(), |ctx| {
            let mut controller = StreamController::default();
            let cfg = AppConfig::default();
            let mut player = PlayerController::new(&cfg);
            let theme = Theme::from_name("Eclipse", None);

            // Add active jobs
            let mut job1 = DownloadJob::new("job1", sample_track("t1", "Track 1"));
            job1.status = DownloadStatus::Downloading(0.45);
            controller.download_jobs.push(job1);

            let mut job2 = DownloadJob::new("job2", sample_track("t2", "Track 2"));
            job2.status = DownloadStatus::Resolving;
            controller.download_jobs.push(job2);

            // Add completed job
            let mut job3 = DownloadJob::new("job3", sample_track("t3", "Track 3"));
            job3.status = DownloadStatus::Completed(std::path::PathBuf::from("C:/music/track3.mp3"));
            controller.download_jobs.push(job3);

            // Add failed job
            let mut job4 = DownloadJob::new("job4", sample_track("t4", "Track 4"));
            job4.status = DownloadStatus::Failed("Network timeout".to_string());
            job4.error = Some("Network timeout".to_string());
            controller.download_jobs.push(job4);

            assert_eq!(controller.active_downloads_count(), 2);

            let mut show = true;
            render_downloads_popover(ctx, &mut controller, &mut player, &theme, &mut show);
            assert!(show);
        });
    }

    #[test]
    fn test_downloads_tab_switching() {
        assert_eq!(DownloadsTab::Queue, DownloadsTab::Queue);
        assert_ne!(DownloadsTab::Queue, DownloadsTab::History);
    }

    #[test]
    fn test_download_format_switching_and_badges() {
        let mut controller = StreamController::default();
        assert_eq!(controller.selected_download_format, DownloadFormat::Flac);

        controller.set_download_format(DownloadFormat::Mp3);
        assert_eq!(controller.selected_download_format, DownloadFormat::Mp3);

        controller.set_download_format(DownloadFormat::Opus);
        assert_eq!(controller.selected_download_format, DownloadFormat::Opus);

        // Verify format badges on jobs
        let mut job = DownloadJob::new("job_flac", sample_track("tr_1", "Lossless Song"));
        job.format = DownloadFormat::Flac;
        assert_eq!(job.format.label(), "FLAC");

        let mut job_mp3 = DownloadJob::new("job_mp3", sample_track("tr_2", "Compressed Song"));
        job_mp3.format = DownloadFormat::Mp3;
        assert_eq!(job_mp3.format.label(), "MP3");
    }

    #[test]
    fn test_batch_controls_cancel_all_and_retry_failed() {
        let mut controller = StreamController::default();

        // 1. Add active, failed, and completed jobs
        let mut job1 = DownloadJob::new("job1", sample_track("t1", "Track 1"));
        job1.status = DownloadStatus::Downloading(0.5);
        controller.download_jobs.push(job1);

        let mut job2 = DownloadJob::new("job2", sample_track("t2", "Track 2"));
        job2.status = DownloadStatus::Resolving;
        controller.download_jobs.push(job2);

        let mut job3 = DownloadJob::new("job3", sample_track("t3", "Track 3"));
        job3.status = DownloadStatus::Failed("HTTP 500 error".to_string());
        job3.error = Some("HTTP 500 error".to_string());
        controller.download_jobs.push(job3);

        let mut job4 = DownloadJob::new("job4", sample_track("t4", "Track 4"));
        job4.status = DownloadStatus::Failed("Rate limited".to_string());
        job4.error = Some("Rate limited".to_string());
        controller.download_jobs.push(job4);

        assert_eq!(controller.active_downloads_count(), 2);

        // Cancel all active
        controller.cancel_all_downloads();
        assert_eq!(controller.active_downloads_count(), 0);

        // All 4 jobs are now failed/cancelled
        let failed_count = controller
            .download_jobs
            .iter()
            .filter(|j| matches!(j.status, DownloadStatus::Failed(_)))
            .count();
        assert_eq!(failed_count, 4);

        // Retry all failed downloads
        controller.retry_all_failed_downloads();
        for job in &controller.download_jobs {
            assert_eq!(job.status, DownloadStatus::Queued);
            assert!(job.error.is_none());
        }
        assert_eq!(controller.active_downloads_count(), 4);
    }
}
