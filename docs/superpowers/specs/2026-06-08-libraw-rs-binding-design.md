# libraw-rs: Rust Bindings for LibRaw — Design Spec

**Date:** 2026-06-08  
**Author:** Nicely  
**Status:** Draft

## 1. Overview

Create idiomatic Rust bindings for [LibRaw](https://www.libraw.org/) v0.22.1, a C++ library for reading and processing RAW image files from digital cameras. LibRaw is vendored as a git submodule at `vendor/LibRaw`.

### 1.1 Goals

- Provide a safe, idiomatic Rust API for all LibRaw C API functions
- Build LibRaw from vendored source via CMake, with no system-installed LibRaw required
- Follow Rust ecosystem conventions (RAII, `Result<T, E>`, builder pattern)
- Offer two API tiers: a high-level facade for common tasks, and expert-level access for full control

### 1.2 Non-Goals

- Wrapping the C++ class API directly (bind to the C API facade only)
- GPU-accelerated processing
- Streaming/chunked processing of very large RAW files beyond what LibRaw supports

## 2. Architecture

### 2.1 Crate Structure

Single crate `libraw-rs`, with two internal modules:

```
libraw-rs/
├── build.rs                    # CMake build + bindgen generation
├── Cargo.toml
├── vendor/LibRaw/              # git submodule (v0.22.1)
├── src/
│   ├── lib.rs                  # crate root, re-exports public API
│   ├── sys.rs                  # include!(concat!(env!("OUT_DIR"), "/bindings.rs"))
│   ├── error.rs                # error types and Result alias
│   ├── context.rs              # LibRaw context (init, lifecycle, drop)
│   ├── params.rs               # output parameter builder
│   ├── image.rs                # output image types
│   ├── metadata.rs             # read-only metadata accessors
│   ├── process.rs              # processing pipeline (unpack, dcraw_process, thumbs, etc.)
│   ├── stream.rs               # custom data stream support (memory buffer, open_bayer)
│   └── high_level.rs           # high-level facade API
├── examples/
│   └── simple.rs               # basic usage example
├── tests/
│   ├── integration.rs          # integration tests
│   └── data/                   # test RAW files
```

- `sys` module: bindgen output, generated at build time, not user-facing
- All other modules: safe Rust wrappers forming the public API surface

### 2.2 API Tiers

**High-level facade** (`high_level.rs`): covers the 80% use case with simple methods like `open()`, `process_default()`, `to_image()`, `convert_to_tiff()`. Uses sensible defaults.

**Expert layer** (all other modules): exposes every LibRaw C API function with safe Rust signatures. Users can import and use individual modules for fine-grained control.

## 3. Build System

### 3.1 CMake Build

`build.rs` uses the `cmake` crate to compile LibRaw from `vendor/LibRaw/`:

- Static library build (`LIBRAW_BUILD_STATIC=ON`)
- LCMS support enabled by default (`ENABLE_LCMS=ON`)
- RawSpeed support enabled by default (`ENABLE_RAWSPEED=ON`)
- DNG SDK disabled by default (`ENABLE_DNGSDK=OFF`)
- Examples, docs, shared libs all disabled

### 3.2 bindgen Generation

Run after CMake build:

- Input header: `vendor/LibRaw/libraw/libraw.h`
- Allowlist: `libraw_.*` functions and types
- Opaque: `std::.*` types
- Output: `OUT_DIR/bindings.rs`, included by `src/sys.rs`

### 3.3 Cargo Features → CMake Flags

| Cargo feature | CMake define | Purpose |
|---|---|---|
| *(default, no features)* | — | Basic RAW decoding (no RawSpeed, no LCMS, no OpenMP) |
| `rawspeed` | `ENABLE_RAWSPEED=ON` | High-performance unpack (parallel decode) |
| `lcms` | `ENABLE_LCMS=ON` | ICC color management |
| `dng-sdk` | `ENABLE_DNGSDK=ON` | Adobe DNG SDK support |
| `demosaic-packs` | `USE_DEMOSAIC_PACK_GPL=ON` etc. | Extra demosaic algorithms |
| `openmp` | `ENABLE_OPENMP=ON` | Multi-threaded processing |
| `jasper` | `USE_JASPER=ON` | JPEG 2000 decode |
| `jpeg` | `USE_JPEG=ON` | JPEG thumbnail extraction |

### 3.4 Build Dependencies

- CMake (for building LibRaw)
- Clang / libclang (for bindgen)
- A C++ compiler (msvc, gcc, or clang)

Users set `LIBCLANG_PATH` if libclang is not on the default path.

## 4. Error Handling

### 4.1 Error Type

```rust
#[derive(Error, Debug)]
pub enum LibRawError {
    OutOfMemory,
    UnsupportedFormat,
    Io(String),
    DataError,
    UnsupportedFeature(String),
    Cancelled,
    CallbackError,
    BadParams(String),
    Unknown { code: i32, msg: String },
}

pub type Result<T> = std::result::Result<T, LibRawError>;
```

- Maps ~15 specific LibRaw error codes to structured variants
- Fallback `Unknown` variant catches any unrecognized codes
- Uses `libraw_strerror()` for human-readable error messages in `Display`
- `std::io::Error` is converted to `LibRawError::Io(...)` where relevant

### 4.2 Safety Boundaries

- All `extern "C"` calls are wrapped in `unsafe { }` blocks inside safe Rust methods
- Null pointer returns from `libraw_init()` etc. are checked and converted to `Result::Err`
- No panics escape through FFI boundaries
- Public safe API never panics

## 5. Context & Lifecycle

### 5.1 Core Type

```rust
pub struct LibRaw {
    pub(crate) inner: *mut sys::libraw_data_t,
}
```

### 5.2 Lifecycle

```
new() → open_file() → unpack() → dcraw_process() → make_mem_image() → ProcessedImage
  │        │              │              │                   │
  ▼        ▼              ▼              ▼                   ▼
 drop   recycle()    free_image()   (auto in        dcraw_clear_mem
        (re-open)    raw2image data  process)        (in Drop)
```

- `LibRaw::new()` / `LibRaw::with_flags(flags)` — create context (`libraw_init`)
- `Drop` — release all resources (`libraw_close`)
- `recycle()` — free image data, keep context for another file
- `raw2image()` allocates pixel arrays; `free_image()` releases them

### 5.3 Thread Safety

- `Send`: yes (ownership transfer between threads after `recycle`)
- `Sync`: no (internal mutable state, TLS data)
- Safety relies on `&mut self` for all mutating operations

## 6. Output Parameters

Builder pattern wrapping `libraw_output_params_t` (~40+ fields):

- `OutputParams::default()` initializes to LibRaw's documented defaults
- Chainable setter methods: `.output_color(sRGB)`, `.output_bps(16)`, `.highlight_mode(Blend)`, etc.
- Enum types for named values (`OutputColor`, `HighlightMode`, etc.)
- Applied via `LibRaw::set_params(&params)` or direct access via `params_mut()`

## 7. Metadata Access

### 7.1 Accessor Types

- `ImageInfo<'a>` — wraps `libraw_iparams_t` (camera make/model, ISO, shutter, aperture, sizes, filters, etc.)
- `LensInfo<'a>` — wraps `libraw_lensinfo_t` (focal range, aperture range, lens make/model/serial)
- `ImageOther<'a>` — wraps `libraw_imgother_t` (timestamp, shot order, GPS, description, artist)
- `ImageSizes<'a>` — wraps `libraw_image_sizes_t`

### 7.2 Design

- All accessors borrow from `&LibRaw` via `&'a` lifetime, preventing use-after-free
- `&self` (shared reference) — metadata is read-only after file open
- Available immediately after `open_file()`, no unpack needed
- C string fields (`[u8; N]`) are converted to `&str`, stripping null terminators

## 8. Processing Pipeline

### 8.1 Core Methods (Expert Layer)

- `open_file(path)` / `open_buffer(&[u8])` — open RAW source
- `unpack()` — decode RAW sensor data
- `unpack_thumb()` / `unpack_thumb_ex(index)` — extract thumbnails only
- `raw2image()` — convert encoded data to pixel array
- `free_image()` — release pixel array memory
- `subtract_black()` — subtract black level
- `dcraw_process()` — full dcraw-compatible pipeline (demosaic → color → gamma)
- `make_mem_image()` — produce in-memory output image
- `make_mem_thumb()` — produce in-memory thumbnail
- `save_tiff(path)` — write TIFF/PPM to disk
- `save_thumb(path)` — write thumbnail to disk
- `adjust_to_raw_inset_crop(mask, maxcrop)` — adjust sizes for cropped sensors

### 8.2 Helper Queries

- `is_fuji_rotated()` — check if Fuji rotation applies
- `is_sraw()` / `is_nikon_sraw()` — check for small RAW variants
- `is_coolscan_nef()` — check for Nikon Coolscan format
- `is_jpeg_thumb()` — check thumbnail format
- `is_floating_point()` / `have_fpdata()` — floating-point RAW detection
- `unpack_function_name()` — name of the decoder used
- `get_decoder_info()` — decoder details

## 9. Output Image

### 9.1 ProcessedImage

```rust
pub struct ProcessedImage {
    pub(crate) inner: *mut sys::libraw_processed_image_t,
}
```

- `image_type()` — JPEG or Bitmap
- `data()` — `&[u8]` slice over pixel or JPEG data
- `width()`, `height()`, `colors()`, `bits()` — bitmap metadata
- `save(path)` — write to file
- `Drop` — calls `libraw_dcraw_clear_mem()`
- `Send` but not `Clone`

## 10. High-Level Facade

Convenience methods on `LibRaw` for the most common workflows:

```rust
LibRaw::open(path)              // new + open_file
LibRaw::open_from_buffer(buf)   // new + open_buffer
.process_default()              // set default params, unpack, dcraw_process
.process_with(&params)          // set custom params, unpack, dcraw_process
.to_image()                     // make_mem_image shortcut
.convert_to_tiff(input, output) // static: open + process + save_tiff
```

## 11. Callbacks

LibRaw supports four callback types. Each uses a C function pointer + `void *data` pattern.

### 11.1 Callback Types

| Callback | C Type | Purpose |
|---|---|---|
| Exif parser | `exif_parser_callback` | Custom tag processing during EXIF parsing |
| Makernotes parser | `exif_parser_callback` | Custom makernote tag processing |
| Data error | `data_callback` | Called on corrupted data (can skip/bypass) |
| Progress | `progress_callback` | Progress reporting (stage, iteration, total) |

### 11.2 Rust Wrapping

Each callback is registered via a setter method on `LibRaw`:

```rust
impl LibRaw {
    pub fn set_exifparser_handler<F>(&mut self, callback: F)
        where F: FnMut(&[u8], u32) -> () + 'static;

    pub fn set_makernotes_handler<F>(&mut self, callback: F)
        where F: FnMut(&[u8], u32) -> () + 'static;

    pub fn set_dataerror_handler<F>(&mut self, callback: F)
        where F: FnMut(&[u8], usize) -> DataErrorAction + 'static;

    pub fn set_progress_handler<F>(&mut self, callback: F)
        where F: FnMut(ProgressStage, u32, u32) -> ProgressAction + 'static;
}
```

### 11.3 FFI Trampoline

Internally, the Rust closure is boxed as `Box<dyn FnMut(...)>`, converted to a raw pointer, and stored as `void *data`. A single `unsafe extern "C"` trampoline function per callback type casts `data` back to the closure and invokes it. The closure is leaked until `LibRaw` is dropped, at which point it is reconstructed into a `Box` and freed.

### 11.4 Safety Constraints

- Each handler can only be set once (re-setting replaces and drops the previous closure)
- Closures are dropped when `LibRaw` is dropped or recycled
- The `'static` bound prevents borrowing local variables without explicit synchronization

## 12. Testing Strategy

- **Unit tests:** error conversion, parameter builder defaults, metadata accessor correctness
- **Integration tests:** read real RAW files from various camera vendors (placed in `tests/data/`)
- **Safety tests:** verify `Drop` is called correctly, no double-free, no use-after-free via lifetime enforcement
- **Feature-gate tests:** verify builds succeed with various feature combinations

## 13. Dependencies

| Crate | Purpose |
|---|---|
| `libc` | FFI primitive types |
| `thiserror` | Derive `Error` trait |
| `bindgen` (build) | Generate FFI bindings from C headers |
| `cmake` (build) | Build LibRaw from vendor source |
| `cc` (build) | Fallback for small C compilation needs |
| `pkg-config` (build) | Optional: detect system LibRaw |
| `tempfile` (dev) | Temporary test files |

## 14. License

LibRaw is dual-licensed under LGPL 2.1 and CDDL 1.0. The Rust bindings (`libraw-rs`) are licensed under MIT OR Apache-2.0. Users linking against LibRaw must comply with one of LibRaw's licenses.
