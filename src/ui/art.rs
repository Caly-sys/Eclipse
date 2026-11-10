use crate::library::scanner::get_or_extract_cover_art;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet, VecDeque};
use std::path::Path;
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const MAX_TEXTURE_CACHE_SIZE: usize = 200;

pub struct AlbumArtManager {
    textures: HashMap<String, TextureHandle>,
    access_order: VecDeque<String>,
    pending: HashSet<String>,
    failed: HashSet<String>,
    req_tx: Sender<String>,
    res_rx: Receiver<(String, Option<ColorImage>)>,
}

impl Default for AlbumArtManager {
    fn default() -> Self {
        Self::new()
    }
}

impl AlbumArtManager {
    pub fn new() -> Self {
        let (req_tx, req_rx) = channel::<String>();
        let (res_tx, res_rx) = channel::<(String, Option<ColorImage>)>();

        // Dedicated background decoding worker thread to keep UI thread at 60 FPS
        thread::spawn(move || {
            while let Ok(track_path) = req_rx.recv() {
                let path = Path::new(&track_path);
                let maybe_img = get_or_extract_cover_art(path).and_then(|bytes| {
                    let dyn_img = image::load_from_memory(&bytes).ok()?;
                    // Downsample to max 512x512 to preserve GPU memory
                    let resized = if dyn_img.width() > 512 || dyn_img.height() > 512 {
                        dyn_img.thumbnail(512, 512)
                    } else {
                        dyn_img
                    };
                    let rgba = resized.to_rgba8();
                    let size = [rgba.width() as usize, rgba.height() as usize];
                    Some(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
                });

                let _ = res_tx.send((track_path, maybe_img));
            }
        });

        Self {
            textures: HashMap::new(),
            access_order: VecDeque::new(),
            pending: HashSet::new(),
            failed: HashSet::new(),
            req_tx,
            res_rx,
        }
    }

    /// Process completed decodings from background thread without blocking.
    pub fn update(&mut self, ctx: &Context) {
        while let Ok((path, maybe_img)) = self.res_rx.try_recv() {
            self.pending.remove(&path);
            if let Some(color_image) = maybe_img {
                let name = format!("cover_{}", path);
                let texture = ctx.load_texture(name, color_image, TextureOptions::LINEAR);
                self.textures.insert(path.clone(), texture);
                self.access_order.push_back(path.clone());

                // Evict oldest textures when exceeding LRU capacity
                while self.textures.len() > MAX_TEXTURE_CACHE_SIZE {
                    if let Some(oldest) = self.access_order.pop_front() {
                        self.textures.remove(&oldest);
                    } else {
                        break;
                    }
                }

                ctx.request_repaint();
            } else {
                self.failed.insert(path);
            }
        }
    }

    /// Retrieve or request an egui texture for the given audio track path.
    /// Non-blocking: returns None immediately if not yet loaded and dispatches background decode.
    pub fn get_or_load(&mut self, ctx: &Context, track_path: &str) -> Option<&TextureHandle> {
        self.update(ctx);

        if self.failed.contains(track_path) {
            return None;
        }

        if self.textures.contains_key(track_path) {
            // Refresh LRU access order
            if let Some(pos) = self.access_order.iter().position(|p| p == track_path) {
                self.access_order.remove(pos);
            }
            self.access_order.push_back(track_path.to_string());
            return self.textures.get(track_path);
        }

        if !self.pending.contains(track_path) {
            self.pending.insert(track_path.to_string());
            let _ = self.req_tx.send(track_path.to_string());
        }

        None
    }

    /// Check if texture is already loaded
    pub fn get(&self, track_path: &str) -> Option<&TextureHandle> {
        self.textures.get(track_path)
    }

    /// Clear cache
    pub fn clear(&mut self) {
        self.textures.clear();
        self.access_order.clear();
        self.pending.clear();
        self.failed.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_art_manager_initialization() {
        let mut mgr = AlbumArtManager::new();
        assert!(mgr.textures.is_empty());
        assert!(mgr.failed.is_empty());

        mgr.failed.insert("dummy_path".to_string());
        assert!(mgr.failed.contains("dummy_path"));
        mgr.clear();
        assert!(mgr.failed.is_empty());
    }

    #[test]
    fn test_art_manager_async_request() {
        let mut mgr = AlbumArtManager::new();
        let ctx = egui::Context::default();
        let res = mgr.get_or_load(&ctx, "non_existent_file.flac");
        assert!(res.is_none());
        assert!(mgr.pending.contains("non_existent_file.flac"));
        // Second lookup does not double-queue
        let res2 = mgr.get_or_load(&ctx, "non_existent_file.flac");
        assert!(res2.is_none());
    }

    #[test]
    fn test_synthetic_image_decoding() {
        use image::{ImageBuffer, Rgba};
        let mut img: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::new(4, 4);
        for pixel in img.pixels_mut() {
            *pixel = Rgba([180, 140, 255, 255]);
        }
        let mut buffer = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buffer, image::ImageFormat::Png).expect("Failed to encode png");
        let png_bytes = buffer.into_inner();

        let decoded = image::load_from_memory(&png_bytes).expect("Failed to decode png");
        assert_eq!(decoded.width(), 4);
        assert_eq!(decoded.height(), 4);
    }
}
