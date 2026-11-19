use egui::{Color32, Context, Painter, Rect, TextureHandle, TextureId, TextureOptions};

/// Cache for ambient blurred artwork backgrounds.
/// Caches the generated blurred texture across frames and only regenerates
/// when the track changes, ensuring smooth 60+ FPS rendering.
pub struct AmbientBlurCache {
    current_key: Option<String>,
    texture_handle: Option<TextureHandle>,
}

impl Default for AmbientBlurCache {
    fn default() -> Self {
        Self::new()
    }
}

impl AmbientBlurCache {
    pub fn new() -> Self {
        Self {
            current_key: None,
            texture_handle: None,
        }
    }

    /// Generates a downscaled, heavily blurred background texture once per track change (never per-frame) so FPS stays 60+.
    pub fn get_or_build(
        &mut self,
        ctx: &Context,
        track_key: &str,
        raw_image_bytes: Option<&[u8]>,
    ) -> Option<TextureId> {
        let is_same_track = self.current_key.as_deref() == Some(track_key);
        let has_texture = self.texture_handle.is_some();

        // If the track is unchanged and we already have a texture (or we already evaluated it with no bytes available), return cached
        if is_same_track && (has_texture || raw_image_bytes.is_none()) {
            return self.texture_handle.as_ref().map(|t| t.id());
        }

        self.current_key = Some(track_key.to_string());
        self.texture_handle = None;

        if let Some(bytes) = raw_image_bytes {
            if let Ok(dyn_img) = image::load_from_memory(bytes) {
                // Downscale to a small thumbnail (64x64) for extreme speed and natural softening
                let thumbnail = dyn_img.thumbnail_exact(64, 64);

                // Apply multi-pass blur for cinematic, silky ambient glow
                let blurred = thumbnail.blur(12.0).blur(8.0);

                let rgba = blurred.to_rgba8();
                let size = [rgba.width() as usize, rgba.height() as usize];
                let color_image = egui::ColorImage::from_rgba_unmultiplied(size, rgba.as_raw());

                let handle = ctx.load_texture("ambient_blur_art", color_image, TextureOptions::LINEAR);
                self.texture_handle = Some(handle);
            }
        }

        self.texture_handle.as_ref().map(|t| t.id())
    }

    /// Paints the blurred background stretched to fill `rect` with a dark readability scrim (or smooth dark gradient fallback if no cover art).
    pub fn paint_background(
        &self,
        painter: &Painter,
        rect: Rect,
        texture_id: Option<TextureId>,
    ) {
        if rect.width() <= 0.0 || rect.height() <= 0.0 {
            return;
        }

        if let Some(id) = texture_id {
            // Paint the texture stretched over the target rect using egui::Mesh with full UVs [0.0, 0.0] to [1.0, 1.0]
            let mut mesh = egui::Mesh::with_texture(id);
            let uv = Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(1.0, 1.0));
            mesh.add_rect_with_uv(rect, uv, Color32::from_white_alpha(220));
            painter.add(egui::Shape::mesh(mesh));

            // Overlay dark purple-tinted readability scrim to guarantee high text contrast
            let scrim_color = Color32::from_rgba_premultiplied(10, 8, 16, 190);
            painter.rect_filled(rect, egui::Rounding::ZERO, scrim_color);
        } else {
            // Fallback: smooth dark vertical gradient matching Eclipse's purple glass aesthetic
            // #181124 (top) -> #0a0810 (bottom)
            let mut mesh = egui::Mesh::default();
            let top_color = Color32::from_rgb(0x18, 0x11, 0x24);
            let bottom_color = Color32::from_rgb(0x0a, 0x08, 0x10);

            mesh.add_triangle(0, 1, 2);
            mesh.add_triangle(0, 2, 3);

            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.left_top(),
                uv: egui::epaint::WHITE_UV,
                color: top_color,
            });
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.right_top(),
                uv: egui::epaint::WHITE_UV,
                color: top_color,
            });
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.right_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: bottom_color,
            });
            mesh.vertices.push(egui::epaint::Vertex {
                pos: rect.left_bottom(),
                uv: egui::epaint::WHITE_UV,
                color: bottom_color,
            });

            painter.add(egui::Shape::mesh(mesh));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use egui::{pos2, vec2, LayerId, Rect};

    #[test]
    fn test_ambient_blur_cache_creation() {
        let cache = AmbientBlurCache::new();
        assert!(cache.current_key.is_none());
        assert!(cache.texture_handle.is_none());

        let default_cache = AmbientBlurCache::default();
        assert!(default_cache.current_key.is_none());
        assert!(default_cache.texture_handle.is_none());
    }

    #[test]
    fn test_ambient_blur_cache_key_matching_and_fallback() {
        let ctx = Context::default();
        let mut cache = AmbientBlurCache::new();

        // 1. None bytes yields None texture id
        let tex_id = cache.get_or_build(&ctx, "track_1", None);
        assert!(tex_id.is_none());
        assert_eq!(cache.current_key.as_deref(), Some("track_1"));

        // Same track key with None bytes should stay None without error
        let tex_id2 = cache.get_or_build(&ctx, "track_1", None);
        assert!(tex_id2.is_none());

        // 2. Corrupt bytes yields None gracefully without panicking
        let corrupt = [0xde, 0xad, 0xbe, 0xef];
        let tex_id_corrupt = cache.get_or_build(&ctx, "track_2", Some(&corrupt));
        assert!(tex_id_corrupt.is_none());
        assert_eq!(cache.current_key.as_deref(), Some("track_2"));

        // 3. Valid image bytes should produce Some(TextureId)
        let mut img = image::RgbaImage::new(8, 8);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([180, 100, 240, 255]);
        }
        let mut png_bytes = Vec::new();
        let write_res = img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png);
        assert!(write_res.is_ok());

        let tex_id_valid = cache.get_or_build(&ctx, "track_3", Some(&png_bytes));
        assert!(tex_id_valid.is_some());
        assert_eq!(cache.current_key.as_deref(), Some("track_3"));

        // 4. Repeated call with same track_key returns cached texture ID
        let tex_id_cached = cache.get_or_build(&ctx, "track_3", Some(&png_bytes));
        assert_eq!(tex_id_cached, tex_id_valid);
    }

    #[test]
    fn test_paint_background_render_paths() {
        let ctx = Context::default();
        let rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(800.0, 600.0));
        let painter = Painter::new(ctx.clone(), LayerId::background(), rect);
        let mut cache = AmbientBlurCache::new();

        // Test fallback gradient rendering (no texture id)
        cache.paint_background(&painter, rect, None);

        // Test with real texture
        let mut img = image::RgbaImage::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = image::Rgba([100, 50, 150, 255]);
        }
        let mut png_bytes = Vec::new();
        let _ = img.write_to(&mut std::io::Cursor::new(&mut png_bytes), image::ImageFormat::Png);
        let tex_id = cache.get_or_build(&ctx, "test_track", Some(&png_bytes));
        assert!(tex_id.is_some());

        cache.paint_background(&painter, rect, tex_id);

        // Test zero-dimension rect boundary condition (should gracefully return early)
        let zero_rect = Rect::from_min_size(pos2(0.0, 0.0), vec2(0.0, 0.0));
        cache.paint_background(&painter, zero_rect, tex_id);
        cache.paint_background(&painter, zero_rect, None);
    }
}
