# libraw-rs

[![Rust](https://img.shields.io/badge/rust-1.95%2B-orange)](https://www.rust-lang.org)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue)](LICENSE)

Idiomatic Rust bindings for [LibRaw](https://www.libraw.org/) — the industry-standard library for reading and processing RAW image files from digital cameras.

Built on LibRaw 0.22.1, supporting **1,200+ camera models** from Canon, Nikon, Sony, Fujifilm, Leica, Pentax, Olympus, Panasonic, Hasselblad, Phase One, and many more.

## Quick Start

```toml
[dependencies]
libraw-rs = "0.1"
```

```rust
use libraw_rs::LibRaw;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // One-liner: open a RAW file
    let mut raw = LibRaw::open("photo.CR3")?;

    // Read EXIF metadata
    let info = raw.image_info();
    println!("Camera: {} {}", info.make(), info.model());
    println!("ISO: {}", raw.image_other().iso_speed());
    println!("Aperture: f/{:.1}", raw.image_other().aperture());

    // Process to sRGB image
    raw.process_default()?;
    let img = raw.to_image()?;
    img.save("output.jpg")?;

    Ok(())
}
```

```bash
cargo run --example simple -- /path/to/photo.RAW
```

## Architecture

libraw-rs uses pre-built static libraries linked via `build.rs` + bindgen-generated FFI, then wraps everything in safe, idiomatic Rust.

```
┌─────────────────────────────────┐
│  High-level Facade              │  ──  open(), process_default(), to_image()
├─────────────────────────────────┤
│  Expert Layer                   │  ──  unpack(), raw2image(), dcraw_process(), make_mem_image()
├─────────────────────────────────┤
│  Types & Accessors              │  ──  ImageInfo, LensInfo, OutputParams, ProcessedImage
├─────────────────────────────────┤
│  Safe Wrappers                  │  ──  RAII (Drop), Result<T,E>, builder pattern, &'a lifetimes
├─────────────────────────────────┤
│  sys module (bindgen)           │  ──  extern "C" FFI bindings
├─────────────────────────────────┤
│  libraw_static.lib / libraw_r.a │  ──  Pre-built C++ static library
└─────────────────────────────────┘
```

## Module Overview

| Module | Purpose |
|---|---|
| `LibRaw` | Main context — creation, file open, lifecycle (RAII via `Drop`) |
| `error` | `Error` enum (17 variants) mapped from LibRaw error codes, `Result<T>` alias |
| `params` | `OutputParams` builder — 25+ chainable setters, `OutputColor`, `HighlightMode` enums |
| `metadata` | `ImageInfo`, `LensInfo`, `ImageOther`, `ImageSizes` — lifetime-bound read-only accessors |
| `image` | `ProcessedImage` — in-memory image result, `ImageType` enum, auto-free on drop |
| `process` | Expert pipeline: `unpack()`, `dcraw_process()`, `make_mem_image()`, etc. |
| `stream` | `open_buffer()` (in-memory) and `open_bayer()` (raw sensor data) |
| `callbacks` | EXIF/makernotes/data-error/progress callback registration |
| `utils` | `version()`, `camera_count()`, `camera_list()`, `capabilities()` |

## API Examples

### Read Metadata (no processing needed)

```rust
let raw = LibRaw::open("photo.NEF")?;

let info = raw.image_info();
println!("Camera:  {} {}", info.make(), info.model());
println!("Foveon:  {}", info.is_foveon());
println!("Filters: {:#x}", info.filters());

let sizes = raw.sizes();
println!("Size:    {}×{} (raw: {}×{})",
    sizes.width(), sizes.height(),
    sizes.raw_width(), sizes.raw_height());

let other = raw.image_other();
println!("ISO:     {}", other.iso_speed());
println!("Shutter: 1/{}s", (1.0 / other.shutter()) as i32);
println!("Artist:  {}", other.artist());

let lens = raw.lens_info();
println!("Lens:    {} {} ({:.0}–{:.0}mm f/{:.1})",
    lens.make(), lens.model(),
    lens.min_focal(), lens.max_focal(),
    lens.max_ap4_min_focal());
```

### Process with Custom Parameters

```rust
use libraw_rs::{OutputParams, OutputColor, HighlightMode};

let params = OutputParams::default()
    .output_color(OutputColor::AdobeRgb)
    .output_bps(16)
    .highlight_mode(HighlightMode::Blend)
    .use_camera_wb(true)
    .brightness(1.2)
    .half_size(false);

let mut raw = LibRaw::open("photo.CR3")?;
raw.process_with(&params)?;
let img = raw.to_image()?;
img.save("output.tiff")?;
```

### Expert Pipeline (Fine-Grained Control)

```rust
let mut raw = LibRaw::new()?;
raw.open_file("photo.DNG")?;

// Step-by-step control
raw.unpack()?;
raw.subtract_black();
raw.raw2image()?;

// Query decoder info
println!("Decoder: {}", raw.unpack_function_name());

// Access raw sensor data via C API getters
println!("RAW dimensions: {}×{}", raw.raw_width(), raw.raw_height());
println!("Color max: {}", raw.color_maximum());

// Camera multiplier at index 0 (Red)
let red_mul = raw.cam_mul(0);

raw.free_image();
raw.recycle(); // Ready for another file

// Set progress callback
raw.set_progress_handler(|stage, iter, expected| {
    println!("Stage {stage}: {iter}/{expected}");
    libraw_rs::callbacks::ProgressAction::Continue
});
```

### Extract JPEG Preview / Thumbnail

RAW files often contain an embedded JPEG preview. Use `unpack_thumb()` for fast extraction without processing the full sensor data:

```rust
let mut raw = LibRaw::open("photo.CR3")?;

// Unpack only the embedded thumbnail (fast)
raw.unpack_thumb()?;

if raw.is_jpeg_thumb() {
    let thumb = raw.make_mem_thumb()?;
    thumb.save("preview.jpg")?;
    println!("{}×{} — {} bytes", thumb.width(), thumb.height(), thumb.data().len());
}
```

```bash
cargo run --example extract_thumb -- photo.CR3 preview.jpg
```

### Convert to TIFF (One-liner)

```rust
libraw_rs::LibRaw::convert_to_tiff("input.RAW", "output.tiff")?;
```

### List Supported Cameras

```rust
use libraw_rs::{camera_count, camera_list};

println!("{} cameras supported", camera_count());
let list = camera_list();
// Each entry is a *const c_char in "Make/Model" format
```

## Requirements

### Runtime
- **Windows**: MSVC toolchain (the pre-built `libraw_static.lib` is MSVC-compiled)
- **macOS**: Standard Xcode toolchain (`libraw_r.a` with RawSpeed)
- **Linux**: `libraw.a` placed in `static_lib/linux/`

### Build-time (bindgen)
- **LLVM/Clang** — `LIBCLANG_PATH` should point to `libclang` (e.g., `C:\Program Files\LLVM\bin`)
- The static libraries are pre-built — no C++ compiler needed at build time

## Building

```bash
# Set LIBCLANG_PATH if not on PATH
export LIBCLANG_PATH="/c/Program Files/LLVM/bin"

# Build
cargo build

# Run tests
cargo test

# Run examples
cargo run --example simple -- path/to/file.RAW
cargo run --example extract_thumb -- path/to/file.RAW preview.jpg
```

## License

The Rust bindings are licensed under **MIT OR Apache-2.0**.

LibRaw itself is dual-licensed under **LGPL 2.1** and **CDDL 1.0**. Users of this crate must comply with one of LibRaw's licenses when distributing compiled binaries.

## Links

- [LibRaw Official Site](https://www.libraw.org/)
- [LibRaw GitHub](https://github.com/LibRaw/LibRaw)
- [LibRaw API Reference](https://www.libraw.org/docs/API-C-eng.html)
