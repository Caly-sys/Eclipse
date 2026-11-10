# Releasing Eclipse

This document outlines the packaging and release verification procedure for Eclipse.

---

## 1. Release Build

Ensure `.cargo/config.toml` includes static CRT linking:
```toml
[target.x86_64-pc-windows-msvc]
rustflags = ["-C", "target-feature=+crt-static"]
```

Build the release binary:
```powershell
cargo build --release
```

---

## 2. Packaging the Standalone Archive

```powershell
# 1. Copy the optimized binary to the root directory
Copy-Item target\release\eclipse.exe eclipse.exe -Force

# 2. Package the portable archive
Compress-Archive -Path eclipse.exe, run.bat, README.md, LICENSE -DestinationPath eclipse-windows-x64.zip -Force

# 3. Generate SHA-256 checksums
Get-FileHash -Algorithm SHA256 eclipse.exe, eclipse-windows-x64.zip
```

---

## 3. GitHub Release Publishing

```powershell
gh release create v<version> eclipse.exe eclipse-windows-x64.zip --title "Eclipse v<version>" --notes-file release_notes.md
```
