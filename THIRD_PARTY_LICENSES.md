# Third-Party Software & Open-Source Licenses

Eclipse is developed using open-source Rust libraries. This document contains attribution and licensing details for all third-party software dependencies included in Eclipse.

---

## Direct Core Dependencies

| Crate | Version | License | Description | Link |
|---|---|---|---|---|
| **[base64](https://github.com/marshallpierce/rust-base64)** | `0.22.1` | `MIT OR Apache-2.0` | encodes and decodes base64 as bytes or utf8 | [Repository](https://github.com/marshallpierce/rust-base64) |
| **[cpal](https://github.com/rustaudio/cpal)** | `0.15.3` | `Apache-2.0` | Low-level cross-platform audio I/O library in pure Rust. | [Repository](https://github.com/rustaudio/cpal) |
| **[eframe](https://github.com/emilk/egui/tree/master/crates/eframe)** | `0.29.1` | `MIT OR Apache-2.0` | egui framework - write GUI apps that compiles to web and/... | [Repository](https://github.com/emilk/egui/tree/master/crates/eframe) |
| **[egui](https://github.com/emilk/egui)** | `0.29.1` | `MIT OR Apache-2.0` | An easy-to-use immediate mode GUI that runs on both web a... | [Repository](https://github.com/emilk/egui) |
| **[egui_extras](https://github.com/emilk/egui)** | `0.29.1` | `MIT OR Apache-2.0` | Extra functionality and widgets for the egui GUI library | [Repository](https://github.com/emilk/egui) |
| **[image](https://github.com/image-rs/image)** | `0.25.10` | `MIT OR Apache-2.0` | Imaging library. Provides basic image processing and enco... | [Repository](https://github.com/image-rs/image) |
| **[keyring](https://github.com/hwchen/keyring-rs.git)** | `3.6.3` | `MIT OR Apache-2.0` | Cross-platform library for managing passwords/credentials | [Repository](https://github.com/hwchen/keyring-rs.git) |
| **[lofty](https://github.com/Serial-ATA/lofty-rs)** | `0.21.1` | `MIT OR Apache-2.0` | Audio metadata library | [Repository](https://github.com/Serial-ATA/lofty-rs) |
| **[rand](https://github.com/rust-random/rand)** | `0.8.8` | `MIT OR Apache-2.0` | Random number generators and other randomness functionality. | [Repository](https://github.com/rust-random/rand) |
| **[rfd](https://github.com/PolyMeilex/rfd)** | `0.15.4` | `MIT` | Rusty File Dialog | [Repository](https://github.com/PolyMeilex/rfd) |
| **[rodio](https://github.com/RustAudio/rodio)** | `0.19.0` | `MIT OR Apache-2.0` | Audio playback library | [Repository](https://github.com/RustAudio/rodio) |
| **[rusqlite](https://github.com/rusqlite/rusqlite)** | `0.32.1` | `MIT` | Ergonomic wrapper for SQLite | [Repository](https://github.com/rusqlite/rusqlite) |
| **[rustfft](https://github.com/ejmahler/RustFFT)** | `6.4.1` | `MIT OR Apache-2.0` | High-performance FFT library written in pure Rust. | [Repository](https://github.com/ejmahler/RustFFT) |
| **[serde](https://github.com/serde-rs/serde)** | `1.0.229` | `MIT OR Apache-2.0` | A generic serialization/deserialization framework | [Repository](https://github.com/serde-rs/serde) |
| **[serde_json](https://github.com/serde-rs/json)** | `1.0.154` | `MIT OR Apache-2.0` | A JSON serialization file format | [Repository](https://github.com/serde-rs/json) |
| **[sha2](https://github.com/RustCrypto/hashes)** | `0.10.9` | `MIT OR Apache-2.0` | Pure Rust implementation of the SHA-2 hash function famil... | [Repository](https://github.com/RustCrypto/hashes) |
| **[symphonia](https://github.com/pdeljanov/Symphonia)** | `0.5.5` | `MPL-2.0` | Pure Rust media container and audio decoding library. | [Repository](https://github.com/pdeljanov/Symphonia) |
| **[tao](https://github.com/tauri-apps/tao)** | `0.30.8` | `Apache-2.0` | Cross-platform window manager library. | [Repository](https://github.com/tauri-apps/tao) |
| **[tray-icon](https://github.com/tauri-apps/tray-icon)** | `0.26.1` | `MIT OR Apache-2.0` | Create tray icons for desktop applications | [Repository](https://github.com/tauri-apps/tray-icon) |
| **[ureq](https://github.com/algesten/ureq)** | `2.12.1` | `MIT OR Apache-2.0` | Simple, safe HTTP client | [Repository](https://github.com/algesten/ureq) |
| **[url](https://github.com/servo/rust-url)** | `2.5.8` | `MIT OR Apache-2.0` | URL library for Rust, based on the WHATWG URL Standard | [Repository](https://github.com/servo/rust-url) |
| **[walkdir](https://github.com/BurntSushi/walkdir)** | `2.5.0` | `Unlicense/MIT` | Recursively walk a directory. | [Repository](https://github.com/BurntSushi/walkdir) |
| **[webbrowser](https://github.com/amodm/webbrowser-rs)** | `1.2.6` | `MIT OR Apache-2.0` | Open URLs in web browsers available on a platform | [Repository](https://github.com/amodm/webbrowser-rs) |
| **[windows-sys](https://github.com/microsoft/windows-rs)** | `0.52.0` | `MIT OR Apache-2.0` | Rust for Windows | [Repository](https://github.com/microsoft/windows-rs) |
| **[wry](https://github.com/tauri-apps/wry)** | `0.47.2` | `Apache-2.0 OR MIT` | Cross-platform WebView rendering library | [Repository](https://github.com/tauri-apps/wry) |

---

## License Distribution Summary

| License Expression | Number of Crates |
|---|---|
| `MIT OR Apache-2.0` | 266 |
| `MIT` | 155 |
| `Apache-2.0 OR MIT` | 39 |
| `MIT/Apache-2.0` | 26 |
| `MPL-2.0` | 26 |
| `Apache-2.0` | 18 |
| `Unicode-3.0` | 18 |
| `Zlib OR Apache-2.0 OR MIT` | 8 |
| `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | 8 |
| `BSD-3-Clause` | 6 |
| `MIT OR Apache-2.0 OR Zlib` | 6 |
| `Unlicense OR MIT` | 5 |
| `Apache-2.0/MIT` | 5 |
| `ISC` | 4 |
| `MIT / Apache-2.0` | 2 |
| `BSL-1.0` | 2 |
| `MIT OR Zlib OR Apache-2.0` | 2 |
| `BSD-3-Clause OR Apache-2.0` | 2 |
| `BSD-3-Clause OR MIT OR Apache-2.0` | 2 |
| `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | 2 |
| `Unlicense/MIT` | 2 |
| `Zlib` | 2 |
| `CDLA-Permissive-2.0` | 2 |
| `BSD-2-Clause OR Apache-2.0 OR MIT` | 2 |
| `0BSD OR MIT OR Apache-2.0` | 1 |
| `BSD-2-Clause` | 1 |
| `Apache-2.0 AND MIT` | 1 |
| `CC0-1.0 OR MIT-0 OR Apache-2.0` | 1 |
| `(Apache-2.0 OR MIT) AND BSD-3-Clause` | 1 |
| `(MIT OR Apache-2.0) AND OFL-1.1 AND LicenseRef-UFL-1.0` | 1 |
| `BSD-2-Clause OR MIT OR Apache-2.0` | 1 |
| `Apache-2.0 AND ISC` | 1 |
| `Apache-2.0 OR ISC OR MIT` | 1 |
| `Apache-2.0 WITH LLVM-exception` | 1 |
| `(MIT OR Apache-2.0) AND Unicode-3.0` | 1 |

---

## All Third-Party Dependencies (Alphabetical)

| Crate | Version | License | Repository / Source |
|---|---|---|---|
| `ab_glyph` | `0.2.32` | `Apache-2.0` | [https://github.com/alexheretic/ab-glyph](https://github.com/alexheretic/ab-glyph) |
| `ab_glyph_rasterizer` | `0.1.10` | `Apache-2.0` | [https://github.com/alexheretic/ab-glyph](https://github.com/alexheretic/ab-glyph) |
| `accesskit` | `0.16.3` | `MIT OR Apache-2.0` | [https://github.com/AccessKit/accesskit](https://github.com/AccessKit/accesskit) |
| `adler2` | `2.0.1` | `0BSD OR MIT OR Apache-2.0` | [https://github.com/oyvindln/adler2](https://github.com/oyvindln/adler2) |
| `ahash` | `0.8.12` | `MIT OR Apache-2.0` | [https://github.com/tkaitchuck/ahash](https://github.com/tkaitchuck/ahash) |
| `aho-corasick` | `1.1.5` | `Unlicense OR MIT` | [https://github.com/BurntSushi/aho-corasick](https://github.com/BurntSushi/aho-corasick) |
| `alsa` | `0.9.1` | `Apache-2.0/MIT` | [https://github.com/diwic/alsa-rs](https://github.com/diwic/alsa-rs) |
| `alsa-sys` | `0.3.1` | `MIT` | [https://github.com/diwic/alsa-sys](https://github.com/diwic/alsa-sys) |
| `android-activity` | `0.6.1` | `MIT OR Apache-2.0` | [https://github.com/rust-mobile/android-activity](https://github.com/rust-mobile/android-activity) |
| `android-properties` | `0.2.2` | `MIT` | [https://github.com/miklelappo/android-properties](https://github.com/miklelappo/android-properties) |
| `arboard` | `3.7.0` | `MIT OR Apache-2.0` | [https://github.com/1Password/arboard](https://github.com/1Password/arboard) |
| `arrayref` | `0.3.9` | `BSD-2-Clause` | [https://github.com/droundy/arrayref](https://github.com/droundy/arrayref) |
| `arrayvec` | `0.7.8` | `MIT OR Apache-2.0` | [https://github.com/bluss/arrayvec](https://github.com/bluss/arrayvec) |
| `as-raw-xcb-connection` | `1.0.1` | `MIT OR Apache-2.0` | [https://github.com/psychon/as-raw-xcb-connection](https://github.com/psychon/as-raw-xcb-connection) |
| `ashpd` | `0.11.1` | `MIT` | [https://github.com/bilelmoussaoui/ashpd](https://github.com/bilelmoussaoui/ashpd) |
| `async-broadcast` | `0.7.2` | `MIT OR Apache-2.0` | [https://github.com/smol-rs/async-broadcast](https://github.com/smol-rs/async-broadcast) |
| `async-channel` | `2.5.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-channel](https://github.com/smol-rs/async-channel) |
| `async-executor` | `1.14.1` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-executor](https://github.com/smol-rs/async-executor) |
| `async-fs` | `2.2.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-fs](https://github.com/smol-rs/async-fs) |
| `async-io` | `2.6.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-io](https://github.com/smol-rs/async-io) |
| `async-lock` | `3.5.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-lock](https://github.com/smol-rs/async-lock) |
| `async-net` | `2.0.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-net](https://github.com/smol-rs/async-net) |
| `async-process` | `2.5.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-process](https://github.com/smol-rs/async-process) |
| `async-recursion` | `1.2.0` | `MIT OR Apache-2.0` | [https://github.com/dcchut/async-recursion](https://github.com/dcchut/async-recursion) |
| `async-signal` | `0.2.14` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-signal](https://github.com/smol-rs/async-signal) |
| `async-task` | `4.7.1` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/async-task](https://github.com/smol-rs/async-task) |
| `async-trait` | `0.1.92` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/async-trait](https://github.com/dtolnay/async-trait) |
| `atk` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `atk-sys` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `atomic-waker` | `1.1.2` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/atomic-waker](https://github.com/smol-rs/atomic-waker) |
| `autocfg` | `1.5.1` | `Apache-2.0 OR MIT` | [https://github.com/cuviper/autocfg](https://github.com/cuviper/autocfg) |
| `base64` | `0.22.1` | `MIT OR Apache-2.0` | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `base64` | `0.21.7` | `MIT OR Apache-2.0` | [https://github.com/marshallpierce/rust-base64](https://github.com/marshallpierce/rust-base64) |
| `bindgen` | `0.72.1` | `BSD-3-Clause` | [https://github.com/rust-lang/rust-bindgen](https://github.com/rust-lang/rust-bindgen) |
| `bitflags` | `2.13.2` | `MIT OR Apache-2.0` | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `bitflags` | `1.3.2` | `MIT/Apache-2.0` | [https://github.com/bitflags/bitflags](https://github.com/bitflags/bitflags) |
| `block` | `0.1.6` | `MIT` | [http://github.com/SSheldon/rust-block](http://github.com/SSheldon/rust-block) |
| `block-buffer` | `0.10.4` | `MIT OR Apache-2.0` | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `block2` | `0.6.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `block2` | `0.5.1` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `blocking` | `1.7.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/blocking](https://github.com/smol-rs/blocking) |
| `bumpalo` | `3.20.3` | `MIT OR Apache-2.0` | [https://github.com/fitzgen/bumpalo](https://github.com/fitzgen/bumpalo) |
| `bytemuck` | `1.25.2` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| `bytemuck_derive` | `1.12.1` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/Lokathor/bytemuck](https://github.com/Lokathor/bytemuck) |
| `byteorder` | `1.5.0` | `Unlicense OR MIT` | [https://github.com/BurntSushi/byteorder](https://github.com/BurntSushi/byteorder) |
| `byteorder-lite` | `0.1.0` | `Unlicense OR MIT` | [https://github.com/image-rs/byteorder-lite](https://github.com/image-rs/byteorder-lite) |
| `bytes` | `1.12.1` | `MIT` | [https://github.com/tokio-rs/bytes](https://github.com/tokio-rs/bytes) |
| `cairo-rs` | `0.18.5` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `cairo-sys-rs` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `calloop` | `0.13.0` | `MIT` | [https://github.com/Smithay/calloop](https://github.com/Smithay/calloop) |
| `calloop` | `0.14.5` | `MIT` | [https://github.com/Smithay/calloop](https://github.com/Smithay/calloop) |
| `calloop-wayland-source` | `0.3.0` | `MIT` | [https://github.com/smithay/calloop-wayland-source](https://github.com/smithay/calloop-wayland-source) |
| `calloop-wayland-source` | `0.4.1` | `MIT` | [https://github.com/smithay/calloop-wayland-source](https://github.com/smithay/calloop-wayland-source) |
| `cc` | `1.11.1` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `cesu8` | `1.1.0` | `Apache-2.0/MIT` | [https://github.com/emk/cesu8-rs](https://github.com/emk/cesu8-rs) |
| `cexpr` | `0.6.0` | `Apache-2.0/MIT` | [https://github.com/jethrogb/rust-cexpr](https://github.com/jethrogb/rust-cexpr) |
| `cfg-expr` | `0.15.8` | `MIT OR Apache-2.0` | [https://github.com/EmbarkStudios/cfg-expr](https://github.com/EmbarkStudios/cfg-expr) |
| `cfg-if` | `1.0.5` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/cfg-if](https://github.com/rust-lang/cfg-if) |
| `cfg_aliases` | `0.2.2` | `MIT` | [https://github.com/katharostech/cfg_aliases](https://github.com/katharostech/cfg_aliases) |
| `cgl` | `0.3.2` | `MIT / Apache-2.0` | [https://github.com/servo/cgl-rs](https://github.com/servo/cgl-rs) |
| `clang-sys` | `1.9.1` | `Apache-2.0` | [https://github.com/KyleMayes/clang-sys](https://github.com/KyleMayes/clang-sys) |
| `claxon` | `0.4.3` | `Apache-2.0` | [https://github.com/ruuda/claxon](https://github.com/ruuda/claxon) |
| `clipboard-win` | `5.4.3` | `BSL-1.0` | [https://github.com/DoumanAsh/clipboard-win](https://github.com/DoumanAsh/clipboard-win) |
| `cocoa` | `0.26.1` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `cocoa-foundation` | `0.2.1` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `color_quant` | `1.1.0` | `MIT` | [https://github.com/image-rs/color_quant.git](https://github.com/image-rs/color_quant.git) |
| `combine` | `4.6.9` | `MIT` | [https://github.com/Marwes/combine](https://github.com/Marwes/combine) |
| `concurrent-queue` | `2.5.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/concurrent-queue](https://github.com/smol-rs/concurrent-queue) |
| `convert_case` | `0.4.0` | `MIT` | [https://github.com/rutrum/convert-case](https://github.com/rutrum/convert-case) |
| `cookie` | `0.18.2` | `MIT OR Apache-2.0` | [https://github.com/SergioBenitez/cookie-rs](https://github.com/SergioBenitez/cookie-rs) |
| `core-foundation` | `0.9.4` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-foundation` | `0.10.1` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-foundation-sys` | `0.8.7` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics` | `0.23.2` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics` | `0.24.0` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | `0.1.3` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core-graphics-types` | `0.2.0` | `MIT OR Apache-2.0` | [https://github.com/servo/core-foundation-rs](https://github.com/servo/core-foundation-rs) |
| `core_detect` | `1.0.0` | `MIT/Apache-2.0` | [https://github.com/thomcc/core_detect](https://github.com/thomcc/core_detect) |
| `coreaudio-rs` | `0.11.3` | `MIT/Apache-2.0` | [https://github.com/RustAudio/coreaudio-rs.git](https://github.com/RustAudio/coreaudio-rs.git) |
| `coreaudio-sys` | `0.2.18` | `MIT` | [https://github.com/RustAudio/coreaudio-sys.git](https://github.com/RustAudio/coreaudio-sys.git) |
| `cpal` | `0.15.3` | `Apache-2.0` | [https://github.com/rustaudio/cpal](https://github.com/rustaudio/cpal) |
| `cpufeatures` | `0.2.17` | `MIT OR Apache-2.0` | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `crc32fast` | `1.5.4` | `MIT OR Apache-2.0` | [https://github.com/srijs/rust-crc32fast](https://github.com/srijs/rust-crc32fast) |
| `crossbeam-channel` | `0.5.17` | `MIT OR Apache-2.0` | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| `crossbeam-utils` | `0.8.23` | `MIT OR Apache-2.0` | [https://github.com/crossbeam-rs/crossbeam](https://github.com/crossbeam-rs/crossbeam) |
| `crypto-common` | `0.1.7` | `MIT OR Apache-2.0` | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `cssparser` | `0.27.2` | `MPL-2.0` | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| `cssparser-macros` | `0.6.1` | `MPL-2.0` | [https://github.com/servo/rust-cssparser](https://github.com/servo/rust-cssparser) |
| `cursor-icon` | `1.2.0` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/rust-windowing/cursor-icon](https://github.com/rust-windowing/cursor-icon) |
| `dasp_sample` | `0.11.0` | `MIT OR Apache-2.0` | [https://github.com/rustaudio/sample.git](https://github.com/rustaudio/sample.git) |
| `data-encoding` | `2.11.1` | `MIT` | [https://github.com/ia0/data-encoding](https://github.com/ia0/data-encoding) |
| `data-url` | `0.3.2` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `deranged` | `0.5.9` | `MIT OR Apache-2.0` | [https://github.com/jhpratt/deranged](https://github.com/jhpratt/deranged) |
| `derive_more` | `0.99.20` | `MIT` | [https://github.com/JelteF/derive_more](https://github.com/JelteF/derive_more) |
| `digest` | `0.10.7` | `MIT OR Apache-2.0` | [https://github.com/RustCrypto/traits](https://github.com/RustCrypto/traits) |
| `dirs` | `7.0.0` | `MIT OR Apache-2.0` | [https://codeberg.org/dirs/dirs-rs](https://codeberg.org/dirs/dirs-rs) |
| `dirs-sys` | `0.5.0` | `MIT OR Apache-2.0` | [https://github.com/dirs-dev/dirs-sys-rs](https://github.com/dirs-dev/dirs-sys-rs) |
| `dispatch` | `0.2.0` | `MIT` | [http://github.com/SSheldon/rust-dispatch](http://github.com/SSheldon/rust-dispatch) |
| `dispatch2` | `0.3.1` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `displaydoc` | `0.2.7` | `MIT OR Apache-2.0` | [https://github.com/yaahc/displaydoc](https://github.com/yaahc/displaydoc) |
| `dlib` | `0.5.3` | `MIT` | [https://github.com/elinorbgr/dlib](https://github.com/elinorbgr/dlib) |
| `dlopen2` | `0.7.0` | `MIT` | [https://github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| `dlopen2_derive` | `0.4.3` | `MIT` | [https://github.com/OpenByteDev/dlopen2](https://github.com/OpenByteDev/dlopen2) |
| `document-features` | `0.2.12` | `MIT OR Apache-2.0` | [https://github.com/slint-ui/document-features](https://github.com/slint-ui/document-features) |
| `downcast-rs` | `1.2.1` | `MIT/Apache-2.0` | [https://github.com/marcianx/downcast-rs](https://github.com/marcianx/downcast-rs) |
| `dpi` | `0.1.2` | `Apache-2.0 AND MIT` | [https://github.com/rust-windowing/winit](https://github.com/rust-windowing/winit) |
| `dtoa` | `1.0.11` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/dtoa](https://github.com/dtolnay/dtoa) |
| `dtoa-short` | `0.3.5` | `MPL-2.0` | [https://github.com/upsuper/dtoa-short](https://github.com/upsuper/dtoa-short) |
| `dunce` | `1.0.5` | `CC0-1.0 OR MIT-0 OR Apache-2.0` | [https://gitlab.com/kornelski/dunce](https://gitlab.com/kornelski/dunce) |
| `ecolor` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui](https://github.com/emilk/egui) |
| `eframe` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui/tree/master/crates/eframe](https://github.com/emilk/egui/tree/master/crates/eframe) |
| `egui` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui](https://github.com/emilk/egui) |
| `egui-winit` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui/tree/master/crates/egui-winit](https://github.com/emilk/egui/tree/master/crates/egui-winit) |
| `egui_extras` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui](https://github.com/emilk/egui) |
| `egui_glow` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui/tree/master/crates/egui_glow](https://github.com/emilk/egui/tree/master/crates/egui_glow) |
| `ehttp` | `0.5.0` | `MIT OR Apache-2.0` | [https://github.com/emilk/ehttp](https://github.com/emilk/ehttp) |
| `either` | `1.19.0` | `MIT OR Apache-2.0` | [https://github.com/rayon-rs/either](https://github.com/rayon-rs/either) |
| `emath` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui/tree/master/crates/emath](https://github.com/emilk/egui/tree/master/crates/emath) |
| `encoding_rs` | `0.8.44` | `(Apache-2.0 OR MIT) AND BSD-3-Clause` | [https://github.com/hsivonen/encoding_rs](https://github.com/hsivonen/encoding_rs) |
| `endi` | `1.1.1` | `MIT` | [https://github.com/zeenix/endi](https://github.com/zeenix/endi) |
| `enum-map` | `2.7.3` | `MIT OR Apache-2.0` | [https://codeberg.org/xfix/enum-map](https://codeberg.org/xfix/enum-map) |
| `enum-map-derive` | `0.17.0` | `MIT OR Apache-2.0` | [https://codeberg.org/xfix/enum-map](https://codeberg.org/xfix/enum-map) |
| `enumflags2` | `0.7.12` | `MIT OR Apache-2.0` | [https://github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| `enumflags2_derive` | `0.7.12` | `MIT OR Apache-2.0` | [https://github.com/meithecatte/enumflags2](https://github.com/meithecatte/enumflags2) |
| `enumn` | `0.1.14` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/enumn](https://github.com/dtolnay/enumn) |
| `epaint` | `0.29.1` | `MIT OR Apache-2.0` | [https://github.com/emilk/egui/tree/master/crates/epaint](https://github.com/emilk/egui/tree/master/crates/epaint) |
| `epaint_default_fonts` | `0.29.1` | `(MIT OR Apache-2.0) AND OFL-1.1 AND LicenseRef-UFL-1.0` | [https://github.com/emilk/egui/tree/master/crates/epaint_default_fonts](https://github.com/emilk/egui/tree/master/crates/epaint_default_fonts) |
| `equivalent` | `1.0.3` | `Apache-2.0 OR MIT` | [https://github.com/indexmap-rs/equivalent](https://github.com/indexmap-rs/equivalent) |
| `errno` | `0.3.14` | `MIT OR Apache-2.0` | [https://github.com/lambda-fairy/rust-errno](https://github.com/lambda-fairy/rust-errno) |
| `error-code` | `3.4.0` | `BSL-1.0` | [https://github.com/DoumanAsh/error-code](https://github.com/DoumanAsh/error-code) |
| `event-listener` | `5.4.2` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/event-listener](https://github.com/smol-rs/event-listener) |
| `event-listener-strategy` | `0.5.4` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/event-listener-strategy](https://github.com/smol-rs/event-listener-strategy) |
| `extended` | `0.1.0` | `MIT` | [https://github.com/depp/extended-rs](https://github.com/depp/extended-rs) |
| `fallible-iterator` | `0.3.0` | `MIT/Apache-2.0` | [https://github.com/sfackler/rust-fallible-iterator](https://github.com/sfackler/rust-fallible-iterator) |
| `fallible-streaming-iterator` | `0.1.9` | `MIT/Apache-2.0` | [https://github.com/sfackler/fallible-streaming-iterator](https://github.com/sfackler/fallible-streaming-iterator) |
| `fastrand` | `2.5.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/fastrand](https://github.com/smol-rs/fastrand) |
| `fdeflate` | `0.3.7` | `MIT OR Apache-2.0` | [https://github.com/image-rs/fdeflate](https://github.com/image-rs/fdeflate) |
| `field-offset` | `0.3.6` | `MIT OR Apache-2.0` | [https://github.com/Diggsey/rust-field-offset](https://github.com/Diggsey/rust-field-offset) |
| `find-msvc-tools` | `0.1.15` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/cc-rs](https://github.com/rust-lang/cc-rs) |
| `flate2` | `1.1.11` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/flate2-rs](https://github.com/rust-lang/flate2-rs) |
| `float-cmp` | `0.9.0` | `MIT` | [https://github.com/mikedilger/float-cmp](https://github.com/mikedilger/float-cmp) |
| `foreign-types` | `0.5.0` | `MIT/Apache-2.0` | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `foreign-types-macros` | `0.2.4` | `MIT/Apache-2.0` | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `foreign-types-shared` | `0.3.1` | `MIT/Apache-2.0` | [https://github.com/sfackler/foreign-types](https://github.com/sfackler/foreign-types) |
| `form_urlencoded` | `1.2.2` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `futf` | `0.1.5` | `MIT / Apache-2.0` | [https://github.com/servo/futf](https://github.com/servo/futf) |
| `futures-channel` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-core` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-executor` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-io` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-lite` | `2.6.1` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/futures-lite](https://github.com/smol-rs/futures-lite) |
| `futures-macro` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-task` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `futures-util` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/futures-rs](https://github.com/rust-lang/futures-rs) |
| `fxhash` | `0.2.1` | `Apache-2.0/MIT` | [https://github.com/cbreeden/fxhash](https://github.com/cbreeden/fxhash) |
| `gdk` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdk-pixbuf` | `0.18.5` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-pixbuf-sys` | `0.18.0` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gdk-sys` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkwayland-sys` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gdkx11-sys` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `generic-array` | `0.14.7` | `MIT` | [https://github.com/fizyk20/generic-array.git](https://github.com/fizyk20/generic-array.git) |
| `gethostname` | `1.1.0` | `Apache-2.0` | [https://codeberg.org/swsnr/gethostname.rs.git](https://codeberg.org/swsnr/gethostname.rs.git) |
| `getrandom` | `0.2.17` | `MIT OR Apache-2.0` | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | `0.3.4` | `MIT OR Apache-2.0` | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | `0.4.3` | `MIT OR Apache-2.0` | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `getrandom` | `0.1.16` | `MIT OR Apache-2.0` | [https://github.com/rust-random/getrandom](https://github.com/rust-random/getrandom) |
| `gif` | `0.14.2` | `MIT OR Apache-2.0` | [https://github.com/image-rs/image-gif](https://github.com/image-rs/image-gif) |
| `gio` | `0.18.4` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gio-sys` | `0.18.1` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gl_generator` | `0.14.0` | `Apache-2.0` | [https://github.com/brendanzab/gl-rs/](https://github.com/brendanzab/gl-rs/) |
| `glib` | `0.18.5` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-macros` | `0.18.5` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glib-sys` | `0.18.1` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `glob` | `0.3.4` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/glob](https://github.com/rust-lang/glob) |
| `glow` | `0.14.2` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/grovesNL/glow](https://github.com/grovesNL/glow) |
| `glutin` | `0.32.3` | `Apache-2.0` | [https://github.com/rust-windowing/glutin](https://github.com/rust-windowing/glutin) |
| `glutin-winit` | `0.5.0` | `MIT` | [https://github.com/rust-windowing/glutin](https://github.com/rust-windowing/glutin) |
| `glutin_egl_sys` | `0.7.1` | `Apache-2.0` | [https://github.com/rust-windowing/glutin](https://github.com/rust-windowing/glutin) |
| `glutin_glx_sys` | `0.6.1` | `Apache-2.0` | [https://github.com/rust-windowing/glutin](https://github.com/rust-windowing/glutin) |
| `glutin_wgl_sys` | `0.6.1` | `Apache-2.0` | [https://github.com/rust-windowing/glutin](https://github.com/rust-windowing/glutin) |
| `gobject-sys` | `0.18.0` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `gtk` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gtk-sys` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `gtk3-macros` | `0.18.2` | `MIT` | [https://github.com/gtk-rs/gtk3-rs](https://github.com/gtk-rs/gtk3-rs) |
| `hashbrown` | `0.17.1` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | `0.14.5` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashbrown` | `0.12.3` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/hashbrown](https://github.com/rust-lang/hashbrown) |
| `hashlink` | `0.9.1` | `MIT OR Apache-2.0` | [https://github.com/kyren/hashlink](https://github.com/kyren/hashlink) |
| `heck` | `0.5.0` | `MIT OR Apache-2.0` | [https://github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| `heck` | `0.4.1` | `MIT OR Apache-2.0` | [https://github.com/withoutboats/heck](https://github.com/withoutboats/heck) |
| `hermit-abi` | `0.5.3` | `MIT OR Apache-2.0` | [https://github.com/hermit-os/hermit-rs](https://github.com/hermit-os/hermit-rs) |
| `hex` | `0.4.3` | `MIT OR Apache-2.0` | [https://github.com/KokaKiwi/rust-hex](https://github.com/KokaKiwi/rust-hex) |
| `home` | `0.5.12` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/cargo](https://github.com/rust-lang/cargo) |
| `hound` | `3.5.1` | `Apache-2.0` | [https://github.com/ruuda/hound](https://github.com/ruuda/hound) |
| `html5ever` | `0.26.0` | `MIT OR Apache-2.0` | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `http` | `1.5.0` | `MIT OR Apache-2.0` | [https://github.com/hyperium/http](https://github.com/hyperium/http) |
| `icu_collections` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_locale_core` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_normalizer_data` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_properties_data` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `icu_provider` | `2.4.0` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `idna` | `1.1.0` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| `idna_adapter` | `1.2.2` | `Apache-2.0 OR MIT` | [https://github.com/hsivonen/idna_adapter](https://github.com/hsivonen/idna_adapter) |
| `image` | `0.25.10` | `MIT OR Apache-2.0` | [https://github.com/image-rs/image](https://github.com/image-rs/image) |
| `imagesize` | `0.12.0` | `MIT` | [https://github.com/Roughsketch/imagesize](https://github.com/Roughsketch/imagesize) |
| `indexmap` | `1.9.3` | `Apache-2.0 OR MIT` | [https://github.com/bluss/indexmap](https://github.com/bluss/indexmap) |
| `indexmap` | `2.14.2` | `Apache-2.0 OR MIT` | [https://github.com/indexmap-rs/indexmap](https://github.com/indexmap-rs/indexmap) |
| `instant` | `0.1.13` | `BSD-3-Clause` | [https://github.com/sebcrozet/instant](https://github.com/sebcrozet/instant) |
| `itertools` | `0.13.0` | `MIT OR Apache-2.0` | [https://github.com/rust-itertools/itertools](https://github.com/rust-itertools/itertools) |
| `itoa` | `1.0.18` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| `itoa` | `0.4.8` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/itoa](https://github.com/dtolnay/itoa) |
| `javascriptcore-rs` | `1.1.2` | `MIT` | [https://github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| `javascriptcore-rs-sys` | `1.1.1` | `MIT` | [https://github.com/tauri-apps/javascriptcore-rs](https://github.com/tauri-apps/javascriptcore-rs) |
| `jni` | `0.21.1` | `MIT/Apache-2.0` | [https://github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| `jni` | `0.22.4` | `MIT OR Apache-2.0` | [https://github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| `jni-macros` | `0.22.4` | `MIT OR Apache-2.0` | [https://github.com/jni-rs/jni-rs](https://github.com/jni-rs/jni-rs) |
| `jni-sys` | `0.3.1` | `MIT OR Apache-2.0` | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `jni-sys` | `0.4.1` | `MIT OR Apache-2.0` | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `jni-sys-macros` | `0.4.1` | `MIT OR Apache-2.0` | [https://github.com/jni-rs/jni-sys](https://github.com/jni-rs/jni-sys) |
| `jobserver` | `0.1.35` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/jobserver-rs](https://github.com/rust-lang/jobserver-rs) |
| `js-sys` | `0.3.107` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/js-sys) |
| `keyboard-types` | `0.8.3` | `MIT OR Apache-2.0` | [https://github.com/rust-windowing/keyboard-types](https://github.com/rust-windowing/keyboard-types) |
| `keyring` | `3.6.3` | `MIT OR Apache-2.0` | [https://github.com/hwchen/keyring-rs.git](https://github.com/hwchen/keyring-rs.git) |
| `khronos_api` | `3.1.0` | `Apache-2.0` | [https://github.com/brendanzab/gl-rs/](https://github.com/brendanzab/gl-rs/) |
| `kuchikiki` | `0.8.2` | `MIT` | [https://github.com/brave/kuchikiki](https://github.com/brave/kuchikiki) |
| `kurbo` | `0.9.5` | `MIT/Apache-2.0` | [https://github.com/linebender/kurbo](https://github.com/linebender/kurbo) |
| `lazy_static` | `1.5.1` | `MIT OR Apache-2.0` | [https://github.com/rust-lang-nursery/lazy-static.rs](https://github.com/rust-lang-nursery/lazy-static.rs) |
| `lewton` | `0.10.2` | `MIT OR Apache-2.0` | [https://github.com/RustAudio/lewton](https://github.com/RustAudio/lewton) |
| `libappindicator` | `0.9.0` | `Apache-2.0 OR MIT` | [https://crates.io/crates/libappindicator](https://crates.io/crates/libappindicator) |
| `libappindicator-sys` | `0.9.0` | `Apache-2.0 OR MIT` | [https://crates.io/crates/libappindicator-sys](https://crates.io/crates/libappindicator-sys) |
| `libc` | `0.2.191` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/libc](https://github.com/rust-lang/libc) |
| `libloading` | `0.7.4` | `ISC` | [https://github.com/nagisa/rust_libloading/](https://github.com/nagisa/rust_libloading/) |
| `libloading` | `0.8.9` | `ISC` | [https://github.com/nagisa/rust_libloading/](https://github.com/nagisa/rust_libloading/) |
| `libredox` | `0.1.25` | `MIT` | [https://gitlab.redox-os.org/redox-os/libredox.git](https://gitlab.redox-os.org/redox-os/libredox.git) |
| `libsqlite3-sys` | `0.30.1` | `MIT` | [https://github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite) |
| `linux-raw-sys` | `0.4.15` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| `linux-raw-sys` | `0.12.1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/sunfishcode/linux-raw-sys](https://github.com/sunfishcode/linux-raw-sys) |
| `litemap` | `0.8.3` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `litrs` | `1.0.1` | `MIT OR Apache-2.0` | [https://github.com/LukasKalbertodt/litrs](https://github.com/LukasKalbertodt/litrs) |
| `lock_api` | `0.4.14` | `MIT OR Apache-2.0` | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `lofty` | `0.21.1` | `MIT OR Apache-2.0` | [https://github.com/Serial-ATA/lofty-rs](https://github.com/Serial-ATA/lofty-rs) |
| `lofty_attr` | `0.11.1` | `MIT OR Apache-2.0` | [https://github.com/Serial-ATA/lofty-rs](https://github.com/Serial-ATA/lofty-rs) |
| `log` | `0.4.35` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/log](https://github.com/rust-lang/log) |
| `mac` | `0.1.1` | `MIT/Apache-2.0` | [https://github.com/reem/rust-mac.git](https://github.com/reem/rust-mac.git) |
| `mach2` | `0.4.3` | `BSD-2-Clause OR MIT OR Apache-2.0` | [https://github.com/JohnTitor/mach2](https://github.com/JohnTitor/mach2) |
| `malloc_buf` | `0.0.6` | `MIT` | [https://github.com/SSheldon/malloc_buf](https://github.com/SSheldon/malloc_buf) |
| `markup5ever` | `0.11.0` | `MIT OR Apache-2.0` | [https://github.com/servo/html5ever](https://github.com/servo/html5ever) |
| `matches` | `0.1.10` | `MIT` | [https://github.com/SimonSapin/rust-std-candidates](https://github.com/SimonSapin/rust-std-candidates) |
| `memchr` | `2.8.3` | `Unlicense OR MIT` | [https://github.com/BurntSushi/memchr](https://github.com/BurntSushi/memchr) |
| `memmap2` | `0.9.12` | `MIT OR Apache-2.0` | [https://github.com/RazrFalcon/memmap2-rs](https://github.com/RazrFalcon/memmap2-rs) |
| `memoffset` | `0.9.1` | `MIT` | [https://github.com/Gilnaa/memoffset](https://github.com/Gilnaa/memoffset) |
| `mime` | `0.3.18` | `MIT OR Apache-2.0` | [https://github.com/hyperium/mime](https://github.com/hyperium/mime) |
| `mime_guess2` | `2.3.1` | `MIT` | [https://github.com/ttys3/mime_guess2](https://github.com/ttys3/mime_guess2) |
| `minimal-lexical` | `0.2.1` | `MIT/Apache-2.0` | [https://github.com/Alexhuszagh/minimal-lexical](https://github.com/Alexhuszagh/minimal-lexical) |
| `miniz_oxide` | `0.8.9` | `MIT OR Zlib OR Apache-2.0` | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `miniz_oxide` | `0.9.1` | `MIT OR Zlib OR Apache-2.0` | [https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide](https://github.com/Frommi/miniz_oxide/tree/master/miniz_oxide) |
| `moxcms` | `0.8.1` | `BSD-3-Clause OR Apache-2.0` | [https://github.com/awxkee/moxcms.git](https://github.com/awxkee/moxcms.git) |
| `muda` | `0.21.2` | `Apache-2.0 OR MIT` | [https://github.com/tauri-apps/muda](https://github.com/tauri-apps/muda) |
| `multiversion_no_op` | `1.0.0` | `Apache-2.0 OR MIT` | [https://github.com/hsivonen/multiversion_no_op](https://github.com/hsivonen/multiversion_no_op) |
| `ndk` | `0.8.0` | `MIT OR Apache-2.0` | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `ndk` | `0.9.0` | `MIT OR Apache-2.0` | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `ndk-context` | `0.1.1` | `MIT OR Apache-2.0` | [https://github.com/rust-windowing/android-ndk-rs](https://github.com/rust-windowing/android-ndk-rs) |
| `ndk-sys` | `0.6.0+11769913` | `MIT OR Apache-2.0` | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `ndk-sys` | `0.5.0+25.2.9519653` | `MIT OR Apache-2.0` | [https://github.com/rust-mobile/ndk](https://github.com/rust-mobile/ndk) |
| `new_debug_unreachable` | `1.0.6` | `MIT` | [https://github.com/mbrubeck/rust-debug-unreachable](https://github.com/mbrubeck/rust-debug-unreachable) |
| `nodrop` | `0.1.14` | `MIT/Apache-2.0` | [https://github.com/bluss/arrayvec](https://github.com/bluss/arrayvec) |
| `nohash-hasher` | `0.2.0` | `Apache-2.0 OR MIT` | [https://github.com/paritytech/nohash-hasher](https://github.com/paritytech/nohash-hasher) |
| `nom` | `7.1.3` | `MIT` | [https://github.com/Geal/nom](https://github.com/Geal/nom) |
| `num-complex` | `0.4.6` | `MIT OR Apache-2.0` | [https://github.com/rust-num/num-complex](https://github.com/rust-num/num-complex) |
| `num-conv` | `0.2.2` | `MIT OR Apache-2.0` | [https://github.com/jhpratt/num-conv](https://github.com/jhpratt/num-conv) |
| `num-derive` | `0.4.2` | `MIT OR Apache-2.0` | [https://github.com/rust-num/num-derive](https://github.com/rust-num/num-derive) |
| `num-integer` | `0.1.47` | `MIT OR Apache-2.0` | [https://github.com/rust-num/num-integer](https://github.com/rust-num/num-integer) |
| `num-traits` | `0.2.19` | `MIT OR Apache-2.0` | [https://github.com/rust-num/num-traits](https://github.com/rust-num/num-traits) |
| `num_enum` | `0.7.6` | `BSD-3-Clause OR MIT OR Apache-2.0` | [https://github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |
| `num_enum_derive` | `0.7.6` | `BSD-3-Clause OR MIT OR Apache-2.0` | [https://github.com/illicitonion/num_enum](https://github.com/illicitonion/num_enum) |
| `objc` | `0.2.7` | `MIT` | [http://github.com/SSheldon/rust-objc](http://github.com/SSheldon/rust-objc) |
| `objc-sys` | `0.3.5` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2` | `0.5.3` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2` | `0.6.5` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-app-kit` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-app-kit` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-cloud-kit` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-contacts` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-data` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-foundation` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-graphics` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-image` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-core-location` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-encode` | `4.1.0` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-foundation` | `0.3.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-foundation` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-io-surface` | `0.3.2` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-link-presentation` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-metal` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-quartz-core` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-symbols` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-ui-kit` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-uniform-type-identifiers` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-user-notifications` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `objc2-web-kit` | `0.2.2` | `MIT` | [https://github.com/madsmtm/objc2](https://github.com/madsmtm/objc2) |
| `oboe` | `0.6.1` | `Apache-2.0` | [https://github.com/katyo/oboe-rs](https://github.com/katyo/oboe-rs) |
| `oboe-sys` | `0.6.1` | `Apache-2.0` | [https://github.com/katyo/oboe-rs](https://github.com/katyo/oboe-rs) |
| `ogg` | `0.8.0` | `BSD-3-Clause` | [https://github.com/RustAudio/ogg](https://github.com/RustAudio/ogg) |
| `ogg_pager` | `0.6.1` | `MIT OR Apache-2.0` | [https://github.com/Serial-ATA/lofty-rs](https://github.com/Serial-ATA/lofty-rs) |
| `once_cell` | `1.21.4` | `MIT OR Apache-2.0` | [https://github.com/matklad/once_cell](https://github.com/matklad/once_cell) |
| `option-ext` | `0.2.0` | `MPL-2.0` | [https://github.com/soc/option-ext.git](https://github.com/soc/option-ext.git) |
| `orbclient` | `0.3.55` | `MIT` | [https://gitlab.redox-os.org/redox-os/orbclient](https://gitlab.redox-os.org/redox-os/orbclient) |
| `ordered-stream` | `0.2.0` | `MIT OR Apache-2.0` | [https://github.com/danieldg/ordered-stream](https://github.com/danieldg/ordered-stream) |
| `owned_ttf_parser` | `0.25.1` | `Apache-2.0` | [https://github.com/alexheretic/owned-ttf-parser](https://github.com/alexheretic/owned-ttf-parser) |
| `pango` | `0.18.3` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `pango-sys` | `0.18.0` | `MIT` | [https://github.com/gtk-rs/gtk-rs-core](https://github.com/gtk-rs/gtk-rs-core) |
| `parking` | `2.2.1` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/parking](https://github.com/smol-rs/parking) |
| `parking_lot` | `0.12.5` | `MIT OR Apache-2.0` | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `parking_lot_core` | `0.9.12` | `MIT OR Apache-2.0` | [https://github.com/Amanieu/parking_lot](https://github.com/Amanieu/parking_lot) |
| `paste` | `1.0.15` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/paste](https://github.com/dtolnay/paste) |
| `percent-encoding` | `2.3.2` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-url/](https://github.com/servo/rust-url/) |
| `phf` | `0.11.3` | `MIT` | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf` | `0.8.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf` | `0.10.1` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_codegen` | `0.10.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_codegen` | `0.8.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_generator` | `0.10.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_generator` | `0.11.3` | `MIT` | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf_generator` | `0.8.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_macros` | `0.11.3` | `MIT` | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `phf_macros` | `0.8.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_shared` | `0.8.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_shared` | `0.10.0` | `MIT` | [https://github.com/sfackler/rust-phf](https://github.com/sfackler/rust-phf) |
| `phf_shared` | `0.11.3` | `MIT` | [https://github.com/rust-phf/rust-phf](https://github.com/rust-phf/rust-phf) |
| `pico-args` | `0.5.0` | `MIT` | [https://github.com/RazrFalcon/pico-args](https://github.com/RazrFalcon/pico-args) |
| `pin-project` | `1.1.13` | `Apache-2.0 OR MIT` | [https://github.com/taiki-e/pin-project](https://github.com/taiki-e/pin-project) |
| `pin-project-internal` | `1.1.13` | `Apache-2.0 OR MIT` | [https://github.com/taiki-e/pin-project](https://github.com/taiki-e/pin-project) |
| `pin-project-lite` | `0.2.17` | `Apache-2.0 OR MIT` | [https://github.com/taiki-e/pin-project-lite](https://github.com/taiki-e/pin-project-lite) |
| `piper` | `0.2.5` | `MIT OR Apache-2.0` | [https://github.com/smol-rs/piper](https://github.com/smol-rs/piper) |
| `pkg-config` | `0.3.34` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/pkg-config-rs](https://github.com/rust-lang/pkg-config-rs) |
| `plain` | `0.2.3` | `MIT/Apache-2.0` | [https://github.com/randomites/plain](https://github.com/randomites/plain) |
| `png` | `0.18.1` | `MIT OR Apache-2.0` | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| `png` | `0.17.16` | `MIT OR Apache-2.0` | [https://github.com/image-rs/image-png](https://github.com/image-rs/image-png) |
| `polling` | `3.11.0` | `Apache-2.0 OR MIT` | [https://github.com/smol-rs/polling](https://github.com/smol-rs/polling) |
| `pollster` | `0.4.0` | `Apache-2.0/MIT` | [https://github.com/zesterer/pollster](https://github.com/zesterer/pollster) |
| `potential_utf` | `0.1.6` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `powerfmt` | `0.2.1` | `MIT OR Apache-2.0` | [https://github.com/jhpratt/powerfmt](https://github.com/jhpratt/powerfmt) |
| `ppv-lite86` | `0.2.21` | `MIT OR Apache-2.0` | [https://github.com/cryptocorrosion/cryptocorrosion](https://github.com/cryptocorrosion/cryptocorrosion) |
| `precomputed-hash` | `0.1.1` | `MIT` | [https://github.com/emilio/precomputed-hash](https://github.com/emilio/precomputed-hash) |
| `primal-check` | `0.3.4` | `MIT OR Apache-2.0` | [https://github.com/huonw/primal](https://github.com/huonw/primal) |
| `proc-macro-crate` | `1.3.1` | `MIT OR Apache-2.0` | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | `2.0.2` | `MIT OR Apache-2.0` | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-crate` | `3.5.0` | `MIT OR Apache-2.0` | [https://github.com/bkchr/proc-macro-crate](https://github.com/bkchr/proc-macro-crate) |
| `proc-macro-error` | `1.0.4` | `MIT OR Apache-2.0` | [https://gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro-error-attr` | `1.0.4` | `MIT OR Apache-2.0` | [https://gitlab.com/CreepySkeleton/proc-macro-error](https://gitlab.com/CreepySkeleton/proc-macro-error) |
| `proc-macro-hack` | `0.5.20+deprecated` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/proc-macro-hack](https://github.com/dtolnay/proc-macro-hack) |
| `proc-macro2` | `1.0.107` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/proc-macro2](https://github.com/dtolnay/proc-macro2) |
| `pxfm` | `0.1.30` | `BSD-3-Clause OR Apache-2.0` | [https://github.com/awxkee/pxfm](https://github.com/awxkee/pxfm) |
| `quick-xml` | `0.41.0` | `MIT` | [https://github.com/tafia/quick-xml](https://github.com/tafia/quick-xml) |
| `quote` | `1.0.47` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/quote](https://github.com/dtolnay/quote) |
| `r-efi` | `5.3.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `r-efi` | `6.0.0` | `MIT OR Apache-2.0 OR LGPL-2.1-or-later` | [https://github.com/r-efi/r-efi](https://github.com/r-efi/r-efi) |
| `rand` | `0.9.5` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand` | `0.7.3` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand` | `0.8.8` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_chacha` | `0.9.0` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_chacha` | `0.3.1` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_chacha` | `0.2.2` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_core` | `0.9.5` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_core` | `0.6.4` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_core` | `0.5.1` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_hc` | `0.2.0` | `MIT/Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `rand_pcg` | `0.2.1` | `MIT OR Apache-2.0` | [https://github.com/rust-random/rand](https://github.com/rust-random/rand) |
| `raw-window-handle` | `0.6.2` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/rust-windowing/raw-window-handle](https://github.com/rust-windowing/raw-window-handle) |
| `rctree` | `0.5.0` | `MIT` | [https://github.com/RazrFalcon/rctree](https://github.com/RazrFalcon/rctree) |
| `redox_syscall` | `0.4.1` | `MIT` | [https://gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_syscall` | `0.5.18` | `MIT` | [https://gitlab.redox-os.org/redox-os/syscall](https://gitlab.redox-os.org/redox-os/syscall) |
| `redox_syscall` | `0.9.5` | `MIT` | [https://gitlab.redox-os.org/redox-os/kernel](https://gitlab.redox-os.org/redox-os/kernel) |
| `redox_users` | `0.5.3` | `MIT` | [https://gitlab.redox-os.org/redox-os/users](https://gitlab.redox-os.org/redox-os/users) |
| `regex` | `1.13.1` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-automata` | `0.4.18` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `regex-syntax` | `0.8.11` | `MIT OR Apache-2.0` | [https://github.com/rust-lang/regex](https://github.com/rust-lang/regex) |
| `resvg` | `0.37.0` | `MPL-2.0` | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| `rfd` | `0.15.4` | `MIT` | [https://github.com/PolyMeilex/rfd](https://github.com/PolyMeilex/rfd) |
| `rgb` | `0.8.53` | `MIT` | [https://github.com/kornelski/rust-rgb](https://github.com/kornelski/rust-rgb) |
| `ring` | `0.17.14` | `Apache-2.0 AND ISC` | [https://github.com/briansmith/ring](https://github.com/briansmith/ring) |
| `rodio` | `0.19.0` | `MIT OR Apache-2.0` | [https://github.com/RustAudio/rodio](https://github.com/RustAudio/rodio) |
| `ron` | `0.8.1` | `MIT OR Apache-2.0` | [https://github.com/ron-rs/ron](https://github.com/ron-rs/ron) |
| `roxmltree` | `0.19.0` | `MIT OR Apache-2.0` | [https://github.com/RazrFalcon/roxmltree](https://github.com/RazrFalcon/roxmltree) |
| `rusqlite` | `0.32.1` | `MIT` | [https://github.com/rusqlite/rusqlite](https://github.com/rusqlite/rusqlite) |
| `rustc-hash` | `2.1.3` | `Apache-2.0 OR MIT` | [https://github.com/rust-lang/rustc-hash](https://github.com/rust-lang/rustc-hash) |
| `rustc_version` | `0.4.1` | `MIT OR Apache-2.0` | [https://github.com/djc/rustc-version-rs](https://github.com/djc/rustc-version-rs) |
| `rustfft` | `6.4.1` | `MIT OR Apache-2.0` | [https://github.com/ejmahler/RustFFT](https://github.com/ejmahler/RustFFT) |
| `rustix` | `0.38.44` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `rustix` | `1.1.6` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/rustix](https://github.com/bytecodealliance/rustix) |
| `rustls` | `0.23.46` | `Apache-2.0 OR ISC OR MIT` | [https://github.com/rustls/rustls](https://github.com/rustls/rustls) |
| `rustls-pki-types` | `1.15.1` | `MIT OR Apache-2.0` | [https://github.com/rustls/pki-types](https://github.com/rustls/pki-types) |
| `rustls-webpki` | `0.103.16` | `ISC` | [https://github.com/rustls/webpki](https://github.com/rustls/webpki) |
| `rustversion` | `1.0.23` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/rustversion](https://github.com/dtolnay/rustversion) |
| `same-file` | `1.0.6` | `Unlicense/MIT` | [https://github.com/BurntSushi/same-file](https://github.com/BurntSushi/same-file) |
| `scoped-tls` | `1.0.1` | `MIT/Apache-2.0` | [https://github.com/alexcrichton/scoped-tls](https://github.com/alexcrichton/scoped-tls) |
| `scopeguard` | `1.2.0` | `MIT OR Apache-2.0` | [https://github.com/bluss/scopeguard](https://github.com/bluss/scopeguard) |
| `selectors` | `0.22.0` | `MPL-2.0` | [https://github.com/servo/servo](https://github.com/servo/servo) |
| `semver` | `1.0.28` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/semver](https://github.com/dtolnay/semver) |
| `serde` | `1.0.229` | `MIT OR Apache-2.0` | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_core` | `1.0.229` | `MIT OR Apache-2.0` | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_derive` | `1.0.229` | `MIT OR Apache-2.0` | [https://github.com/serde-rs/serde](https://github.com/serde-rs/serde) |
| `serde_json` | `1.0.154` | `MIT OR Apache-2.0` | [https://github.com/serde-rs/json](https://github.com/serde-rs/json) |
| `serde_repr` | `0.1.21` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/serde-repr](https://github.com/dtolnay/serde-repr) |
| `serde_spanned` | `0.6.9` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `servo_arc` | `0.1.1` | `MIT/Apache-2.0` | [https://github.com/servo/servo](https://github.com/servo/servo) |
| `sha2` | `0.10.9` | `MIT OR Apache-2.0` | [https://github.com/RustCrypto/hashes](https://github.com/RustCrypto/hashes) |
| `shlex` | `1.3.0` | `MIT OR Apache-2.0` | [https://github.com/comex/rust-shlex](https://github.com/comex/rust-shlex) |
| `shlex` | `2.0.1` | `MIT OR Apache-2.0` | [https://github.com/comex/rust-shlex](https://github.com/comex/rust-shlex) |
| `signal-hook-registry` | `1.4.8` | `MIT OR Apache-2.0` | [https://github.com/vorner/signal-hook](https://github.com/vorner/signal-hook) |
| `simd-adler32` | `0.3.10` | `MIT` | [https://github.com/mcountryman/simd-adler32](https://github.com/mcountryman/simd-adler32) |
| `simd_cesu8` | `1.2.0` | `Apache-2.0 OR MIT` | [https://github.com/seancroach/simd_cesu8](https://github.com/seancroach/simd_cesu8) |
| `simdutf8` | `0.1.5` | `MIT OR Apache-2.0` | [https://github.com/rusticstuff/simdutf8](https://github.com/rusticstuff/simdutf8) |
| `simplecss` | `0.2.2` | `Apache-2.0 OR MIT` | [https://github.com/linebender/simplecss](https://github.com/linebender/simplecss) |
| `siphasher` | `1.0.4` | `MIT OR Apache-2.0` | [https://github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| `siphasher` | `0.3.11` | `MIT/Apache-2.0` | [https://github.com/jedisct1/rust-siphash](https://github.com/jedisct1/rust-siphash) |
| `slab` | `0.4.12` | `MIT` | [https://github.com/tokio-rs/slab](https://github.com/tokio-rs/slab) |
| `slotmap` | `1.1.1` | `Zlib` | [https://github.com/orlp/slotmap](https://github.com/orlp/slotmap) |
| `smallvec` | `1.16.5` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-smallvec](https://github.com/servo/rust-smallvec) |
| `smithay-client-toolkit` | `0.20.0` | `MIT` | [https://github.com/smithay/client-toolkit](https://github.com/smithay/client-toolkit) |
| `smithay-client-toolkit` | `0.19.2` | `MIT` | [https://github.com/smithay/client-toolkit](https://github.com/smithay/client-toolkit) |
| `smithay-clipboard` | `0.7.3` | `MIT` | [https://github.com/smithay/smithay-clipboard](https://github.com/smithay/smithay-clipboard) |
| `smol_str` | `0.2.2` | `MIT OR Apache-2.0` | [https://github.com/rust-analyzer/smol_str](https://github.com/rust-analyzer/smol_str) |
| `soup3` | `0.5.0` | `MIT` | [https://gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `soup3-sys` | `0.5.0` | `MIT` | [https://gitlab.gnome.org/World/Rust/soup3-rs](https://gitlab.gnome.org/World/Rust/soup3-rs) |
| `stable_deref_trait` | `1.2.1` | `MIT OR Apache-2.0` | [https://github.com/storyyeller/stable_deref_trait](https://github.com/storyyeller/stable_deref_trait) |
| `static_assertions` | `1.1.0` | `MIT OR Apache-2.0` | [https://github.com/nvzqz/static-assertions-rs](https://github.com/nvzqz/static-assertions-rs) |
| `strength_reduce` | `0.2.4` | `MIT OR Apache-2.0` | [http://github.com/ejmahler/strength_reduce](http://github.com/ejmahler/strength_reduce) |
| `strict-num` | `0.1.1` | `MIT` | [https://github.com/RazrFalcon/strict-num](https://github.com/RazrFalcon/strict-num) |
| `string_cache` | `0.8.9` | `MIT OR Apache-2.0` | [https://github.com/servo/string-cache](https://github.com/servo/string-cache) |
| `string_cache_codegen` | `0.5.4` | `MIT OR Apache-2.0` | [https://github.com/servo/string-cache](https://github.com/servo/string-cache) |
| `subtle` | `2.6.1` | `BSD-3-Clause` | [https://github.com/dalek-cryptography/subtle](https://github.com/dalek-cryptography/subtle) |
| `svgtypes` | `0.13.0` | `MIT/Apache-2.0` | [https://github.com/RazrFalcon/svgtypes](https://github.com/RazrFalcon/svgtypes) |
| `symphonia` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-bundle-flac` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-bundle-mp3` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-codec-aac` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-codec-adpcm` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-codec-alac` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-codec-pcm` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-codec-vorbis` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-core` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-format-caf` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-format-isomp4` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-format-mkv` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-format-ogg` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-format-riff` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-metadata` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `symphonia-utils-xiph` | `0.5.5` | `MPL-2.0` | [https://github.com/pdeljanov/Symphonia](https://github.com/pdeljanov/Symphonia) |
| `syn` | `3.0.9` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `syn` | `1.0.109` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `syn` | `2.0.119` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/syn](https://github.com/dtolnay/syn) |
| `synstructure` | `0.14.0` | `MIT` | [https://github.com/mystor/synstructure](https://github.com/mystor/synstructure) |
| `system-deps` | `6.2.2` | `MIT OR Apache-2.0` | [https://github.com/gdesmott/system-deps](https://github.com/gdesmott/system-deps) |
| `tao` | `0.30.8` | `Apache-2.0` | [https://github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |
| `tao-macros` | `0.1.4` | `MIT OR Apache-2.0` | [https://github.com/tauri-apps/tao](https://github.com/tauri-apps/tao) |
| `target-lexicon` | `0.12.16` | `Apache-2.0 WITH LLVM-exception` | [https://github.com/bytecodealliance/target-lexicon](https://github.com/bytecodealliance/target-lexicon) |
| `tempfile` | `3.27.0` | `MIT OR Apache-2.0` | [https://github.com/Stebalien/tempfile](https://github.com/Stebalien/tempfile) |
| `tendril` | `0.4.3` | `MIT/Apache-2.0` | [https://github.com/servo/tendril](https://github.com/servo/tendril) |
| `thin-slice` | `0.1.1` | `MPL-2.0` | [https://github.com/heycam/thin-slice](https://github.com/heycam/thin-slice) |
| `thiserror` | `2.0.21` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror` | `1.0.69` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | `1.0.69` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `thiserror-impl` | `2.0.21` | `MIT OR Apache-2.0` | [https://github.com/dtolnay/thiserror](https://github.com/dtolnay/thiserror) |
| `time` | `0.3.55` | `MIT OR Apache-2.0` | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| `time-core` | `0.1.9` | `MIT OR Apache-2.0` | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| `time-macros` | `0.2.32` | `MIT OR Apache-2.0` | [https://github.com/time-rs/time](https://github.com/time-rs/time) |
| `tiny-skia` | `0.11.4` | `BSD-3-Clause` | [https://github.com/RazrFalcon/tiny-skia](https://github.com/RazrFalcon/tiny-skia) |
| `tiny-skia-path` | `0.11.4` | `BSD-3-Clause` | [https://github.com/RazrFalcon/tiny-skia/tree/master/path](https://github.com/RazrFalcon/tiny-skia/tree/master/path) |
| `tinystr` | `0.8.5` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `tinyvec` | `1.13.3` | `Zlib OR Apache-2.0 OR MIT` | [https://github.com/Lokathor/tinyvec](https://github.com/Lokathor/tinyvec) |
| `tokio` | `1.54.1` | `MIT` | [https://github.com/tokio-rs/tokio](https://github.com/tokio-rs/tokio) |
| `toml` | `0.8.2` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_datetime` | `0.6.3` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_datetime` | `1.1.3+spec-1.1.0` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | `0.25.19+spec-1.1.0` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | `0.20.2` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_edit` | `0.19.15` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `toml_parser` | `1.1.5+spec-1.1.0` | `MIT OR Apache-2.0` | [https://github.com/toml-rs/toml](https://github.com/toml-rs/toml) |
| `tracing` | `0.1.44` | `MIT` | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-attributes` | `0.1.31` | `MIT` | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `tracing-core` | `0.1.36` | `MIT` | [https://github.com/tokio-rs/tracing](https://github.com/tokio-rs/tracing) |
| `transpose` | `0.2.3` | `MIT OR Apache-2.0` | [https://github.com/ejmahler/transpose](https://github.com/ejmahler/transpose) |
| `tray-icon` | `0.26.1` | `MIT OR Apache-2.0` | [https://github.com/tauri-apps/tray-icon](https://github.com/tauri-apps/tray-icon) |
| `ttf-parser` | `0.25.1` | `MIT OR Apache-2.0` | [https://github.com/harfbuzz/ttf-parser](https://github.com/harfbuzz/ttf-parser) |
| `typenum` | `1.20.1` | `MIT OR Apache-2.0` | [https://github.com/paholg/typenum](https://github.com/paholg/typenum) |
| `uds_windows` | `1.2.1` | `MIT` | [https://github.com/haraldh/rust_uds_windows](https://github.com/haraldh/rust_uds_windows) |
| `unicase` | `2.10.0` | `MIT OR Apache-2.0` | [https://github.com/seanmonstar/unicase](https://github.com/seanmonstar/unicase) |
| `unicode-ident` | `1.0.27` | `(MIT OR Apache-2.0) AND Unicode-3.0` | [https://github.com/dtolnay/unicode-ident](https://github.com/dtolnay/unicode-ident) |
| `unicode-segmentation` | `1.13.3` | `MIT OR Apache-2.0` | [https://github.com/unicode-rs/unicode-segmentation](https://github.com/unicode-rs/unicode-segmentation) |
| `untrusted` | `0.9.0` | `ISC` | [https://github.com/briansmith/untrusted](https://github.com/briansmith/untrusted) |
| `ureq` | `2.12.1` | `MIT OR Apache-2.0` | [https://github.com/algesten/ureq](https://github.com/algesten/ureq) |
| `url` | `2.5.8` | `MIT OR Apache-2.0` | [https://github.com/servo/rust-url](https://github.com/servo/rust-url) |
| `urlencoding` | `2.1.3` | `MIT` | [https://github.com/kornelski/rust_urlencoding](https://github.com/kornelski/rust_urlencoding) |
| `usvg` | `0.37.0` | `MPL-2.0` | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| `usvg-parser` | `0.37.0` | `MPL-2.0` | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| `usvg-tree` | `0.37.0` | `MPL-2.0` | [https://github.com/RazrFalcon/resvg](https://github.com/RazrFalcon/resvg) |
| `utf-8` | `0.7.6` | `MIT OR Apache-2.0` | [https://github.com/SimonSapin/rust-utf8](https://github.com/SimonSapin/rust-utf8) |
| `utf8_iter` | `1.1.0` | `Apache-2.0 OR MIT` | [https://github.com/hsivonen/utf8_iter](https://github.com/hsivonen/utf8_iter) |
| `utf_types` | `1.0.0` | `Apache-2.0 OR MIT` | [https://github.com/hsivonen/utf_types](https://github.com/hsivonen/utf_types) |
| `uuid` | `1.29.1` | `Apache-2.0 OR MIT` | [https://github.com/uuid-rs/uuid](https://github.com/uuid-rs/uuid) |
| `vcpkg` | `0.2.15` | `MIT/Apache-2.0` | [https://github.com/mcgoo/vcpkg-rs](https://github.com/mcgoo/vcpkg-rs) |
| `version-compare` | `0.2.1` | `MIT` | [https://gitlab.com/timvisee/version-compare](https://gitlab.com/timvisee/version-compare) |
| `version_check` | `0.9.5` | `MIT/Apache-2.0` | [https://github.com/SergioBenitez/version_check](https://github.com/SergioBenitez/version_check) |
| `walkdir` | `2.5.0` | `Unlicense/MIT` | [https://github.com/BurntSushi/walkdir](https://github.com/BurntSushi/walkdir) |
| `wasi` | `0.11.1+wasi-snapshot-preview1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| `wasi` | `0.9.0+wasi-snapshot-preview1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/wasi](https://github.com/bytecodealliance/wasi) |
| `wasip2` | `1.0.4+wasi-0.2.12` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/wasi-rs](https://github.com/bytecodealliance/wasi-rs) |
| `wasm-bindgen` | `0.2.130` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen](https://github.com/wasm-bindgen/wasm-bindgen) |
| `wasm-bindgen-futures` | `0.4.80` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/futures) |
| `wasm-bindgen-macro` | `0.2.130` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/macro) |
| `wasm-bindgen-macro-support` | `0.2.130` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support](https://github.com/wasm-bindgen/wasm-bindgen/tree/main/crates/macro-support) |
| `wasm-bindgen-shared` | `0.2.130` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/shared) |
| `wayland-backend` | `0.3.17` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-client` | `0.31.15` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-csd-frame` | `0.3.0` | `MIT` | [https://github.com/rust-windowing/wayland-csd-frame](https://github.com/rust-windowing/wayland-csd-frame) |
| `wayland-cursor` | `0.31.14` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols` | `0.32.13` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-experimental` | `20250721.0.1` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-misc` | `0.3.12` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-plasma` | `0.3.12` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-protocols-wlr` | `0.3.12` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-scanner` | `0.31.11` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `wayland-sys` | `0.31.11` | `MIT` | [https://github.com/smithay/wayland-rs](https://github.com/smithay/wayland-rs) |
| `web-sys` | `0.3.107` | `MIT OR Apache-2.0` | [https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys](https://github.com/wasm-bindgen/wasm-bindgen/tree/master/crates/web-sys) |
| `web-time` | `1.1.0` | `MIT OR Apache-2.0` | [https://github.com/daxpedda/web-time](https://github.com/daxpedda/web-time) |
| `webbrowser` | `1.2.6` | `MIT OR Apache-2.0` | [https://github.com/amodm/webbrowser-rs](https://github.com/amodm/webbrowser-rs) |
| `webkit2gtk` | `2.0.1` | `MIT` | [https://github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webkit2gtk-sys` | `2.0.1` | `MIT` | [https://github.com/tauri-apps/webkit2gtk-rs](https://github.com/tauri-apps/webkit2gtk-rs) |
| `webpki-roots` | `1.0.11` | `CDLA-Permissive-2.0` | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| `webpki-roots` | `0.26.11` | `CDLA-Permissive-2.0` | [https://github.com/rustls/webpki-roots](https://github.com/rustls/webpki-roots) |
| `webview2-com` | `0.33.0` | `MIT` | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `webview2-com-macros` | `0.8.1` | `MIT` | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `webview2-com-sys` | `0.33.0` | `MIT` | [https://github.com/wravery/webview2-rs](https://github.com/wravery/webview2-rs) |
| `weezl` | `0.1.12` | `MIT OR Apache-2.0` | [https://github.com/image-rs/weezl](https://github.com/image-rs/weezl) |
| `winapi` | `0.3.9` | `MIT/Apache-2.0` | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `winapi-i686-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `winapi-util` | `0.1.11` | `Unlicense OR MIT` | [https://github.com/BurntSushi/winapi-util](https://github.com/BurntSushi/winapi-util) |
| `winapi-x86_64-pc-windows-gnu` | `0.4.0` | `MIT/Apache-2.0` | [https://github.com/retep998/winapi-rs](https://github.com/retep998/winapi-rs) |
| `windows` | `0.58.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows` | `0.54.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-core` | `0.58.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-core` | `0.54.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-implement` | `0.58.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-interface` | `0.58.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-link` | `0.2.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-result` | `0.2.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-result` | `0.1.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-strings` | `0.1.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | `0.60.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | `0.59.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | `0.52.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | `0.45.0` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-sys` | `0.61.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | `0.53.5` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-targets` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows-version` | `0.1.7` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_gnullvm` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_aarch64_msvc` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnu` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_gnullvm` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_i686_msvc` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnu` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_gnullvm` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | `0.42.2` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | `0.53.1` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `windows_x86_64_msvc` | `0.52.6` | `MIT OR Apache-2.0` | [https://github.com/microsoft/windows-rs](https://github.com/microsoft/windows-rs) |
| `winit` | `0.30.13` | `Apache-2.0` | [https://github.com/rust-windowing/winit](https://github.com/rust-windowing/winit) |
| `winnow` | `0.5.40` | `MIT` | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `winnow` | `1.0.5` | `MIT` | [https://github.com/winnow-rs/winnow](https://github.com/winnow-rs/winnow) |
| `wit-bindgen` | `0.57.1` | `Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT` | [https://github.com/bytecodealliance/wit-bindgen](https://github.com/bytecodealliance/wit-bindgen) |
| `writeable` | `0.6.4` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `wry` | `0.47.2` | `Apache-2.0 OR MIT` | [https://github.com/tauri-apps/wry](https://github.com/tauri-apps/wry) |
| `x11` | `2.21.0` | `MIT` | [https://github.com/AltF02/x11-rs.git](https://github.com/AltF02/x11-rs.git) |
| `x11-dl` | `2.21.0` | `MIT` | [https://github.com/AltF02/x11-rs.git](https://github.com/AltF02/x11-rs.git) |
| `x11rb` | `0.13.2` | `MIT OR Apache-2.0` | [https://github.com/psychon/x11rb](https://github.com/psychon/x11rb) |
| `x11rb-protocol` | `0.13.2` | `MIT OR Apache-2.0` | [https://github.com/psychon/x11rb](https://github.com/psychon/x11rb) |
| `xcursor` | `0.3.11` | `MIT` | [https://github.com/esposm03/xcursor-rs](https://github.com/esposm03/xcursor-rs) |
| `xkbcommon-dl` | `0.4.2` | `MIT` | [https://github.com/rust-windowing/xkbcommon-dl](https://github.com/rust-windowing/xkbcommon-dl) |
| `xkeysym` | `0.2.1` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/notgull/xkeysym](https://github.com/notgull/xkeysym) |
| `xml-rs` | `0.8.29` | `MIT` | [https://github.com/kornelski/xml-rs](https://github.com/kornelski/xml-rs) |
| `xmlwriter` | `0.1.0` | `MIT` | [https://github.com/RazrFalcon/xmlwriter](https://github.com/RazrFalcon/xmlwriter) |
| `yoke` | `0.8.4` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `yoke-derive` | `0.8.4` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zbus` | `5.19.0` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zbus_macros` | `5.19.0` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zbus_names` | `4.3.5` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zcheapstr` | `1.2.0` | `MIT` | [https://github.com/z-galaxy/zcheapstr/](https://github.com/z-galaxy/zcheapstr/) |
| `zerocopy` | `0.8.63` | `BSD-2-Clause OR Apache-2.0 OR MIT` | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerocopy-derive` | `0.8.63` | `BSD-2-Clause OR Apache-2.0 OR MIT` | [https://github.com/google/zerocopy](https://github.com/google/zerocopy) |
| `zerofrom` | `0.1.8` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerofrom-derive` | `0.1.8` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zeroize` | `1.9.1` | `Apache-2.0 OR MIT` | [https://github.com/RustCrypto/utils](https://github.com/RustCrypto/utils) |
| `zerotrie` | `0.2.5` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec` | `0.11.9` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zerovec-derive` | `0.11.7` | `Unicode-3.0` | [https://github.com/unicode-org/icu4x](https://github.com/unicode-org/icu4x) |
| `zlib-rs` | `0.6.8` | `Zlib` | [https://github.com/trifectatechfoundation/zlib-rs](https://github.com/trifectatechfoundation/zlib-rs) |
| `zmij` | `1.0.23` | `MIT` | [https://github.com/dtolnay/zmij](https://github.com/dtolnay/zmij) |
| `zune-core` | `0.5.3` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/etemesi254/zune-image](https://github.com/etemesi254/zune-image) |
| `zune-jpeg` | `0.5.15` | `MIT OR Apache-2.0 OR Zlib` | [https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg](https://github.com/etemesi254/zune-image/tree/dev/crates/zune-jpeg) |
| `zvariant` | `5.15.1` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zvariant_derive` | `5.15.1` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |
| `zvariant_utils` | `4.2.0` | `MIT` | [https://github.com/z-galaxy/zbus/](https://github.com/z-galaxy/zbus/) |

---

## Common License Texts

### MIT License

```
MIT License

Permission is hereby granted, free of charge, to any person obtaining a copy
of this software and associated documentation files (the "Software"), to deal
in the Software without restriction, including without limitation the rights
to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
copies of the Software, and to permit persons to whom the Software is
furnished to do so, subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
SOFTWARE.
```

### Apache License 2.0

```
Apache License
Version 2.0, January 2004
http://www.apache.org/licenses/

TERMS AND CONDITIONS FOR USE, REPRODUCTION, AND DISTRIBUTION

1. Definitions.
"License" shall mean the terms and conditions for use, reproduction, and
distribution as defined by Sections 1 through 9 of this document.

"Licensor" shall mean the copyright owner or entity authorized by the copyright
owner that is granting the License.

"Legal Entity" shall mean the union of the acting entity and all other entities
that control, are controlled by, or are under common control with that entity.

"You" (or "Your") shall mean an individual or Legal Entity exercising
permissions granted by this License.

2. Grant of Copyright License. Subject to the terms and conditions of this
License, each Contributor hereby grants to You a perpetual, worldwide,
non-exclusive, no-charge, royalty-free, irrevocable copyright license to
reproduce, prepare Derivative Works of, publicly display, publicly perform,
sublicense, and distribute the Work and such Derivative Works in Source or
Object form.

3. Grant of Patent License. Subject to the terms and conditions of this License,
each Contributor hereby grants to You a perpetual, worldwide, non-exclusive,
no-charge, royalty-free, irrevocable patent license to make, have made, use,
offer to sell, sell, import, and otherwise transfer the Work.

4. Redistribution. You may reproduce and distribute copies of the Work or
Derivative Works thereof in any medium, with or without modifications, and in
Source or Object form, provided that You meet the following conditions:
(a) You must give any other recipients of the Work or Derivative Works a copy
of this License; and
(b) You must cause any modified files to carry prominent notices stating that
You changed the files; and
(c) You must retain, in the Source form of any Derivative Works that You
distribute, all copyright, patent, trademark, and attribution notices from the
Source form of the Work; and
(d) If the Work includes a "NOTICE" text file, You must include a readable
copy of the attribution notices contained within such NOTICE file.
```

### Mozilla Public License 2.0 (MPL-2.0)

```
Mozilla Public License Version 2.0
==================================

1. Definitions
1.1. "Contributor" means each individual or legal entity that creates, contributes
to the creation of, or owns Covered Software.
1.2. "Contributor Version" means the combination of the Contributions of others
(if any) used by a Contributor and that particular Contributor's Contribution.
1.3. "Contribution" means Covered Software of a particular Contributor.
1.4. "Covered Software" means Source Code Form to which the initial Contributor
has attached the notice in Exhibit A, the Executable Form of such Source Code
Form, and Modifications of such Source Code Form, in each case including portions
thereof.

2. License Grants and Conditions
2.1. Grants
Each Contributor hereby grants You a world-wide, royalty-free, non-exclusive
license under Intellectual Property Rights to use, reproduce, make available,
modify, display, perform, distribute, and otherwise exploit its Contributions,
either on an unmodified basis, with Modifications, or as part of a Larger Work.

3. Responsibilities
3.1. Distribution of Source Form
All distribution of Covered Software in Source Code Form must be made under the
terms of this License.
3.2. Distribution of Executable Form
If You distribute Covered Software in Executable Form then such Covered Software
must also be made available in Source Code Form.
```

### BSD 2-Clause and 3-Clause Licenses

```
Redistribution and use in source and binary forms, with or without
modification, are permitted provided that the following conditions are met:

1. Redistributions of source code must retain the above copyright notice, this
   list of conditions and the following disclaimer.
2. Redistributions in binary form must reproduce the above copyright notice,
   this list of conditions and the following disclaimer in the documentation
   and/or other materials provided with the distribution.
3. Neither the name of the copyright holder nor the names of its contributors
   may be used to endorse or promote products derived from this software
   without specific prior written permission.

THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS "AS IS"
AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE
IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE
DISCLAIMED. IN NO EVENT SHALL THE COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE
FOR ANY DIRECT, INDIRECT, INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL
DAMAGES (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER
CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY,
OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE
OF THIS SOFTWARE, EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
```
