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
    pub fn open() -> Result<Self> {
        let db_path = app_data_dir().join("library.db");
        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
            CREATE TABLE IF NOT EXISTS tracks (
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
            );",
        )?;

        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn upsert_track(&self, t: &Track) -> Result<i64> {
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "UPDATE tracks SET favorite = CASE WHEN favorite = 1 THEN 0 ELSE 1 END WHERE id = ?1",
            params![id],
        )?;
        let mut stmt = conn.prepare("SELECT favorite FROM tracks WHERE id = ?1")?;
        let fav: i32 = stmt.query_row(params![id], |r| r.get(0))?;
        Ok(fav != 0)
    }

    pub fn increment_play_count(&self, id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
        conn.execute("DELETE FROM playlist_items WHERE playlist_id = ?1", params![id])?;
        conn.execute("DELETE FROM playlists WHERE id = ?1", params![id])?;
        Ok(())
    }

    pub fn add_to_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT OR IGNORE INTO playlist_items (playlist_id, track_id, pos)
             VALUES (?1, ?2, (SELECT COALESCE(MAX(pos), 0) + 1 FROM playlist_items WHERE playlist_id = ?1))",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    pub fn remove_from_playlist(&self, playlist_id: i64, track_id: i64) -> Result<()> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "DELETE FROM playlist_items WHERE playlist_id = ?1 AND track_id = ?2",
            params![playlist_id, track_id],
        )?;
        Ok(())
    }

    pub fn get_playlist_tracks(&self, playlist_id: i64) -> Result<Vec<Track>> {
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
        let conn = self.conn.lock().unwrap();
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
}
