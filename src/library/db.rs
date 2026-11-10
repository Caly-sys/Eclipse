use crate::config::app_data_dir;
use crate::library::model::{Playlist, Track};
use rusqlite::{params, Connection, Result};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

fn map_track_row(row: &rusqlite::Row<'_>) -> rusqlite::Result<Track> {
    Ok(Track {
        id: row.get(0)?,
        path: row.get(1)?,
        title: row.get(2)?,
        artist: row.get(3)?,
        album_artist: row.get(4)?,
        album: row.get(5)?,
        genre: row.get(6)?,
        year: row.get(7)?,
        track_number: row.get(8)?,
        duration_secs: row.get(9)?,
        bitrate_kbps: row.get(10)?,
        sample_rate: row.get(11)?,
        channels: row.get(12)?,
        format: row.get(13)?,
        file_size: row.get(14)?,
        play_count: row.get(15)?,
        favorite: row.get::<_, i32>(16)? != 0,
        has_art: row.get::<_, i32>(17)? != 0,
        folder: row.get(18)?,
    })
}

#[derive(Clone)]
pub struct Database {
    conn: Arc<Mutex<Connection>>,
}

impl Database {
    fn init_schema(conn: &Connection) -> Result<()> {
        conn.execute_batch(
            "CREATE TABLE IF NOT EXISTS tracks (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                path TEXT UNIQUE NOT NULL,
                source TEXT DEFAULT 'local',
                url TEXT,
                thumb TEXT,
                title TEXT DEFAULT '',
                artist TEXT DEFAULT '',
                albumartist TEXT DEFAULT '',
                album TEXT DEFAULT '',
                genre TEXT DEFAULT '',
                year INTEGER DEFAULT 0,
                track INTEGER DEFAULT 0,
                tracktotal INTEGER DEFAULT 0,
                disc INTEGER DEFAULT 0,
                disctotal INTEGER DEFAULT 0,
                composer TEXT,
                comment TEXT,
                duration REAL DEFAULT 0.0,
                bitrate INTEGER DEFAULT 0,
                samplerate INTEGER DEFAULT 0,
                bitdepth INTEGER DEFAULT 0,
                channels INTEGER DEFAULT 2,
                codec TEXT DEFAULT '',
                lossless INTEGER DEFAULT 0,
                filesize INTEGER DEFAULT 0,
                mtime REAL DEFAULT 0,
                added REAL DEFAULT 0,
                play_count INTEGER DEFAULT 0,
                skip_count INTEGER DEFAULT 0,
                last_played REAL,
                favorite INTEGER DEFAULT 0,
                rating INTEGER DEFAULT 0,
                has_art INTEGER DEFAULT 0,
                rg_track REAL,
                rg_album REAL,
                lufs REAL,
                peak REAL,
                dr REAL,
                folder TEXT DEFAULT ''
            );
            CREATE INDEX IF NOT EXISTS ix_artist ON tracks(artist);
            CREATE INDEX IF NOT EXISTS ix_album ON tracks(album);

            CREATE TABLE IF NOT EXISTS playlists (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                name TEXT NOT NULL,
                created REAL DEFAULT 0.0,
                description TEXT DEFAULT '',
                smart TEXT DEFAULT '',
                origin TEXT DEFAULT '',
                link TEXT DEFAULT '',
                thumb TEXT DEFAULT '',
                synced REAL DEFAULT 0.0
            );

            CREATE TABLE IF NOT EXISTS playlist_items (
                playlist_id INTEGER NOT NULL,
                track_id INTEGER NOT NULL,
                pos INTEGER DEFAULT 0,
                PRIMARY KEY (playlist_id, track_id)
            );

            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                track_id INTEGER NOT NULL,
                timestamp REAL NOT NULL
            );

            CREATE TABLE IF NOT EXISTS stream_cache (
                key TEXT PRIMARY KEY,
                provider TEXT NOT NULL,
                resource_type TEXT NOT NULL,
                data TEXT NOT NULL,
                updated_at REAL NOT NULL
            );
            CREATE INDEX IF NOT EXISTS ix_stream_cache_key ON stream_cache(key);",
        )?;
        Ok(())
    }

    pub fn open() -> Result<Self> {
        let db_path = app_data_dir().join("library.db");
        let conn = Connection::open(db_path)?;
        let _ = conn.execute_batch("PRAGMA journal_mode = WAL;");
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn open_in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()?;
        Self::init_schema(&conn)?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    #[inline]
    pub fn lock_conn(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().unwrap_or_else(|e| e.into_inner())
    }

    pub fn upsert_tracks_batch(&self, tracks: &[Track]) -> Result<usize> {
        if tracks.is_empty() {
            return Ok(0);
        }
        let mut conn = self.lock_conn();
        let tx = conn.transaction()?;
        {
            let mut stmt = tx.prepare_cached(
                "INSERT INTO tracks (
                    path, title, artist, albumartist, album, genre, year, track,
                    duration, bitrate, samplerate, channels, codec, filesize,
                    has_art, folder
                ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
                ON CONFLICT(path) DO UPDATE SET
                    title = excluded.title,
                    artist = excluded.artist,
                    albumartist = excluded.albumartist,
                    album = excluded.album,
                    genre = excluded.genre,
                    year = excluded.year,
                    track = excluded.track,
                    duration = excluded.duration,
                    bitrate = excluded.bitrate,
                    samplerate = excluded.samplerate,
                    channels = excluded.channels,
                    codec = excluded.codec,
                    filesize = excluded.filesize,
                    has_art = excluded.has_art,
                    folder = excluded.folder;",
            )?;

            for t in tracks {
                stmt.execute(params![
                    t.path,
                    t.title,
                    t.artist,
                    t.album_artist,
                    t.album,
                    t.genre,
                    t.year,
                    t.track_number,
                    t.duration_secs,
                    t.bitrate_kbps,
                    t.sample_rate,
                    t.channels,
                    t.format,
                    t.file_size,
                    t.has_art as i32,
                    t.folder,
                ])?;
            }
        }
        tx.commit()?;
        Ok(tracks.len())
    }

    pub fn upsert_track(&self, t: &Track) -> Result<i64> {
        let conn = self.lock_conn();
        conn.execute(
            "INSERT INTO tracks (
                path, title, artist, albumartist, album, genre, year, track,
                duration, bitrate, samplerate, channels, codec, filesize,
                has_art, folder
            ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16)
            ON CONFLICT(path) DO UPDATE SET
                title = excluded.title,
                artist = excluded.artist,
                albumartist = excluded.albumartist,
                album = excluded.album,
                genre = excluded.genre,
                year = excluded.year,
                track = excluded.track,
                duration = excluded.duration,
                bitrate = excluded.bitrate,
                samplerate = excluded.samplerate,
                channels = excluded.channels,
                codec = excluded.codec,
                filesize = excluded.filesize,
                has_art = excluded.has_art,
                folder = excluded.folder;",
            params![
                t.path,
                t.title,
                t.artist,
                t.album_artist,
                t.album,
                t.genre,
                t.year,
                t.track_number,
                t.duration_secs,
                t.bitrate_kbps,
                t.sample_rate,
                t.channels,
                t.format,
                t.file_size,
                t.has_art as i32,
                t.folder,
            ],
        )?;

        let mut stmt = conn.prepare("SELECT id FROM tracks WHERE path = ?1")?;
        let id: i64 = stmt.query_row(params![t.path], |row| row.get(0))?;
        Ok(id)
    }

    pub fn get_all_tracks(&self) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track ASC",
        )?;

        let rows = stmt.query_map([], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_track_by_id(&self, id: i64) -> Result<Option<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks WHERE id = ?1",
        )?;

        let mut rows = stmt.query_map(params![id], map_track_row)?;
        if let Some(track) = rows.next() {
            Ok(Some(track?))
        } else {
            Ok(None)
        }
    }

    pub fn toggle_favorite(&self, id: i64) -> Result<bool> {
        let conn = self.lock_conn();
        conn.execute(
            "UPDATE tracks SET favorite = CASE WHEN favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        let mut stmt = conn.prepare("SELECT favorite FROM tracks WHERE id = ?1")?;
        let fav: i32 = stmt.query_row(params![id], |r| r.get(0))?;
        Ok(fav != 0)
    }

    pub fn increment_play_count(&self, id: i64) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute(
            "UPDATE tracks SET play_count = play_count + 1 WHERE id = ?1",
            params![id],
        )?;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        conn.execute(
            "INSERT INTO history (track_id, timestamp) VALUES (?1, ?2)",
            params![id, now],
        )?;
        Ok(())
    }

    pub fn get_playlists(&self) -> Result<Vec<Playlist>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT p.id, p.name, COALESCE(p.created, 0.0), COALESCE(p.description, ''),
                    COUNT(pi.track_id) as track_count
             FROM playlists p
             LEFT JOIN playlist_items pi ON p.id = pi.playlist_id
             GROUP BY p.id
             ORDER BY p.name COLLATE NOCASE",
        )?;

        let rows = stmt.query_map([], |row| {
            Ok(Playlist {
                id: row.get(0)?,
                name: row.get(1)?,
                created: row.get(2)?,
                description: row.get(3)?,
                track_count: row.get::<_, i64>(4)? as usize,
            })
        })?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn create_playlist(&self, name: &str) -> Result<i64> {
        let conn = self.lock_conn();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        conn.execute(
            "INSERT INTO playlists (name, created, description) VALUES (?1, ?2, '')",
            params![name, now],
        )?;
        Ok(conn.last_insert_rowid())
    }

    pub fn delete_playlist(&self, id: i64) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute("DELETE FROM playlist_items WHERE playlist_id = ?1", params![id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_to_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute(
            "INSERT OR IGNORE INTO playlist_items (playlist_id, track_id, pos)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(pos), 0) + 1 FROM playlist_items WHERE playlist_id = ?1))",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    #[inline]
    pub fn add_track_to_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        self.add_to_playlist(playlist_id, track_id)
    }

    pub fn remove_from_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    pub fn get_playlist_tracks(&self, playlist_id: i64) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.path, COALESCE(t.title,''), COALESCE(t.artist,''), COALESCE(t.albumartist,''),
                    COALESCE(t.album,''), COALESCE(t.genre,''), COALESCE(t.year,0), COALESCE(t.track,0),
                    COALESCE(t.duration,0.0), COALESCE(t.bitrate,0), COALESCE(t.samplerate,0),
                    COALESCE(t.channels,2), COALESCE(t.codec,''), COALESCE(t.filesize,0),
                    COALESCE(t.play_count,0), COALESCE(t.favorite,0), COALESCE(t.has_art,0), COALESCE(t.folder,'')
             FROM tracks t
             INNER JOIN playlist_items pi ON t.id = pi.track_id
             WHERE pi.playlist_id = ?1
             ORDER BY pi.pos ASC",
        )?;

        let rows = stmt.query_map(params![playlist_id], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_recent_history(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT t.id, t.path, COALESCE(t.title,''), COALESCE(t.artist,''), COALESCE(t.albumartist,''),
                    COALESCE(t.album,''), COALESCE(t.genre,''), COALESCE(t.year,0), COALESCE(t.track,0),
                    COALESCE(t.duration,0.0), COALESCE(t.bitrate,0), COALESCE(t.samplerate,0),
                    COALESCE(t.channels,2), COALESCE(t.codec,''), COALESCE(t.filesize,0),
                    COALESCE(t.play_count,0), COALESCE(t.favorite,0), COALESCE(t.has_art,0), COALESCE(t.folder,'')
             FROM tracks t
             INNER JOIN (
                 SELECT track_id, MAX(timestamp) as ts FROM history GROUP BY track_id ORDER BY ts DESC LIMIT ?1
             ) h ON t.id = h.track_id
             ORDER BY h.ts DESC",
        )?;

        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_recently_played(&self, limit: usize) -> Result<Vec<Track>> {
        let history = self.get_recent_history(limit)?;
        if !history.is_empty() {
            return Ok(history);
        }
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks WHERE play_count > 0 ORDER BY play_count DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_most_played(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY play_count DESC, title COLLATE NOCASE ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_recently_added(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY added DESC, id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_favorites(&self) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks WHERE favorite != 0 ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track ASC",
        )?;
        let rows = stmt.query_map([], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_never_played(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks WHERE play_count = 0 ORDER BY artist COLLATE NOCASE, album COLLATE NOCASE, track ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_smart_random_discovery(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY RANDOM() LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_top_played_tracks(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY play_count DESC, id ASC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_top_played(&self, limit: usize) -> Result<Vec<Track>> {
        self.get_top_played_tracks(limit)
    }

    pub fn get_recently_added_tracks(&self, limit: usize) -> Result<Vec<Track>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare(
            "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                    COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                    COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                    COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                    COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
             FROM tracks ORDER BY id DESC LIMIT ?1",
        )?;
        let rows = stmt.query_map(params![limit as i64], map_track_row)?;
        let mut list = Vec::new();
        for r in rows {
            list.push(r?);
        }
        Ok(list)
    }

    pub fn get_recently_added(&self, limit: usize) -> Result<Vec<Track>> {
        self.get_recently_added_tracks(limit)
    }

    pub fn get_most_recent_or_first_track(&self) -> Option<Track> {
        if let Ok(history) = self.get_recent_history(1) {
            if let Some(track) = history.into_iter().next() {
                return Some(track);
            }
        }
        let conn = self.conn.lock().ok()?;
        let mut stmt = conn
            .prepare(
                "SELECT id, path, COALESCE(title,''), COALESCE(artist,''), COALESCE(albumartist,''),
                        COALESCE(album,''), COALESCE(genre,''), COALESCE(year,0), COALESCE(track,0),
                        COALESCE(duration,0.0), COALESCE(bitrate,0), COALESCE(samplerate,0),
                        COALESCE(channels,2), COALESCE(codec,''), COALESCE(filesize,0),
                        COALESCE(play_count,0), COALESCE(favorite,0), COALESCE(has_art,0), COALESCE(folder,'')
                 FROM tracks ORDER BY id ASC LIMIT 1",
            )
            .ok()?;
        let mut rows = stmt.query_map([], map_track_row).ok()?;
        rows.next()?.ok()
    }

    pub fn get_total_listening_hours(&self) -> f64 {
        let conn = match self.conn.lock() {
            Ok(c) => c,
            Err(_) => return 0.0,
        };

        let track_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM tracks", [], |row| row.get(0))
            .unwrap_or(0);

        if track_count == 0 {
            return 0.0;
        }

        let total_secs: f64 = conn
            .query_row(
                "SELECT COALESCE(SUM(duration * play_count), 0.0) FROM tracks",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0.0);

        let hours = total_secs / 3600.0;
        hours.max(0.1)
    }

    pub fn total_listening_time_hours(&self) -> f64 {
        self.get_total_listening_hours()
    }

    pub fn set_track_play_count(&self, id: i64, count: u32) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute(
            "UPDATE tracks SET play_count = ?1 WHERE id = ?2",
            params![count, id],
        )?;
        Ok(())
    }

    /// Retrieve cached streaming metadata JSON by unique key.
    pub fn get_stream_cache(&self, key: &str) -> Result<Option<String>> {
        let conn = self.lock_conn();
        let mut stmt = conn.prepare("SELECT data FROM stream_cache WHERE key = ?1")?;
        let mut rows = stmt.query(params![key])?;
        if let Some(row) = rows.next()? {
            let data: String = row.get(0)?;
            Ok(Some(data))
        } else {
            Ok(None)
        }
    }

    /// Insert or update streaming metadata JSON in the cache table.
    pub fn set_stream_cache(
        &self,
        key: &str,
        provider: &str,
        resource_type: &str,
        data: &str,
    ) -> Result<()> {
        let conn = self.lock_conn();
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs_f64())
            .unwrap_or(0.0);
        conn.execute(
            "INSERT INTO stream_cache (key, provider, resource_type, data, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5)
             ON CONFLICT(key) DO UPDATE SET
                 provider = excluded.provider,
                 resource_type = excluded.resource_type,
                 data = excluded.data,
                 updated_at = excluded.updated_at;",
            params![key, provider, resource_type, data, now],
        )?;
        Ok(())
    }

    /// Delete a cached streaming entry by key.
    pub fn delete_stream_cache(&self, key: &str) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute("DELETE FROM stream_cache WHERE key = ?1", params![key])?;
        Ok(())
    }

    /// Clear all cached streaming metadata entries.
    pub fn clear_stream_cache(&self) -> Result<()> {
        let conn = self.lock_conn();
        conn.execute("DELETE FROM stream_cache", [])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dummy_track(path: &str, title: &str, artist: &str, duration: f64) -> Track {
        Track {
            id: 0,
            path: path.to_string(),
            title: title.to_string(),
            artist: artist.to_string(),
            album: "Test Album".to_string(),
            album_artist: artist.to_string(),
            genre: "Electronic".to_string(),
            year: 2024,
            track_number: 1,
            duration_secs: duration,
            bitrate_kbps: 1411,
            sample_rate: 44100,
            channels: 2,
            format: "FLAC".to_string(),
            file_size: 10_000_000,
            play_count: 0,
            favorite: false,
            has_art: false,
            folder: "C:/music".to_string(),
        }
    }

    #[test]
    fn test_top_played_tracks_ordering() -> Result<()> {
        let db = Database::open_in_memory()?;
        let id1 = db.upsert_track(&dummy_track("track1.flac", "Track 1", "Artist A", 180.0))?;
        let id2 = db.upsert_track(&dummy_track("track2.flac", "Track 2", "Artist B", 200.0))?;
        let id3 = db.upsert_track(&dummy_track("track3.flac", "Track 3", "Artist C", 220.0))?;

        db.set_track_play_count(id1, 5)?;
        db.set_track_play_count(id2, 25)?;
        db.set_track_play_count(id3, 12)?;

        let top = db.get_top_played_tracks(2)?;
        assert_eq!(top.len(), 2);
        assert_eq!(top[0].id, id2);
        assert_eq!(top[0].play_count, 25);
        assert_eq!(top[1].id, id3);
        assert_eq!(top[1].play_count, 12);

        // Test alias get_top_played
        let top_alias = db.get_top_played(2)?;
        assert_eq!(top_alias.len(), 2);
        assert_eq!(top_alias[0].id, id2);
        Ok(())
    }

    #[test]
    fn test_recently_added_tracks_ordering() -> Result<()> {
        let db = Database::open_in_memory()?;
        let id1 = db.upsert_track(&dummy_track("track1.flac", "Track 1", "Artist A", 180.0))?;
        let id2 = db.upsert_track(&dummy_track("track2.flac", "Track 2", "Artist B", 200.0))?;
        let id3 = db.upsert_track(&dummy_track("track3.flac", "Track 3", "Artist C", 220.0))?;

        let recent = db.get_recently_added_tracks(2)?;
        assert_eq!(recent.len(), 2);
        assert_eq!(recent[0].id, id3);
        assert_eq!(recent[1].id, id2);

        // Test alias get_recently_added
        let recent_alias = db.get_recently_added(3)?;
        assert_eq!(recent_alias.len(), 3);
        assert_eq!(recent_alias[0].id, id3);
        assert_eq!(recent_alias[1].id, id2);
        assert_eq!(recent_alias[2].id, id1);
        Ok(())
    }

    #[test]
    fn test_most_recent_or_first_track() -> Result<()> {
        let db = Database::open_in_memory()?;

        // Empty DB returns None
        assert!(db.get_most_recent_or_first_track().is_none());

        // Insert tracks
        let id1 = db.upsert_track(&dummy_track("track1.flac", "First Track", "Artist A", 180.0))?;
        let id2 = db.upsert_track(&dummy_track("track2.flac", "Second Track", "Artist B", 200.0))?;

        // No history yet -> returns first track (id1)
        let first = db.get_most_recent_or_first_track();
        assert!(first.is_some());
        assert_eq!(first.unwrap().id, id1);

        // Play track 2 (adds to history)
        db.increment_play_count(id2)?;

        // Now most recent track from history should be id2
        let recent = db.get_most_recent_or_first_track();
        assert!(recent.is_some());
        assert_eq!(recent.unwrap().id, id2);

        Ok(())
    }

    #[test]
    fn test_total_listening_hours_calculation() -> Result<()> {
        let db = Database::open_in_memory()?;

        // Empty database returns 0.0
        assert_eq!(db.get_total_listening_hours(), 0.0);
        assert_eq!(db.total_listening_time_hours(), 0.0);

        // Add a track with duration 3600s, 0 plays
        let id1 = db.upsert_track(&dummy_track("track1.flac", "Track 1", "Artist", 3600.0))?;

        // Tracks exist but 0 plays -> returns minimum 0.1
        assert_eq!(db.get_total_listening_hours(), 0.1);
        assert_eq!(db.total_listening_time_hours(), 0.1);

        // Set play_count to 2 -> 7200 seconds = 2.0 hours
        db.set_track_play_count(id1, 2)?;
        let hours = db.get_total_listening_hours();
        assert!((hours - 2.0).abs() < 1e-4);
        assert!((db.total_listening_time_hours() - 2.0).abs() < 1e-4);

        Ok(())
    }

    #[test]
    fn test_stream_cache_crud() -> Result<()> {
        let db = Database::open_in_memory()?;
        let key = "spotify:track:4cOdK2wGLETKBW3PvgPWqT";

        // Initially empty
        assert_eq!(db.get_stream_cache(key)?, None);

        // Insert
        let payload = r#"{"id":"4cOdK2wGLETKBW3PvgPWqT","title":"Never Gonna Give You Up"}"#;
        db.set_stream_cache(key, "spotify", "track", payload)?;
        assert_eq!(db.get_stream_cache(key)?, Some(payload.to_string()));

        // Update
        let updated = r#"{"id":"4cOdK2wGLETKBW3PvgPWqT","title":"Updated Title"}"#;
        db.set_stream_cache(key, "spotify", "track", updated)?;
        assert_eq!(db.get_stream_cache(key)?, Some(updated.to_string()));

        // Delete single
        db.delete_stream_cache(key)?;
        assert_eq!(db.get_stream_cache(key)?, None);

        // Clear all
        db.set_stream_cache("k1", "spotify", "album", "data1")?;
        db.set_stream_cache("k2", "spotify", "playlist", "data2")?;
        assert_eq!(db.get_stream_cache("k1")?, Some("data1".to_string()));
        assert_eq!(db.get_stream_cache("k2")?, Some("data2".to_string()));
        db.clear_stream_cache()?;
        assert_eq!(db.get_stream_cache("k1")?, None);
        assert_eq!(db.get_stream_cache("k2")?, None);

        Ok(())
    }
}
