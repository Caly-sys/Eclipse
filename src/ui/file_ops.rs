//! Native Windows and cross-platform file operations.
//!
//! Provides production-grade helpers for:
//! - Revealing files/folders in File Explorer without freezing egui (using COM/Win32 Shell API).
//! - Moving files to the Windows Recycle Bin (using `SHFileOperationW`).
//! - Normalizing paths (canonicalization, stripping verbatim `\\?\` and `\\?\UNC\` prefixes, slash conversions).

use std::path::Path;

#[cfg(target_os = "windows")]
#[cfg(target_pointer_width = "64")]
#[repr(C)]
#[allow(non_snake_case, dead_code, clippy::upper_case_acronyms)]
struct SHFILEOPSTRUCTW {
    hwnd: *mut std::ffi::c_void,
    wFunc: u32,
    pFrom: *const u16,
    pTo: *const u16,
    fFlags: u16,
    fAnyOperationsAborted: i32,
    hNameMappings: *mut std::ffi::c_void,
    lpszProgressTitle: *const u16,
}

#[cfg(target_os = "windows")]
#[cfg(target_pointer_width = "32")]
#[repr(C, packed(1))]
#[allow(non_snake_case, dead_code, clippy::upper_case_acronyms)]
struct SHFILEOPSTRUCTW {
    hwnd: *mut std::ffi::c_void,
    wFunc: u32,
    pFrom: *const u16,
    pTo: *const u16,
    fFlags: u16,
    fAnyOperationsAborted: i32,
    hNameMappings: *mut std::ffi::c_void,
    lpszProgressTitle: *const u16,
}

#[cfg(target_os = "windows")]
#[link(name = "ole32")]
#[allow(dead_code)]
extern "system" {
    fn CoInitializeEx(pv_reserved: *mut std::ffi::c_void, dw_co_init: u32) -> i32;
    fn CoUninitialize();
}

#[cfg(target_os = "windows")]
#[link(name = "shell32")]
#[allow(dead_code)]
extern "system" {
    fn ILCreateFromPathW(psz_path: *const u16) -> *mut std::ffi::c_void;
    fn ILFree(pidl: *mut std::ffi::c_void);
    fn SHOpenFolderAndSelectItems(
        pidl_folder: *mut std::ffi::c_void,
        cidl: u32,
        apidl: *const *mut std::ffi::c_void,
        dw_flags: u32,
    ) -> i32;
    fn SHFileOperationW(lp_file_op: *mut SHFILEOPSTRUCTW) -> i32;
}

/// Normalizes a file system path for Windows Explorer consumption.
///
/// - Canonicalizes existing paths to resolve symlinks and relative segments,
///   falling back cleanly to the original path on canonicalization errors.
/// - Converts all forward slashes `/` to standard Windows backslashes `\`.
/// - Strips Windows verbatim prefixes (`\\?\UNC\` -> `\\`, `\\?\` -> empty string).
/// - Returns `None` if the resulting string is empty.
#[allow(dead_code)]
pub fn normalize_windows_explorer_path(path: &Path) -> Option<String> {
    let path_buf = if path.exists() {
        match path.canonicalize() {
            Ok(canon) => canon,
            Err(_) => path.to_path_buf(),
        }
    } else {
        path.to_path_buf()
    };

    let raw_str = path_buf.to_string_lossy();
    if raw_str.is_empty() {
        return None;
    }

    // Convert forward slashes to backslashes first for uniform prefix matching
    let unified = raw_str.replace('/', "\\");

    // Strip verbatim prefixes: check \\?\UNC\ before \\?\
    let stripped = if let Some(unc_suffix) = unified.strip_prefix(r"\\?\UNC\") {
        format!(r"\\{}", unc_suffix)
    } else if let Some(unc_suffix) = unified.strip_prefix(r"\\?\unc\") {
        format!(r"\\{}", unc_suffix)
    } else if let Some(local_suffix) = unified.strip_prefix(r"\\?\") {
        local_suffix.to_string()
    } else {
        unified
    };

    if stripped.is_empty() {
        None
    } else {
        Some(stripped)
    }
}

/// Encodes a string slice into double-null-terminated UTF-16 code units,
/// as required by Win32 shell operations (`SHFILEOPSTRUCTW::pFrom`).
#[allow(dead_code)]
pub fn to_double_null_terminated_wide(s: &str) -> Vec<u16> {
    let mut wide: Vec<u16> = s.encode_utf16().collect();
    wide.push(0);
    wide.push(0);
    wide
}

/// Reveals a file or directory in Windows File Explorer or the platform's default file manager.
///
/// - If the file does not exist on disk: checks if its parent directory exists;
///   if the parent exists, reveals/opens the parent directory; otherwise logs a warning
///   and returns cleanly without launching Explorer into the default Documents folder.
/// - On Windows:
///   - Runs on a dedicated lightweight background thread so COM initialization never stalls the egui UI thread.
///   - Uses raw Win32 FFI (`CoInitializeEx` -> `ILCreateFromPathW` -> `SHOpenFolderAndSelectItems` -> `ILFree` -> `CoUninitialize`).
///   - Falls back to `explorer.exe .raw_arg("/select,\"{}\"")` ONLY if PIDL creation or `SHOpenFolderAndSelectItems` fails.
/// - On non-Windows:
///   - Provides safe fallback using `open -R` (macOS) or `xdg-open` (Linux).
#[allow(dead_code)]
pub fn reveal_in_file_explorer(path: &Path) {
    let target_path = if path.exists() {
        path.to_path_buf()
    } else if let Some(parent) = path.parent() {
        if parent.exists() {
            parent.to_path_buf()
        } else {
            eprintln!(
                "[WARN] reveal_in_file_explorer: Target path and parent directory do not exist: {}",
                path.display()
            );
            return;
        }
    } else {
        eprintln!(
            "[WARN] reveal_in_file_explorer: Target path does not exist: {}",
            path.display()
        );
        return;
    };

    #[cfg(target_os = "windows")]
    {
        let Some(normalized) = normalize_windows_explorer_path(&target_path) else {
            eprintln!(
                "[WARN] reveal_in_file_explorer: Failed to normalize target path: {}",
                target_path.display()
            );
            return;
        };

        let _ = std::thread::spawn(move || {
            use std::os::windows::process::CommandExt;

            let hr = unsafe { CoInitializeEx(std::ptr::null_mut(), 0x2 /* COINIT_APARTMENTTHREADED */) };
            let co_initialized = hr >= 0;

            let mut wide: Vec<u16> = normalized.encode_utf16().collect();
            wide.push(0);

            let mut success = false;
            let pidl = unsafe { ILCreateFromPathW(wide.as_ptr()) };
            if !pidl.is_null() {
                let res = unsafe { SHOpenFolderAndSelectItems(pidl, 0, std::ptr::null(), 0) };
                if res >= 0 {
                    success = true;
                }
                unsafe { ILFree(pidl) };
            }

            if !success {
                let _ = std::process::Command::new("explorer.exe")
                    .raw_arg(format!("/select,\"{}\"", normalized))
                    .spawn();
            }

            if co_initialized {
                unsafe { CoUninitialize() };
            }
        });
    }

    #[cfg(not(target_os = "windows"))]
    {
        #[cfg(target_os = "macos")]
        {
            let _ = std::process::Command::new("open")
                .arg("-R")
                .arg(&target_path)
                .spawn();
        }

        #[cfg(not(target_os = "macos"))]
        {
            let open_target = if target_path.is_dir() {
                target_path
            } else {
                match target_path.parent() {
                    Some(p) => p.to_path_buf(),
                    None => target_path,
                }
            };
            let _ = std::process::Command::new("xdg-open")
                .arg(&open_target)
                .spawn();
        }
    }
}

/// Moves a file to the system Recycle Bin (or deletes it if unsupported).
///
/// - If `!path.exists()`, returns `Ok(())` (allows cleaning up stale library records).
/// - On Windows:
///   - Calls `SHFileOperationW` (`shell32.dll`) with `FO_DELETE` and `FOF_ALLOWUNDO`.
///   - Validates that `ret == 0` and `fAnyOperationsAborted == 0`.
/// - On non-Windows:
///   - Falls back to `std::fs::remove_file(path)`.
#[allow(dead_code)]
pub fn move_to_recycle_bin(path: &Path) -> Result<(), String> {
    if !path.exists() {
        return Ok(());
    }

    #[cfg(target_os = "windows")]
    {
        let normalized = match normalize_windows_explorer_path(path) {
            Some(s) => s,
            None => path.to_string_lossy().replace('/', "\\"),
        };

        let wide = to_double_null_terminated_wide(&normalized);

        const FO_DELETE: u32 = 3;
        const FOF_ALLOWUNDO: u16 = 0x0040;
        const FOF_NOCONFIRMATION: u16 = 0x0010;
        const FOF_SILENT: u16 = 0x0004;
        const FOF_NOERRORUI: u16 = 0x0400;

        let flags = FOF_ALLOWUNDO | FOF_NOCONFIRMATION | FOF_SILENT | FOF_NOERRORUI;

        let mut file_op = SHFILEOPSTRUCTW {
            hwnd: std::ptr::null_mut(),
            wFunc: FO_DELETE,
            pFrom: wide.as_ptr(),
            pTo: std::ptr::null(),
            fFlags: flags,
            fAnyOperationsAborted: 0,
            hNameMappings: std::ptr::null_mut(),
            lpszProgressTitle: std::ptr::null(),
        };

        let ret = unsafe { SHFileOperationW(&mut file_op) };
        if ret != 0 {
            return Err(format!(
                "SHFileOperationW failed with error code: 0x{:X} ({})",
                ret, ret
            ));
        }

        if file_op.fAnyOperationsAborted != 0 {
            return Err("File operation was aborted by user or system".to_string());
        }

        Ok(())
    }

    #[cfg(not(target_os = "windows"))]
    {
        std::fs::remove_file(path).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn test_normalize_verbatim_prefix_stripping() {
        let input = Path::new(r"\\?\C:\music\Coldplay - Yellow.flac");
        let normalized = normalize_windows_explorer_path(input);
        match normalized {
            Some(res) => assert_eq!(res, r"C:\music\Coldplay - Yellow.flac"),
            None => panic!("Should have Some normalized path"),
        }

        let input_unc = Path::new(r"\\?\UNC\server\share\Coldplay - Yellow.flac");
        let normalized_unc = normalize_windows_explorer_path(input_unc);
        match normalized_unc {
            Some(res) => assert_eq!(res, r"\\server\share\Coldplay - Yellow.flac"),
            None => panic!("Should have Some normalized UNC path"),
        }
    }

    #[test]
    fn test_normalize_forward_slash() {
        let input = Path::new("C:/music/artist/track.flac");
        let normalized = normalize_windows_explorer_path(input);
        match normalized {
            Some(res) => assert_eq!(res, r"C:\music\artist\track.flac"),
            None => panic!("Should have Some normalized path"),
        }

        let mixed = Path::new("C:/music\\artist/track.flac");
        let normalized_mixed = normalize_windows_explorer_path(mixed);
        match normalized_mixed {
            Some(res) => assert_eq!(res, r"C:\music\artist\track.flac"),
            None => panic!("Should have Some normalized path"),
        }
    }

    #[test]
    fn test_normalize_special_characters_and_spaces() {
        let input = Path::new("C:/music/deyluvkirby, eddyoetty, фрози/track.flac");
        let normalized = normalize_windows_explorer_path(input);
        match normalized {
            Some(res) => {
                assert_eq!(res, r"C:\music\deyluvkirby, eddyoetty, фрози\track.flac");
            }
            None => panic!("Should have Some normalized path"),
        }

        let input_jp = Path::new("C:/music/死人/01.flac");
        let normalized_jp = normalize_windows_explorer_path(input_jp);
        match normalized_jp {
            Some(res) => assert_eq!(res, r"C:\music\死人\01.flac"),
            None => panic!("Should have Some normalized path"),
        }
    }

    #[test]
    fn test_normalize_empty_path() {
        let input = Path::new("");
        let normalized = normalize_windows_explorer_path(input);
        assert!(normalized.is_none());
    }

    #[test]
    fn test_double_null_termination_logic() {
        let path_str = r"C:\music\Coldplay - Yellow.flac";
        let wide = to_double_null_terminated_wide(path_str);

        assert!(wide.len() >= 2);
        assert_eq!(wide[wide.len() - 1], 0);
        assert_eq!(wide[wide.len() - 2], 0);

        // Verify that the prefix decodes exactly back to the original string
        let content_units = &wide[..wide.len() - 2];
        let decoded = String::from_utf16_lossy(content_units);
        assert_eq!(decoded, path_str);
    }

    #[test]
    fn test_move_to_recycle_bin_non_existent() {
        let non_existent = Path::new(r"C:\non_existent_folder_abc_xyz_12345\missing_file.flac");
        let result = move_to_recycle_bin(non_existent);
        match result {
            Ok(()) => {}
            Err(e) => panic!("Should have Ok(()) for non-existent path, got Err: {}", e),
        }
    }

    #[test]
    fn test_reveal_in_file_explorer_non_existent_target() {
        let non_existent = Path::new(r"C:\non_existent_folder_abc_xyz_12345\missing_file.flac");
        // Must return cleanly without panicking or spawning invalid explorer process
        reveal_in_file_explorer(non_existent);
    }
}
