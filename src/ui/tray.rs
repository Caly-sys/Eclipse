use std::sync::atomic::{AtomicIsize, Ordering};
use std::sync::mpsc::{channel, Receiver};
use tray_icon::{Icon, MouseButton, MouseButtonState, TrayIcon, TrayIconBuilder, TrayIconEvent};

pub static MAIN_HWND: AtomicIsize = AtomicIsize::new(0);

pub fn set_main_hwnd(hwnd: isize) {
    MAIN_HWND.store(hwnd, Ordering::SeqCst);
}

#[cfg(windows)]
pub fn get_main_hwnd() -> isize {
    let current = MAIN_HWND.load(Ordering::SeqCst);
    if current != 0 {
        return current;
    }

    use windows_sys::Win32::Foundation::{BOOL, HWND, LPARAM};
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        EnumWindows, GetParent, GetWindowTextW, GetWindowThreadProcessId, IsWindowVisible,
    };

    let target_pid = std::process::id();

    unsafe extern "system" fn enum_proc(hwnd: HWND, target_pid_param: LPARAM) -> BOOL {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == target_pid_param as u32 {
            let mut title = [0u16; 256];
            let len = GetWindowTextW(hwnd, title.as_mut_ptr(), 256);
            if len > 0 {
                let title_str = String::from_utf16_lossy(&title[..len as usize]);
                if title_str.contains("Eclipse") {
                    MAIN_HWND.store(hwnd, Ordering::SeqCst);
                    return 0; // Found main Eclipse window
                }
            }
        }
        1
    }

    unsafe {
        EnumWindows(Some(enum_proc), target_pid as LPARAM);
    }

    let found = MAIN_HWND.load(Ordering::SeqCst);
    if found != 0 {
        return found;
    }

    // Secondary pass: look for a visible top-level window without parent
    unsafe extern "system" fn visible_top_level_proc(hwnd: HWND, target_pid_param: LPARAM) -> BOOL {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        if pid == target_pid_param as u32 && GetParent(hwnd) == 0 && IsWindowVisible(hwnd) != 0 {
            MAIN_HWND.store(hwnd, Ordering::SeqCst);
            return 0;
        }
        1
    }

    unsafe {
        EnumWindows(Some(visible_top_level_proc), target_pid as LPARAM);
    }

    MAIN_HWND.load(Ordering::SeqCst)
}

#[cfg(not(windows))]
pub fn get_main_hwnd() -> isize {
    0
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    Show,
    TogglePlay,
    Next,
    Previous,
    Exit,
}

pub struct TrayManager {
    _tray_icon: TrayIcon,
    action_rx: Receiver<TrayAction>,
    last_tooltip: String,
}

impl TrayManager {
    pub fn new(ctx: egui::Context) -> Option<Self> {
        let (action_tx, action_rx) = channel();

        // 1. Load Icon
        let icon = load_tray_icon()?;

        // 2. Build TrayIcon without muda menu to avoid Win32 modal message drops
        let tray_icon = TrayIconBuilder::new()
            .with_tooltip("Eclipse — Music Player")
            .with_icon(icon)
            .build()
            .ok()?;

        // 3. Dedicated Background Listener Thread:
        // Uses native Win32 TrackPopupMenuEx with TPM_RETURNCMD on right-click,
        // which returns the clicked menu action synchronously and never drops clicks.
        let tx = action_tx;
        let ctx_clone = ctx;
        std::thread::spawn(move || {
            let tray_channel = TrayIconEvent::receiver();

            loop {
                let mut triggered = false;

                while let Ok(event) = tray_channel.try_recv() {
                    match event {
                        TrayIconEvent::Click {
                            button: MouseButton::Left,
                            button_state: MouseButtonState::Up,
                            ..
                        }
                        | TrayIconEvent::DoubleClick {
                            button: MouseButton::Left,
                            ..
                        } => {
                            #[cfg(windows)]
                            show_window_native();
                            let _ = tx.send(TrayAction::Show);
                            triggered = true;
                        }
                        TrayIconEvent::Click {
                            button: MouseButton::Right,
                            button_state: MouseButtonState::Up,
                            ..
                        } => {
                            #[cfg(windows)]
                            if let Some(act) = show_native_tray_menu() {
                                let _ = tx.send(act);
                                triggered = true;
                            }
                        }
                        _ => {}
                    }
                }

                if triggered {
                    ctx_clone.request_repaint();
                }

                std::thread::sleep(std::time::Duration::from_millis(25));
            }
        });

        Some(Self {
            _tray_icon: tray_icon,
            action_rx,
            last_tooltip: "Eclipse — Music Player".to_string(),
        })
    }

    pub fn try_recv_action(&self) -> Option<TrayAction> {
        self.action_rx.try_recv().ok()
    }

    pub fn update_tooltip(&mut self, text: &str) {
        if self.last_tooltip != text {
            self.last_tooltip = text.to_string();
            let _ = self._tray_icon.set_tooltip(Some(text));
        }
    }
}

#[cfg(windows)]
pub fn show_native_tray_menu() -> Option<TrayAction> {
    use windows_sys::Win32::Foundation::POINT;
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, DestroyMenu, GetCursorPos, PostMessageW, SetForegroundWindow,
        TrackPopupMenuEx, MF_SEPARATOR, MF_STRING, TPM_BOTTOMALIGN, TPM_LEFTALIGN, TPM_NONOTIFY,
        TPM_RETURNCMD, WM_NULL,
    };

    unsafe {
        let hmenu = CreatePopupMenu();
        if hmenu == 0 {
            return None;
        }

        let to_wide = |s: &str| -> Vec<u16> {
            s.encode_utf16().chain(std::iter::once(0)).collect()
        };

        const CMD_SHOW: usize = 1;
        const CMD_PLAY: usize = 2;
        const CMD_PREV: usize = 3;
        const CMD_NEXT: usize = 4;
        const CMD_EXIT: usize = 5;

        AppendMenuW(hmenu, MF_STRING, CMD_SHOW, to_wide("Show Eclipse").as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(hmenu, MF_STRING, CMD_PLAY, to_wide("Play / Pause").as_ptr());
        AppendMenuW(hmenu, MF_STRING, CMD_PREV, to_wide("Previous Track").as_ptr());
        AppendMenuW(hmenu, MF_STRING, CMD_NEXT, to_wide("Next Track").as_ptr());
        AppendMenuW(hmenu, MF_SEPARATOR, 0, std::ptr::null());
        AppendMenuW(hmenu, MF_STRING, CMD_EXIT, to_wide("Exit Eclipse").as_ptr());

        let mut pt = POINT { x: 0, y: 0 };
        GetCursorPos(&mut pt);

        let hwnd = get_main_hwnd();
        if hwnd != 0 {
            SetForegroundWindow(hwnd);
        }

        let cmd = TrackPopupMenuEx(
            hmenu,
            TPM_RETURNCMD | TPM_NONOTIFY | TPM_BOTTOMALIGN | TPM_LEFTALIGN,
            pt.x,
            pt.y,
            hwnd,
            std::ptr::null(),
        );

        DestroyMenu(hmenu);

        if hwnd != 0 {
            PostMessageW(hwnd, WM_NULL, 0, 0);
        }

        if cmd as usize == CMD_SHOW {
            show_window_native();
        }

        match cmd as usize {
            CMD_SHOW => Some(TrayAction::Show),
            CMD_PLAY => Some(TrayAction::TogglePlay),
            CMD_PREV => Some(TrayAction::Previous),
            CMD_NEXT => Some(TrayAction::Next),
            CMD_EXIT => Some(TrayAction::Exit),
            _ => None,
        }
    }
}

#[cfg(windows)]
pub fn show_window_native() {
    use windows_sys::Win32::UI::WindowsAndMessaging::{
        SetForegroundWindow, ShowWindow, SW_RESTORE, SW_SHOW,
    };
    let hwnd = get_main_hwnd();
    if hwnd != 0 {
        unsafe {
            ShowWindow(hwnd, SW_SHOW);
            ShowWindow(hwnd, SW_RESTORE);
            SetForegroundWindow(hwnd);
        }
    }
}

#[cfg(not(windows))]
pub fn show_window_native() {}

#[cfg(windows)]
pub fn hide_window_native() -> bool {
    use windows_sys::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
    let hwnd = get_main_hwnd();
    if hwnd != 0 {
        unsafe {
            ShowWindow(hwnd, SW_HIDE);
        }
        true
    } else {
        false
    }
}

#[cfg(not(windows))]
pub fn hide_window_native() -> bool {
    false
}

fn load_tray_icon() -> Option<Icon> {
    let icon_bytes = include_bytes!("../../assets/eclipse.png");
    if let Ok(img) = image::load_from_memory(icon_bytes) {
        let resized = img.resize_exact(32, 32, image::imageops::FilterType::Lanczos3);
        let rgba = resized.to_rgba8();
        let (width, height) = rgba.dimensions();
        return Icon::from_rgba(rgba.into_raw(), width, height).ok();
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tray_icon_image_loading() {
        let icon = load_tray_icon();
        assert!(icon.is_some(), "Tray icon must load from assets/eclipse.png successfully");
    }

    #[test]
    fn test_tray_action_equality() {
        assert_eq!(TrayAction::Show, TrayAction::Show);
        assert_ne!(TrayAction::Show, TrayAction::Exit);
    }

    #[test]
    fn test_tray_manager_init() {
        let ctx = egui::Context::default();
        let tm = TrayManager::new(ctx);
        println!("TrayManager::new result: is_some = {}", tm.is_some());
    }
}
