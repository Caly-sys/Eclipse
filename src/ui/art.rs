use crate::library::scanner::get_or_extract_cover_art;
use egui::{ColorImage, Context, TextureHandle, TextureOptions};
use std::collections::{HashMap, HashSet, VecDeque};
use std::io::Read;
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::thread;

const MAX_TEXTURE_CACHE_SIZE: usize = 200;

/// Directory for persistent cache of downloaded stream & remote artwork.
pub fn stream_art_cache_dir() -> PathBuf {
    let dir = std::env::temp_dir().join("eclipse_art_cache");
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Helper to decode raw image bytes (PNG, JPEG, WebP) and downsample to 512x512.
pub fn decode_image_bytes(bytes: &[u8]) -> Option<ColorImage> {
    let dyn_img = image::load_from_memory(bytes).ok()?;
    let resized = if dyn_img.width() > 512 || dyn_img.height() > 512 {
        dyn_img.thumbnail(512, 512)
    } else {
        dyn_img
    };
    let rgba = resized.to_rgba8();
    let size = [rgba.width() as usize, rgba.height() as usize];
    Some(ColorImage::from_rgba_unmultiplied(size, rgba.as_raw()))
}

/// Extract clean Spotify resource kind and ID from a URI or URL.
/// Handles `stream:spotify:{id}`, `stream:spotify:track:{id}`, `spotify:track:{id}`, `spotify:{id}`, etc.
pub fn parse_spotify_track_id(input: &str) -> Option<(&'static str, &str)> {
    let (res_type, id_part) = if let Some(rest) = input.strip_prefix("stream:spotify:") {
        if let Some(t_id) = rest.strip_prefix("track:") {
            ("track", t_id)
        } else if let Some(a_id) = rest.strip_prefix("album:") {
            ("album", a_id)
        } else if let Some(p_id) = rest.strip_prefix("playlist:") {
            ("playlist", p_id)
        } else {
            ("track", rest)
        }
    } else {
        let rest = input.strip_prefix("spotify:")?;
        if let Some(t_id) = rest.strip_prefix("track:") {
            ("track", t_id)
        } else if let Some(a_id) = rest.strip_prefix("album:") {
            ("album", a_id)
        } else if let Some(p_id) = rest.strip_prefix("playlist:") {
            ("playlist", p_id)
        } else {
            ("track", rest)
        }
    };

    let clean_id = id_part.split('?').next().unwrap_or(id_part).trim().trim_matches('/');
    if clean_id.is_empty() {
        None
    } else {
        Some((res_type, clean_id))
    }
}

/// Resolves artwork for Spotify tracks/albums/playlists via disk cache or public oEmbed.
pub fn resolve_spotify_artwork(path: &str) -> Option<Vec<u8>> {
    let (res_type, id) = parse_spotify_track_id(path)?;
    let cache_dir = stream_art_cache_dir();

    // 1. Check if cached thumbnail file already exists on disk in temp/eclipse_art_cache
    let cache_file = cache_dir.join(format!("spotify_{id}.img"));
    if cache_file.is_file() {
        if let Ok(bytes) = std::fs::read(&cache_file) {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
    }

    // Also check other potential image extensions
    for ext in &["jpg", "jpeg", "png", "webp"] {
        let alt = cache_dir.join(format!("spotify_{id}.{ext}"));
        if alt.is_file() {
            if let Ok(bytes) = std::fs::read(&alt) {
                if !bytes.is_empty() {
                    return Some(bytes);
                }
            }
        }
    }

    // 2. Fetch public metadata asynchronously via official oEmbed
    let oembed_url = format!("https://open.spotify.com/oembed?url=https://open.spotify.com/{res_type}/{id}");
    let resp = ureq::get(&oembed_url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()?;

    let json_val: serde_json::Value = resp.into_json().ok()?;
    let thumb_url = json_val.get("thumbnail_url")?.as_str()?;

    // 3. Download image bytes
    let img_resp = ureq::get(thumb_url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()?;

    let mut bytes = Vec::new();
    img_resp.into_reader().read_to_end(&mut bytes).ok()?;

    if !bytes.is_empty() {
        let _ = std::fs::write(&cache_file, &bytes);
        Some(bytes)
    } else {
        None
    }
}

/// Resolves remote HTTP/HTTPS artwork with disk caching in eclipse_art_cache.
pub fn resolve_remote_url_artwork(url: &str) -> Option<Vec<u8>> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let cache_dir = stream_art_cache_dir();
    let mut hasher = DefaultHasher::new();
    url.hash(&mut hasher);
    let hash = hasher.finish();
    let cache_file = cache_dir.join(format!("remote_{:016x}.img", hash));

    if cache_file.is_file() {
        if let Ok(bytes) = std::fs::read(&cache_file) {
            if !bytes.is_empty() {
                return Some(bytes);
            }
        }
    }

    let resp = ureq::get(url)
        .timeout(std::time::Duration::from_secs(5))
        .call()
        .ok()?;

    let mut bytes = Vec::new();
    resp.into_reader().read_to_end(&mut bytes).ok()?;

    if !bytes.is_empty() {
        let _ = std::fs::write(&cache_file, &bytes);
        Some(bytes)
    } else {
        None
    }
}

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
                let maybe_img = if track_path.starts_with("stream:spotify:") || track_path.starts_with("spotify:") {
                    // Spotify streaming URI: resolve via oEmbed/disk cache, do NOT open as local file
                    resolve_spotify_artwork(&track_path).and_then(|b| decode_image_bytes(&b))
                } else if track_path.starts_with("stream:") {
                    // Other non-Spotify stream URI: do NOT call std::fs::File::open
                    None
                } else if track_path.starts_with("http://") || track_path.starts_with("https://") {
                    // Remote HTTP/HTTPS image URL: cache to disk and load
                    resolve_remote_url_artwork(&track_path).and_then(|b| decode_image_bytes(&b))
                } else {
                    // Local audio file path
                    let path = Path::new(&track_path);
                    get_or_extract_cover_art(path).and_then(|b| decode_image_bytes(&b))
                };

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

        let color_img = decode_image_bytes(&png_bytes).expect("Failed to decode_image_bytes");
        assert_eq!(color_img.width(), 4);
        assert_eq!(color_img.height(), 4);
    }

    #[test]
    fn test_parse_spotify_track_id() {
        assert_eq!(
            parse_spotify_track_id("stream:spotify:4cOdK2wGLETKBW3PvgPWqT"),
            Some(("track", "4cOdK2wGLETKBW3PvgPWqT"))
        );
        assert_eq!(
            parse_spotify_track_id("stream:spotify:track:4cOdK2wGLETKBW3PvgPWqT"),
            Some(("track", "4cOdK2wGLETKBW3PvgPWqT"))
        );
        assert_eq!(
            parse_spotify_track_id("spotify:track:4cOdK2wGLETKBW3PvgPWqT"),
            Some(("track", "4cOdK2wGLETKBW3PvgPWqT"))
        );
        assert_eq!(
            parse_spotify_track_id("spotify:album:1A2B3C4D5E"),
            Some(("album", "1A2B3C4D5E"))
        );
        assert_eq!(
            parse_spotify_track_id("stream:spotify:playlist:1A2B3C4D5E"),
            Some(("playlist", "1A2B3C4D5E"))
        );
        assert_eq!(
            parse_spotify_track_id("stream:spotify:4cOdK2wGLETKBW3PvgPWqT?si=abc123xyz"),
            Some(("track", "4cOdK2wGLETKBW3PvgPWqT"))
        );
        assert_eq!(parse_spotify_track_id("stream:youtube:dQw4w9WgXcQ"), None);
        assert_eq!(parse_spotify_track_id("C:\\music\\song.flac"), None);
    }

    #[test]
    fn test_stream_art_cache_spotify() {
        let cache_dir = stream_art_cache_dir();
        assert!(cache_dir.exists());
        let test_id = "test_track_dummy_id_123";
        let cache_file = cache_dir.join(format!("spotify_{test_id}.img"));
        let _ = std::fs::write(&cache_file, b"test_image_bytes");
        let result = resolve_spotify_artwork(&format!("stream:spotify:{test_id}"));
        assert_eq!(result, Some(b"test_image_bytes".to_vec()));
        let _ = std::fs::remove_file(&cache_file);
    }
}
