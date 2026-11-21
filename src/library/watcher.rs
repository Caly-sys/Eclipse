use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver, Sender};
use std::sync::{Arc, Mutex, RwLock};
use std::thread::JoinHandle;

#[cfg(target_os = "windows")]
#[allow(clippy::upper_case_acronyms)]
mod win32 {
    use std::ffi::c_void;

    pub type HANDLE = *mut c_void;
    pub type BOOL = i32;
    pub type DWORD = u32;

    pub const INVALID_HANDLE_VALUE: HANDLE = -1isize as HANDLE;
    pub const WAIT_OBJECT_0: DWORD = 0x0000_0000;
    pub const FILE_NOTIFY_CHANGE_FILE_NAME: DWORD = 0x0000_0001;
    pub const FILE_NOTIFY_CHANGE_DIR_NAME: DWORD = 0x0000_0002;
    pub const FILE_NOTIFY_CHANGE_LAST_WRITE: DWORD = 0x0000_0010;

    #[link(name = "kernel32")]
    extern "system" {
        pub fn FindFirstChangeNotificationW(
            lpPathName: *const u16,
            bWatchSubtree: BOOL,
            dwNotifyFilter: DWORD,
        ) -> HANDLE;

        pub fn FindNextChangeNotification(hChangeHandle: HANDLE) -> BOOL;

        pub fn FindCloseChangeNotification(hChangeHandle: HANDLE) -> BOOL;

        pub fn WaitForMultipleObjects(
            nCount: DWORD,
            lpHandles: *const HANDLE,
            bWaitAll: BOOL,
            dwMilliseconds: DWORD,
        ) -> DWORD;

        pub fn WaitForSingleObject(
            hHandle: HANDLE,
            dwMilliseconds: DWORD,
        ) -> DWORD;
    }
}

/// Lightweight live folder watcher for library roots.
/// Emits change events when files in the watched roots are created, deleted, or modified.
pub struct LibraryWatcher {
    receiver: Mutex<Receiver<()>>,
    stop_flag: Arc<AtomicBool>,
    roots: Arc<RwLock<Vec<PathBuf>>>,
    thread_handle: Mutex<Option<JoinHandle<()>>>,
}

impl LibraryWatcher {
    /// Create and start a new watcher for the provided library root directories.
    pub fn new<I, P>(roots: I) -> Self
    where
        I: IntoIterator<Item = P>,
        P: Into<PathBuf>,
    {
        let roots_vec: Vec<PathBuf> = roots.into_iter().map(Into::into).collect();
        Self::start(roots_vec)
    }

    /// Start a new watcher with the given root directories.
    pub fn start(roots: Vec<PathBuf>) -> Self {
        let (tx, rx) = channel();
        let stop_flag = Arc::new(AtomicBool::new(false));
        let roots_arc = Arc::new(RwLock::new(roots));

        let stop_clone = Arc::clone(&stop_flag);
        let roots_clone = Arc::clone(&roots_arc);

        let handle = std::thread::Builder::new()
            .name("library-watcher".to_string())
            .spawn(move || {
                run_watcher_loop(roots_clone, stop_clone, tx);
            })
            .ok();

        Self {
            receiver: Mutex::new(rx),
            stop_flag,
            roots: roots_arc,
            thread_handle: Mutex::new(handle),
        }
    }

    /// Dynamically update the set of watched root folders.
    pub fn update_roots(&self, new_roots: Vec<PathBuf>) {
        if let Ok(mut lock) = self.roots.write() {
            *lock = new_roots;
        }
    }

    /// Try to receive a change notification without blocking.
    /// Drains any queued events and returns true if at least one change occurred.
    pub fn try_recv_change(&self) -> bool {
        let mut changed = false;
        if let Ok(rx) = self.receiver.lock() {
            while rx.try_recv().is_ok() {
                changed = true;
            }
        }
        changed
    }

    /// Access the internal event receiver mutex.
    pub fn receiver(&self) -> &Mutex<Receiver<()>> {
        &self.receiver
    }

    /// Request the background watcher thread to stop.
    pub fn stop(&self) {
        self.stop_flag.store(true, Ordering::SeqCst);
    }
}

impl Drop for LibraryWatcher {
    fn drop(&mut self) {
        self.stop();
        if let Ok(mut lock) = self.thread_handle.lock() {
            if let Some(handle) = lock.take() {
                let _ = handle.join();
            }
        }
    }
}

#[cfg(target_os = "windows")]
fn run_watcher_loop(
    roots_arc: Arc<RwLock<Vec<PathBuf>>>,
    stop_flag: Arc<AtomicBool>,
    tx: Sender<()>,
) {
    use std::os::windows::ffi::OsStrExt;
    use std::time::Duration;

    let mut active_handles: Vec<(PathBuf, win32::HANDLE)> = Vec::new();
    let mut last_observed_roots: Vec<PathBuf> = Vec::new();

    let close_handles = |handles: &mut Vec<(PathBuf, win32::HANDLE)>| {
        for (_, handle) in handles.drain(..) {
            unsafe {
                win32::FindCloseChangeNotification(handle);
            }
        }
    };

    while !stop_flag.load(Ordering::Relaxed) {
        let current_roots = match roots_arc.read() {
            Ok(r) => r.clone(),
            Err(_) => Vec::new(),
        };

        if current_roots != last_observed_roots {
            close_handles(&mut active_handles);
            last_observed_roots = current_roots.clone();

            for root in &current_roots {
                if root.exists() && root.is_dir() {
                    let wide: Vec<u16> = root.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
                    let handle = unsafe {
                        win32::FindFirstChangeNotificationW(
                            wide.as_ptr(),
                            1, // Watch subtree
                            win32::FILE_NOTIFY_CHANGE_FILE_NAME
                                | win32::FILE_NOTIFY_CHANGE_DIR_NAME
                                | win32::FILE_NOTIFY_CHANGE_LAST_WRITE,
                        )
                    };

                    if handle != win32::INVALID_HANDLE_VALUE && !handle.is_null() {
                        active_handles.push((root.clone(), handle));
                        if active_handles.len() >= 64 {
                            break;
                        }
                    }
                }
            }
        }

        if active_handles.is_empty() {
            std::thread::sleep(Duration::from_millis(250));
            continue;
        }

        let handles_raw: Vec<win32::HANDLE> = active_handles.iter().map(|(_, h)| *h).collect();
        let wait_res = unsafe {
            win32::WaitForMultipleObjects(
                handles_raw.len() as win32::DWORD,
                handles_raw.as_ptr(),
                0, // Return when any handle is signaled
                250, // Responsive polling timeout
            )
        };

        let signaled_count = handles_raw.len() as win32::DWORD;
        let is_signaled = wait_res < signaled_count;

        if is_signaled {
            let idx = wait_res as usize;
            if let Some((_, handle)) = active_handles.get(idx) {
                unsafe {
                    win32::FindNextChangeNotification(*handle);
                }
            }

            // Debounce: wait 300ms for writes to finish, then drain any additional signaled handles
            std::thread::sleep(Duration::from_millis(300));
            for (_, handle) in &active_handles {
                if unsafe { win32::WaitForSingleObject(*handle, 0) } == win32::WAIT_OBJECT_0 {
                    unsafe {
                        win32::FindNextChangeNotification(*handle);
                    }
                }
            }

            if tx.send(()).is_err() {
                break;
            }
        }
    }

    close_handles(&mut active_handles);
}

#[cfg(not(target_os = "windows"))]
fn run_watcher_loop(
    roots_arc: Arc<RwLock<Vec<PathBuf>>>,
    stop_flag: Arc<AtomicBool>,
    tx: Sender<()>,
) {
    use std::collections::HashMap;
    use std::time::{Duration, SystemTime};

    let mut last_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

    while !stop_flag.load(Ordering::Relaxed) {
        let current_roots = match roots_arc.read() {
            Ok(r) => r.clone(),
            Err(_) => Vec::new(),
        };

        let mut changed = false;
        let mut new_mtimes: HashMap<PathBuf, SystemTime> = HashMap::new();

        for root in &current_roots {
            if let Ok(meta) = std::fs::metadata(root) {
                if let Ok(mtime) = meta.modified() {
                    new_mtimes.insert(root.clone(), mtime);
                    if let Some(prev) = last_mtimes.get(root) {
                        if *prev != mtime {
                            changed = true;
                        }
                    } else if !last_mtimes.is_empty() {
                        changed = true;
                    }
                }
            }
        }

        if last_mtimes.is_empty() && !new_mtimes.is_empty() {
            last_mtimes = new_mtimes;
        } else if changed {
            last_mtimes = new_mtimes;
            std::thread::sleep(Duration::from_millis(300));
            if tx.send(()).is_err() {
                break;
            }
        }

        std::thread::sleep(Duration::from_millis(500));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

    #[test]
    fn test_library_watcher_detects_file_creation() -> Result<(), Box<dyn std::error::Error>> {
        let timestamp = match SystemTime::now().duration_since(UNIX_EPOCH) {
            Ok(d) => d.as_nanos(),
            Err(_) => 0,
        };
        let temp_dir = std::env::temp_dir().join(format!("eclipse_watcher_test_{timestamp}"));
        fs::create_dir_all(&temp_dir)?;

        let watcher = LibraryWatcher::new(vec![temp_dir.clone()]);

        // Small pause for watcher thread to initialize handles
        std::thread::sleep(Duration::from_millis(150));

        // Drain any initial notifications
        let _ = watcher.try_recv_change();

        // Create a file in the watched directory
        let new_file = temp_dir.join("test_audio.flac");
        fs::write(&new_file, b"test content")?;

        // Poll with timeout for watcher to detect the change
        let start = Instant::now();
        let mut detected = false;
        while start.elapsed() < Duration::from_secs(4) {
            if watcher.try_recv_change() {
                detected = true;
                break;
            }
            std::thread::sleep(Duration::from_millis(100));
        }

        assert!(detected, "Watcher should detect new file creation");

        // Clean up
        let _ = fs::remove_file(&new_file);
        let _ = fs::remove_dir_all(&temp_dir);

        Ok(())
    }

    #[test]
    fn test_library_watcher_lifecycle_stop() -> Result<(), Box<dyn std::error::Error>> {
        let temp_dir = std::env::temp_dir().join("eclipse_watcher_empty_test");
        let watcher = LibraryWatcher::new(vec![temp_dir]);
        assert!(!watcher.try_recv_change());
        watcher.stop();
        Ok(())
    }
}
