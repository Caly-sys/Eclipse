# Building Eclipse from Source

This guide covers the tools, prerequisites, and instructions needed to build Eclipse on Windows.

---

## Prerequisites

| Requirement | Minimum Version | Installation / Command | Purpose |
|---|---|---|---|
| **Operating System** | Windows 10 (1809+) or Windows 11 (64-bit) | Pre-installed | Host compilation OS |
| **Rust Toolchain** | `1.80.0+` (`stable-x86_64-pc-windows-msvc`) | `winget install Rustlang.Rustup` or [rustup.rs](https://rustup.rs/) | Rust compiler (`rustc`) & package manager (`cargo`) |
| **Visual Studio C++ Build Tools** | VS 2019 or 2022 (MSVC v142/v143) | `winget install Microsoft.VisualStudio.2022.BuildTools` or [Visual Studio Downloads](https://visualstudio.microsoft.com/visual-cpp-build-tools/) | Provides C++ compiler (`cl.exe`), linker (`link.exe`), and libraries for compiling native C dependencies (`rusqlite`, `cpal`, `wry`) |
| **Windows 10 / 11 SDK** | 10.0.19041.0 or newer | Bundled with VS C++ Workload | Core Windows API headers and libraries (`kernel32`, `user32`, `dwmapi`, `ole32`) |
| **Git for Windows** | 2.40+ | `winget install Git.Git` or [git-scm.com](https://git-scm.com/) | Version control and repository cloning |
| **Python 3.8+** *(Optional)* | 3.8+ | `winget install Python.Python.3.12` | Required only if running the emoji audit verification script (`tests/verify_no_emojis.py`) |

---

## Installing C++ Build Tools (One-Line Setup)

If you do not have Visual Studio installed, you can install the required C++ build environment via `winget` in an Administrator terminal:

```powershell
winget install Microsoft.VisualStudio.2022.BuildTools --override "--passive --wait --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended"
```

Alternatively, open the **Visual Studio Installer** and ensure the following component is checked:
- **Desktop development with C++** (Workload)
  - **MSVC v143 - VS 2022 C++ x64/x86 build tools**
  - **Windows 11 SDK** or **Windows 10 SDK**

Then ensure your Rust toolchain uses the MSVC ABI:
```powershell
rustup default stable-x86_64-pc-windows-msvc
```

---

## Compilation

```powershell
# 1. Clone the repository
git clone https://github.com/Caly-sys/Eclipse.git
cd Eclipse

# 2. Fast development build
cargo build

# 3. Optimized release build
# (Statically links the C-runtime via .cargo/config.toml)
cargo build --release

# 4. Launch the executable
.\target\release\eclipse.exe
```

---

## Running Tests & Linters

```powershell
# Run the test suite
cargo test --all

# Run the Clippy linter
cargo clippy --all-targets -- -D warnings

# Optional: Run the emoji audit test (requires Python)
python tests/verify_no_emojis.py
```

---

## Troubleshooting

- **`error: linker 'link.exe' not found`**:
  Visual Studio C++ Build Tools or the Windows SDK are missing. Install the **Desktop development with C++** workload in Visual Studio Installer and restart your terminal.
- **`VCRUNTIME140.dll was not found`**:
  Ensure you build with `.cargo/config.toml` containing `rustflags = ["-C", "target-feature=+crt-static"]` (included in the repository). This embeds the C-runtime directly into `eclipse.exe`.
