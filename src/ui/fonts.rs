use std::fs;
use std::path::Path;

/// Configures egui FontDefinitions with system CJK fallback fonts for pristine typography.
/// This eliminates tofu (empty boxes `[]`) for Japanese, Chinese, and Korean characters in song
/// titles (e.g. Aimyon, Yorushika, Eve, Kenshi Yonezu) and synced lyrics.
///
/// Note: Icons are rendered 100% via pure vector paths in `src/ui/icons.rs` without any emoji fonts.
pub fn setup_custom_fonts() -> egui::FontDefinitions {
    let mut fonts = egui::FontDefinitions::default();

    let font_candidates: [(&str, &[&str]); 6] = [
        ("segoe_ui", &[
            r"C:\Windows\Fonts\segoeui.ttf",
        ]),
        ("yu_gothic", &[
            r"C:\Windows\Fonts\YuGothM.ttc",
            r"C:\Windows\Fonts\YuGothR.ttc",
        ]),
        ("ms_gothic", &[
            r"C:\Windows\Fonts\msgothic.ttc",
        ]),
        ("yahei", &[
            r"C:\Windows\Fonts\msyh.ttc",
        ]),
        ("malgun", &[
            r"C:\Windows\Fonts\malgun.ttf",
        ]),
        ("segoe_ui_sym", &[
            r"C:\Windows\Fonts\seguisym.ttf",
        ]),
    ];

    let mut loaded = Vec::new();

    for (name, paths) in font_candidates {
        for path_str in paths {
            let path = Path::new(path_str);
            if path.exists() {
                if let Ok(bytes) = fs::read(path) {
                    fonts.font_data.insert(
                        name.to_string(),
                        egui::FontData::from_owned(bytes),
                    );
                    loaded.push(name.to_string());
                    break;
                }
            }
        }
    }

    // Configure Proportional font priority:
    // Segoe UI -> Yu Gothic -> MS Gothic -> YaHei -> Malgun -> Segoe UI Symbol -> egui defaults
    if let Some(prop) = fonts.families.get_mut(&egui::FontFamily::Proportional) {
        let mut priority_order = Vec::new();
        for key in ["segoe_ui", "yu_gothic", "ms_gothic", "yahei", "malgun", "segoe_ui_sym"] {
            if loaded.iter().any(|f| f == key) {
                priority_order.push(key.to_string());
            }
        }
        let existing = std::mem::take(prop);
        *prop = priority_order;
        for font in existing {
            if !prop.contains(&font) {
                prop.push(font);
            }
        }
    }

    // Also configure Monospace font family for lyrics alignment
    if let Some(mono) = fonts.families.get_mut(&egui::FontFamily::Monospace) {
        for key in ["ms_gothic", "yu_gothic", "yahei", "malgun", "segoe_ui_sym"] {
            if loaded.iter().any(|f| f == key) && !mono.contains(&key.to_string()) {
                mono.push(key.to_string());
            }
        }
    }

    fonts
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_font_definitions_load() {
        let defs = setup_custom_fonts();
        let prop = defs.families.get(&egui::FontFamily::Proportional).unwrap();
        assert!(!prop.is_empty(), "Proportional family must have fonts");
    }

    #[test]
    fn test_cjk_glyphs_present() {
        let ctx = egui::Context::default();
        let defs = setup_custom_fonts();
        ctx.set_fonts(defs);
        let _ = ctx.run(egui::RawInput::default(), |_| {});
        let _ = ctx.run(Default::default(), |_| {});

        // Verify known Japanese track titles render without panic
        let sample_titles = [
            "君はロックを聴かない",
            "ただ君に晴れ",
            "晩餐歌",
            "革命道中 - On The Way",
            "ラグトレイン",
            "感電",
            "星座になれたら",
            "リテラチュア",
            "カワキヲアメク",
            "だから僕は音楽を辞めた",
        ];

        for title in sample_titles {
            let layout = ctx.fonts(|f| {
                f.layout_no_wrap(
                    title.to_string(),
                    egui::FontId::proportional(14.0),
                    egui::Color32::WHITE,
                )
            });
            assert!(!layout.rows.is_empty(), "Failed to layout CJK text: {}", title);
            assert!(layout.rect.width() > 0.0, "CJK text width should be > 0: {}", title);
        }
    }
}
