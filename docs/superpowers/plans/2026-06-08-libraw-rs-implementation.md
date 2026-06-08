# libraw-rs Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Build idiomatic Rust bindings for LibRaw v0.22.1 covering the full C API with a high-level facade and expert-level access.

**Architecture:** Single crate `libraw-rs`. `build.rs` uses cmake+bindgen to build LibRaw from vendor source and generate FFI bindings. Safe Rust modules (`context`, `error`, `params`, `metadata`, `process`, `image`, `stream`, `callbacks`, `high_level`) wrap the generated `sys` module, providing RAII lifecycle management, `Result`-based error handling, builder-pattern params, lifetime-bound metadata accessors, and a convenience facade.

**Tech Stack:** Rust 2024 edition, bindgen 0.70+, cmake 0.1, cc 1.0, libc 0.2, thiserror 1.0

---

### Task 1: Build system — cmake + bindgen + Cargo features

**Files:**
- Modify: `build.rs`
- Modify: `Cargo.toml`

- [ ] **Step 1: Add build.rs with cmake and basic bindgen setup**

Replace the empty `build.rs`:

```rust
use std::env;
use std::path::PathBuf;

fn main() {
    // Step 1: Build LibRaw via CMake
    let mut cmake = cmake::Config::new("vendor/LibRaw");

    // Static library
    cmake.define("BUILD_SHARED_LIBS", "OFF");

    // Disable examples and tools
    cmake.define("ENABLE_EXAMPLES", "OFF");
    cmake.define("ENABLE_TOOLS", "OFF");

    // LCMS support
    if cfg!(feature = "lcms") {
        cmake.define("ENABLE_LCMS", "ON");
    } else {
        cmake.define("ENABLE_LCMS", "OFF");
    }

    // RawSpeed support
    if cfg!(feature = "rawspeed") {
        cmake.define("ENABLE_RAWSPEED", "ON");
    } else {
        cmake.define("ENABLE_RAWSPEED", "OFF");
    }

    // DNG SDK (off by default)
    if cfg!(feature = "dng-sdk") {
        cmake.define("ENABLE_DNGSDK", "ON");
    } else {
        cmake.define("ENABLE_DNGSDK", "OFF");
    }

    // OpenMP
    if cfg!(feature = "openmp") {
        cmake.define("ENABLE_OPENMP", "ON");
    } else {
        cmake.define("ENABLE_OPENMP", "OFF");
    }

    // JPEG support for thumbnails
    if cfg!(feature = "jpeg") {
        cmake.define("USE_JPEG", "ON");
    } else {
        cmake.define("USE_JPEG", "OFF");
    }

    // Jasper (JPEG 2000)
    if cfg!(feature = "jasper") {
        cmake.define("USE_JASPER", "ON");
    } else {
        cmake.define("USE_JASPER", "OFF");
    }

    // Demosaic packs
    if cfg!(feature = "demosaic-packs") {
        cmake.define("USE_DEMOSAIC_PACK_GPL3", "ON");
        cmake.define("USE_DEMOSAIC_PACK_GPL2", "ON");
        cmake.define("USE_DEMOSAIC_PACK_LGPL2", "ON");
    }

    let dst = cmake
        .profile("Release")
        .build();

    // Link LibRaw
    println!("cargo:rustc-link-search=native={}/lib", dst.display());
    println!("cargo:rustc-link-lib=static=raw");

    // On Unix, LibRaw may need additional system libraries
    #[cfg(target_os = "linux")]
    {
        println!("cargo:rustc-link-lib=stdc++");
        println!("cargo:rustc-link-lib=z");
        if cfg!(feature = "lcms") {
            println!("cargo:rustc-link-lib=lcms2");
        }
        if cfg!(feature = "jpeg") {
            println!("cargo:rustc-link-lib=jpeg");
        }
    }

    #[cfg(target_os = "macos")]
    {
        println!("cargo:rustc-link-lib=c++");
        println!("cargo:rustc-link-lib=z");
    }

    #[cfg(target_os = "windows")]
    {
        println!("cargo:rustc-link-lib=user32");
    }

    // Step 2: Generate bindings via bindgen
    let bindings = bindgen::Builder::default()
        .header("vendor/LibRaw/libraw/libraw.h")
        .clang_arg("-Ivendor/LibRaw")
        .allowlist_function("libraw_.*")
        .allowlist_type("libraw_.*")
        .allowlist_type("LibRaw_.*")
        .allowlist_var("LIBRAW_.*")
        .opaque_type("std::.*")
        .size_t_is_usize(true)
        .generate()
        .expect("Unable to generate bindings");

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_dir.join("bindings.rs"))
        .expect("Couldn't write bindings!");

    // Rerun if vendor headers change
    println!("cargo:rerun-if-changed=vendor/LibRaw/libraw/");
}
```

- [ ] **Step 2: Update Cargo.toml features to match build.rs**

Add to `Cargo.toml` (keep existing contents, update `[features]`):

```toml
[features]
default = []
rawspeed = []
lcms = []
dng-sdk = []
demosaic-packs = []
openmp = []
jasper = []
jpeg = []
```

- [ ] **Step 3: Try a dry-run cargo build to verify CMake and bindgen run**

Run: `cargo build 2>&1 | head -40`
Expected: Should see CMake configure/build output, then bindgen output, then a Rust compilation error (modules not written yet). The important thing is that CMake + bindgen complete without error.

- [ ] **Step 4: Commit**

```bash
git add build.rs Cargo.toml
git commit -m "build: add cmake + bindgen build system with feature flags"
```

---

### Task 2: sys module and lib.rs skeleton

**Files:**
- Create: `src/sys.rs`
- Create: `src/lib.rs`

- [ ] **Step 1: Create sys.rs to include generated bindings**

```rust
// src/sys.rs
#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(dead_code)]
#![allow(clippy::all)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));
```

- [ ] **Step 2: Create lib.rs with module declarations**

```rust
// src/lib.rs
mod sys;
pub mod error;
pub mod context;
pub mod params;
pub mod metadata;
pub mod image;
pub mod process;
pub mod stream;
pub mod callbacks;
mod high_level;

pub use context::LibRaw;
pub use error::{Error, Result};
pub use image::ProcessedImage;
pub use params::OutputParams;
pub use high_level::*;
```

- [ ] **Step 3: Verify it compiles**

Run: `cargo build`
Expected: Should compile successfully (many dead_code warnings for sys, that's fine).

- [ ] **Step 4: Commit**

```bash
git add src/sys.rs src/lib.rs
git commit -m "feat: add sys module and lib.rs skeleton"
```

---

### Task 3: Error handling

**Files:**
- Create: `src/error.rs`

- [ ] **Step 1: Write error module with enum and conversion logic**

```rust
// src/error.rs
use std::ffi::CStr;
use thiserror::Error;

use crate::sys;

/// All errors that can occur during LibRaw operations.
#[derive(Error, Debug)]
pub enum Error {
    #[error("Out of memory")]
    OutOfMemory,

    #[error("Unsupported file format")]
    UnsupportedFormat,

    #[error("I/O error: {0}")]
    Io(String),

    #[error("Corrupted data")]
    DataError,

    #[error("Unsupported feature: {0}")]
    UnsupportedFeature(String),

    #[error("Processing cancelled by callback")]
    Cancelled,

    #[error("Callback error")]
    CallbackError,

    #[error("Bad parameters: {0}")]
    BadParams(String),

    #[error("No thumbnail available")]
    NoThumbnail,

    #[error("Unsupported thumbnail format")]
    UnsupportedThumbnail,

    #[error("Input closed")]
    InputClosed,

    #[error("Not implemented")]
    NotImplemented,

    #[error("Out of order call — check processing pipeline order")]
    OutOfOrderCall,

    #[error("Bad crop parameters")]
    BadCrop,

    #[error("File too large to process")]
    TooBig,

    #[error("Memory pool overflow")]
    MempoolOverflow,

    #[error("LibRaw error {code}: {msg}")]
    Unknown { code: i32, msg: String },
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    /// Convert a raw LibRaw error code into an Error.
    /// Panics if `code == 0` (zero is not an error).
    pub(crate) fn from_raw(code: i32) -> Self {
        debug_assert_ne!(code, 0, "zero is not an error code");

        let msg = unsafe {
            let ptr = sys::libraw_strerror(code);
            if ptr.is_null() {
                String::from("unknown error")
            } else {
                CStr::from_ptr(ptr)
                    .to_string_lossy()
                    .into_owned()
            }
        };

        match code {
            -1  => Error::UnsupportedFeature(msg),
            -2  => Error::UnsupportedFormat,
            -3  => Error::UnsupportedFeature(msg),
            -4  => Error::OutOfOrderCall,
            -5  => Error::NoThumbnail,
            -6  => Error::UnsupportedThumbnail,
            -7  => Error::InputClosed,
            -8  => Error::NotImplemented,
            -9  => Error::UnsupportedFeature(msg),
            -100007 => Error::OutOfMemory,
            -100008 => Error::DataError,
            -100009 => Error::Io(msg),
            -100010 => Error::Cancelled,
            -100011 => Error::BadCrop,
            -100012 => Error::TooBig,
            -100013 => Error::MempoolOverflow,
            _ => Error::Unknown { code, msg },
        }
    }

    /// Check raw error code, return Ok(()) for LIBRAW_SUCCESS=0, Err otherwise.
    #[inline]
    pub(crate) fn check(code: i32) -> Result<()> {
        if code == 0 {
            Ok(())
        } else {
            Err(Error::from_raw(code))
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::Io(e.to_string())
    }
}

impl From<std::ffi::NulError> for Error {
    fn from(_: std::ffi::NulError) -> Self {
        Error::BadParams("path contains null byte".into())
    }
}
```

- [ ] **Step 2: Add unit test for known error codes**

Add to `src/error.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_raw_success() {
        assert!(Error::check(0).is_ok());
    }

    #[test]
    fn test_maps_unsupported_format() {
        let err = Error::from_raw(-2);
        assert!(matches!(err, Error::UnsupportedFormat));
    }

    #[test]
    fn test_maps_out_of_memory() {
        let err = Error::from_raw(-100007);
        assert!(matches!(err, Error::OutOfMemory));
    }

    #[test]
    fn test_maps_data_error() {
        let err = Error::from_raw(-100008);
        assert!(matches!(err, Error::DataError));
    }

    #[test]
    fn test_maps_cancelled() {
        let err = Error::from_raw(-100010);
        assert!(matches!(err, Error::Cancelled));
    }

    #[test]
    fn test_unknown_error_code() {
        let err = Error::from_raw(-999999);
        assert!(matches!(err, Error::Unknown { .. }));
    }

    #[test]
    fn test_display_contains_message() {
        let err = Error::BadParams("test".into());
        let s = err.to_string();
        assert!(s.contains("test"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "no file");
        let err: Error = io_err.into();
        assert!(matches!(err, Error::Io(_)));
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p libraw-rs --lib error`
Expected: All 8 tests pass.

- [ ] **Step 4: Commit**

```bash
git add src/error.rs
git commit -m "feat: add error types mapping LibRaw error codes"
```

---

### Task 4: LibRaw context — creation, open_file, Drop

**Files:**
- Create: `src/context.rs`

- [ ] **Step 1: Write context module**

```rust
// src/context.rs
use std::ffi::CString;
use std::path::Path;

use crate::error::{Error, Result};
use crate::sys;

/// LibRaw processing context.
///
/// Owns a `libraw_data_t*`. Created via `LibRaw::new()` and automatically
/// freed on drop via `libraw_close()`.
///
/// # Safety
///
/// Implements `Send` but not `Sync` — usable across threads but not
/// shared concurrently.
pub struct LibRaw {
    pub(crate) inner: *mut sys::libraw_data_t,
}

// SAFETY: libraw_data_t is owned exclusively. Transferring ownership
// between threads is safe (no thread-local state survives recycle).
unsafe impl Send for LibRaw {}

impl LibRaw {
    /// Create a new LibRaw context with no special flags.
    /// Calls `libraw_init(0)`.
    pub fn new() -> Result<Self> {
        Self::with_flags(0)
    }

    /// Create a new LibRaw context with the given init flags.
    /// Valid flags include `LIBRAW_OPTIONS_NONE` (0).
    pub fn with_flags(flags: u32) -> Result<Self> {
        let inner = unsafe { sys::libraw_init(flags) };
        if inner.is_null() {
            return Err(Error::OutOfMemory);
        }
        Ok(Self { inner })
    }

    /// Open a RAW file from the given path.
    /// After this call, metadata is available via `image_info()`, `lens_info()`, etc.
    /// Calls `libraw_open_file()`.
    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        let code = unsafe { sys::libraw_open_file(self.inner, c_path.as_ptr()) };
        Error::check(code)
    }

    /// Free all internal image data but keep the context alive for reuse.
    /// Calls `libraw_recycle()`.
    pub fn recycle(&mut self) {
        unsafe { sys::libraw_recycle(self.inner) };
    }

    /// Free only the datastream, keeping image data.
    /// Calls `libraw_recycle_datastream()`.
    pub fn recycle_datastream(&mut self) {
        unsafe { sys::libraw_recycle_datastream(self.inner) };
    }
}

impl Drop for LibRaw {
    fn drop(&mut self) {
        unsafe { sys::libraw_close(self.inner) };
    }
}
```

- [ ] **Step 2: Write unit test — context creation and drop doesn't crash**

```rust
// Add to src/context.rs:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_context() {
        let raw = LibRaw::new();
        assert!(raw.is_ok());
        let raw = raw.unwrap();
        assert!(!raw.inner.is_null());
        // Drop runs here — should not crash
    }

    #[test]
    fn test_with_flags_zero_same_as_new() {
        let raw = LibRaw::with_flags(0);
        assert!(raw.is_ok());
    }

    #[test]
    fn test_libraw_version() {
        let ver = unsafe {
            let ptr = sys::libraw_version();
            std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
        };
        assert!(ver.starts_with("0.22"));
    }

    #[test]
    fn test_libraw_version_number() {
        let n = unsafe { sys::libraw_versionNumber() };
        assert!(n > 0);
        // LIBRAW_VERSION for 0.22.1 = (0 << 16) | (22 << 8) | 1
        assert_eq!(n, (0 << 16) | (22 << 8) | 1);
    }
}
```

- [ ] **Step 3: Run tests**

Run: `cargo test -p libraw-rs --lib context`
Expected: 4 tests pass (context creates, version checks out).

- [ ] **Step 4: Commit**

```bash
git add src/context.rs
git commit -m "feat: add LibRaw context with open_file, Drop, recycle"
```

---

### Task 5: OutputParams builder

**Files:**
- Create: `src/params.rs`

- [ ] **Step 1: Write params module with builder and enums**

```rust
// src/params.rs
use crate::sys;

/// Output color space.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColor {
    Raw = 0,
    sRGB = 1,
    AdobeRGB = 2,
    WideGamut = 3,
    ProPhoto = 4,
    XYZ = 5,
}

/// Highlight recovery mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightMode {
    Clip = 0,
    Unclip = 1,
    Blend = 2,
    Rebuild = 3,
}

/// Builder for `libraw_output_params_t`.
///
/// Defaults match LibRaw's C++ constructor defaults.
#[derive(Debug, Clone)]
pub struct OutputParams {
    pub(crate) inner: sys::libraw_output_params_t,
}

impl Default for OutputParams {
    fn default() -> Self {
        let mut inner = unsafe { std::mem::zeroed::<sys::libraw_output_params_t>() };
        // LibRaw C++ defaults (from LibRaw constructor):
        inner.greybox = [0, 0, sys::LIBRAW_rect_raw_WIDTH as u32, 0];
        inner.aber = [0.0; 4];
        inner.user_mul = [0.0; 4];
        inner.shot_select = 0;
        inner.bright = 1.0;
        inner.threshold = 0.0;
        inner.half_size = 0;
        inner.four_color_rgb = 0;
        inner.highlight = 0;
        inner.use_auto_wb = 0;
        inner.use_camera_wb = 0;
        inner.use_camera_matrix = 1;
        inner.output_color = 1; // sRGB
        inner.output_bps = 8;
        inner.output_tiff = 0;
        inner.output_flags = 0;
        inner.user_flip = -1;
        inner.user_qual = -1;
        inner.user_black = -1;
        inner.user_cblack = [-1; 4];
        inner.user_sat = -1;
        inner.med_passes = 0;
        inner.auto_bright_thr = sys::LIBRAW_DEFAULT_AUTO_BRIGHTNESS_THRESHOLD as f32;
        inner.adjust_maximum_thr = sys::LIBRAW_DEFAULT_ADJUST_MAXIMUM_THRESHOLD as f32;
        inner.no_auto_bright = 0;
        inner.use_fuji_rotate = -1;
        inner.use_p1_correction = 1;
        inner.green_matching = 0;
        inner.dcb_iterations = -1;
        inner.dcb_enhance_fl = 1;
        inner.fbdd_noiserd = 0;
        inner.exp_correc = 1;
        inner.exp_shift = 1.0;
        inner.exp_preser = 0.0;
        inner.no_auto_scale = 0;
        inner.no_interpolation = 0;
        // profile strings default to null
        inner.output_profile = std::ptr::null_mut();
        inner.camera_profile = std::ptr::null_mut();
        inner.bad_pixels = std::ptr::null_mut();
        inner.dark_frame = std::ptr::null_mut();
        Self { inner }
    }
}

impl OutputParams {
    /// Set output color space.
    pub fn output_color(mut self, color: OutputColor) -> Self {
        self.inner.output_color = color as i32;
        self
    }

    /// Set output bits per sample (8 or 16).
    pub fn output_bps(mut self, bps: u8) -> Self {
        self.inner.output_bps = bps as i32;
        self
    }

    /// Set highlight recovery mode.
    pub fn highlight_mode(mut self, mode: HighlightMode) -> Self {
        self.inner.highlight = mode as i32;
        self
    }

    /// Set user white balance multipliers (0=R, 1=G1, 2=B, 3=G2).
    pub fn user_mul(mut self, index: usize, value: f32) -> Self {
        assert!(index < 4, "mul index must be 0..3");
        self.inner.user_mul[index] = value;
        self
    }

    /// Use automatic white balance.
    pub fn use_auto_wb(mut self, on: bool) -> Self {
        self.inner.use_auto_wb = on as i32;
        self
    }

    /// Use camera white balance from metadata.
    pub fn use_camera_wb(mut self, on: bool) -> Self {
        self.inner.use_camera_wb = on as i32;
        self
    }

    /// Use camera color matrix.
    pub fn use_camera_matrix(mut self, on: bool) -> Self {
        self.inner.use_camera_matrix = on as i32;
        self
    }

    /// Set brightness multiplier (default 1.0).
    pub fn brightness(mut self, bright: f32) -> Self {
        self.inner.bright = bright;
        self
    }

    /// Disable automatic brightness adjustment.
    pub fn no_auto_bright(mut self, on: bool) -> Self {
        self.inner.no_auto_bright = on as i32;
        self
    }

    /// Set auto brightness threshold.
    pub fn auto_bright_thr(mut self, thr: f32) -> Self {
        self.inner.auto_bright_thr = thr;
        self
    }

    /// Set adjust maximum threshold.
    pub fn adjust_maximum_thr(mut self, thr: f32) -> Self {
        self.inner.adjust_maximum_thr = thr;
        self
    }

    /// Set gamma curve: (power, toelinear_slope).
    pub fn gamma(mut self, power: f64, slope: f64) -> Self {
        self.inner.gamm = [power, slope, 0.0, 0.0, 0.0, 0.0];
        self
    }

    /// Use Fuji rotate.
    pub fn use_fuji_rotate(mut self, on: bool) -> Self {
        self.inner.use_fuji_rotate = on as i32;
        self
    }

    /// Set output to half size.
    pub fn half_size(mut self, on: bool) -> Self {
        self.inner.half_size = on as i32;
        self
    }

    /// Output TIFF instead of PPM.
    pub fn output_tiff(mut self, on: bool) -> Self {
        self.inner.output_tiff = on as i32;
        self
    }

    /// Set user flip (-1 = use EXIF, 0 = none, 1..6 = various rotations).
    pub fn user_flip(mut self, flip: i32) -> Self {
        self.inner.user_flip = flip;
        self
    }

    /// Enable green matching.
    pub fn green_matching(mut self, on: bool) -> Self {
        self.inner.green_matching = on as i32;
        self
    }

    /// Set DCB iterations (-1 = default, 0 = off).
    pub fn dcb_iterations(mut self, n: i32) -> Self {
        self.inner.dcb_iterations = n;
        self
    }

    /// Set DCB enhance flag.
    pub fn dcb_enhance(mut self, on: bool) -> Self {
        self.inner.dcb_enhance_fl = on as i32;
        self
    }

    /// Set FBDD noise reduction.
    pub fn fbdd_noiserd(mut self, value: i32) -> Self {
        self.inner.fbdd_noiserd = value;
        self
    }

    /// Set exposure correction before interpolation.
    pub fn exp_correc(mut self, on: bool) -> Self {
        self.inner.exp_correc = on as i32;
        self
    }

    /// Set exposure shift.
    pub fn exp_shift(mut self, shift: f32) -> Self {
        self.inner.exp_shift = shift;
        self
    }

    /// Set exposure preservation factor.
    pub fn exp_preser(mut self, preser: f32) -> Self {
        self.inner.exp_preser = preser;
        self
    }

    /// Disable auto-scaling.
    pub fn no_auto_scale(mut self, on: bool) -> Self {
        self.inner.no_auto_scale = on as i32;
        self
    }

    /// Disable interpolation.
    pub fn no_interpolation(mut self, on: bool) -> Self {
        self.inner.no_interpolation = on as i32;
        self
    }
}
```

- [ ] **Step 2: Update LibRaw to accept params**

Add to `src/context.rs` after the `recycle_datastream` method:

```rust
    /// Apply output parameters to this context.
    pub fn set_params(&mut self, params: &crate::params::OutputParams) {
        unsafe {
            (*self.inner).params = params.inner;
        }
    }

    /// Get a mutable reference to the raw output parameters struct
    /// for direct manipulation.
    pub fn params_mut(&mut self) -> &mut sys::libraw_output_params_t {
        unsafe { &mut (*self.inner).params }
    }
```

- [ ] **Step 3: Write unit test for default params and chain building**

```rust
// Add to src/params.rs:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params_are_valid() {
        let p = OutputParams::default();
        assert_eq!(p.inner.output_color, 1); // sRGB
        assert_eq!(p.inner.output_bps, 8);
        assert_eq!(p.inner.bright, 1.0);
    }

    #[test]
    fn test_builder_chain() {
        let p = OutputParams::default()
            .output_color(OutputColor::AdobeRGB)
            .output_bps(16)
            .highlight_mode(HighlightMode::Blend)
            .use_camera_wb(true)
            .brightness(1.2)
            .half_size(true);
        assert_eq!(p.inner.output_color, 2); // AdobeRGB
        assert_eq!(p.inner.output_bps, 16);
        assert_eq!(p.inner.highlight, 2); // Blend
        assert_eq!(p.inner.use_camera_wb, 1);
        assert_eq!(p.inner.bright, 1.2);
        assert_eq!(p.inner.half_size, 1);
    }

    #[test]
    fn test_user_mul() {
        let p = OutputParams::default()
            .user_mul(0, 2.1)
            .user_mul(1, 1.0)
            .user_mul(2, 1.5)
            .user_mul(3, 1.0);
        assert_eq!(p.inner.user_mul[0], 2.1);
        assert_eq!(p.inner.user_mul[1], 1.0);
        assert_eq!(p.inner.user_mul[2], 1.5);
        assert_eq!(p.inner.user_mul[3], 1.0);
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p libraw-rs --lib params`
Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/params.rs src/context.rs
git commit -m "feat: add OutputParams builder with 25+ chainable setters"
```

---

### Task 6: Metadata accessors

**Files:**
- Create: `src/metadata.rs`

- [ ] **Step 1: Write metadata accessor types**

```rust
// src/metadata.rs
use crate::sys;

/// C-string helper: convert `[u8; N]` to `&str`, stopping at first null.
fn cstr_to_str(bytes: &[u8]) -> &str {
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end]).unwrap_or("")
}

/// Read-only access to image identification params.
/// Available after `open_file()`.
pub struct ImageInfo<'a> {
    pub(crate) inner: &'a sys::libraw_iparams_t,
}

impl<'a> ImageInfo<'a> {
    pub fn make(&self) -> &str           { cstr_to_str(&self.inner.make) }
    pub fn model(&self) -> &str          { cstr_to_str(&self.inner.model) }
    pub fn software(&self) -> &str       { cstr_to_str(&self.inner.software) }
    pub fn normalized_make(&self) -> &str { cstr_to_str(&self.inner.normalized_make) }
    pub fn normalized_model(&self) -> &str { cstr_to_str(&self.inner.normalized_model) }
    pub fn maker_index(&self) -> u32     { self.inner.maker_index }
    pub fn raw_count(&self) -> u32       { self.inner.raw_count }
    pub fn dng_version(&self) -> u32     { self.inner.dng_version }
    pub fn is_foveon(&self) -> bool      { self.inner.is_foveon != 0 }
    pub fn colors(&self) -> i32          { self.inner.colors }
    pub fn filters(&self) -> u32         { self.inner.filters }
    pub fn cdesc(&self) -> &str          { cstr_to_str(&self.inner.cdesc) }
}

/// Read-only access to lens information.
pub struct LensInfo<'a> {
    pub(crate) inner: &'a sys::libraw_lensinfo_t,
}

impl<'a> LensInfo<'a> {
    pub fn min_focal(&self) -> f32           { self.inner.MinFocal }
    pub fn max_focal(&self) -> f32           { self.inner.MaxFocal }
    pub fn max_ap4_min_focal(&self) -> f32   { self.inner.MaxAp4MinFocal }
    pub fn max_ap4_max_focal(&self) -> f32   { self.inner.MaxAp4MaxFocal }
    pub fn exif_max_ap(&self) -> f32         { self.inner.EXIF_MaxAp }
    pub fn make(&self) -> &str               { cstr_to_str(&self.inner.LensMake) }
    pub fn model(&self) -> &str              { cstr_to_str(&self.inner.Lens) }
    pub fn serial(&self) -> &str             { cstr_to_str(&self.inner.LensSerial) }
    pub fn focal_length_in_35mm(&self) -> u16 { self.inner.FocalLengthIn35mmFormat }
}

/// Read-only access to image other metadata.
pub struct ImageOther<'a> {
    pub(crate) inner: &'a sys::libraw_imgother_t,
}

impl<'a> ImageOther<'a> {
    pub fn iso_speed(&self) -> f32   { self.inner.iso_speed }
    pub fn shutter(&self) -> f32     { self.inner.shutter }
    pub fn aperture(&self) -> f32    { self.inner.aperture }
    pub fn focal_len(&self) -> f32   { self.inner.focal_len }
    pub fn timestamp(&self) -> i64 {
        // time_t can be i32 or i64 depending on platform
        #[allow(clippy::useless_conversion)]
        self.inner.timestamp as i64
    }
    pub fn shot_order(&self) -> u32  { self.inner.shot_order }
    pub fn desc(&self) -> &str       { cstr_to_str(&self.inner.desc) }
    pub fn artist(&self) -> &str     { cstr_to_str(&self.inner.artist) }
}

/// Read-only access to image sizes.
pub struct ImageSizes<'a> {
    pub(crate) inner: &'a sys::libraw_image_sizes_t,
}

impl<'a> ImageSizes<'a> {
    pub fn raw_width(&self) -> u16   { self.inner.raw_width }
    pub fn raw_height(&self) -> u16  { self.inner.raw_height }
    pub fn width(&self) -> u16       { self.inner.width }
    pub fn height(&self) -> u16      { self.inner.height }
    pub fn top_margin(&self) -> u16  { self.inner.top_margin }
    pub fn left_margin(&self) -> u16 { self.inner.left_margin }
    pub fn iwidth(&self) -> u16      { self.inner.iwidth }
    pub fn iheight(&self) -> u16     { self.inner.iheight }
    pub fn pixel_aspect(&self) -> f64 { self.inner.pixel_aspect }
    pub fn flip(&self) -> i32        { self.inner.flip }
}
```

- [ ] **Step 2: Add metadata accessors to LibRaw**

Add to `src/context.rs` after the `params_mut` method:

```rust
    /// Get image identification params (requires `open_file`).
    pub fn image_info(&self) -> crate::metadata::ImageInfo<'_> {
        crate::metadata::ImageInfo {
            inner: unsafe { &(*self.inner).idata },
        }
    }

    /// Get lens info (requires `open_file`).
    pub fn lens_info(&self) -> crate::metadata::LensInfo<'_> {
        crate::metadata::LensInfo {
            inner: unsafe { &(*self.inner).lens },
        }
    }

    /// Get other image metadata (requires `open_file`).
    pub fn image_other(&self) -> crate::metadata::ImageOther<'_> {
        crate::metadata::ImageOther {
            inner: unsafe { &(*self.inner).other },
        }
    }

    /// Get image sizes (requires `open_file`).
    pub fn sizes(&self) -> crate::metadata::ImageSizes<'_> {
        crate::metadata::ImageSizes {
            inner: unsafe { &(*self.inner).sizes },
        }
    }
```

- [ ] **Step 3: Write unit test using the C helper functions directly**

```rust
// Add to src/metadata.rs:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_str_normal() {
        let buf = [b'h', b'e', b'l', b'l', b'o', 0, b'x', b'x'];
        assert_eq!(cstr_to_str(&buf), "hello");
    }

    #[test]
    fn test_cstr_to_str_no_null() {
        let buf = [b'h', b'i'];
        assert_eq!(cstr_to_str(&buf), "hi");
    }

    #[test]
    fn test_cstr_to_str_empty() {
        let buf = [0u8; 10];
        assert_eq!(cstr_to_str(&buf), "");
    }
}
```

- [ ] **Step 4: Run tests**

Run: `cargo test -p libraw-rs --lib metadata`
Expected: 3 tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/metadata.rs src/context.rs
git commit -m "feat: add metadata accessors (ImageInfo, LensInfo, ImageOther, ImageSizes)"
```

---

### Task 7: Processing pipeline (expert layer)

**Files:**
- Create: `src/process.rs`
- Modify: `src/context.rs`

- [ ] **Step 1: Write processing module**

```rust
// src/process.rs
use std::ffi::CString;
use std::path::Path;

use crate::error::{Error, Result};
use crate::sys;
use crate::LibRaw;

impl LibRaw {
    /// Open a RAW file from a memory buffer.
    /// Calls `libraw_open_buffer()`.
    pub fn open_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        let code = unsafe {
            sys::libraw_open_buffer(
                self.inner,
                buffer.as_ptr() as *const std::ffi::c_void,
                buffer.len(),
            )
        };
        Error::check(code)
    }

    /// Unpack RAW data (decode pixel data from file format).
    /// Calls `libraw_unpack()`. Must be called after `open_file`/`open_buffer`.
    pub fn unpack(&mut self) -> Result<()> {
        let code = unsafe { sys::libraw_unpack(self.inner) };
        Error::check(code)
    }

    /// Unpack only the thumbnail.
    /// Calls `libraw_unpack_thumb()`.
    pub fn unpack_thumb(&mut self) -> Result<()> {
        let code = unsafe { sys::libraw_unpack_thumb(self.inner) };
        Error::check(code)
    }

    /// Unpack a specific thumbnail (multi-thumbnail formats).
    /// Calls `libraw_unpack_thumb_ex()`.
    pub fn unpack_thumb_ex(&mut self, index: i32) -> Result<()> {
        let code = unsafe { sys::libraw_unpack_thumb_ex(self.inner, index) };
        Error::check(code)
    }

    /// Subtract black level from the raw data.
    /// Calls `libraw_subtract_black()`.
    pub fn subtract_black(&mut self) {
        unsafe { sys::libraw_subtract_black(self.inner) };
    }

    /// Convert the unpacked encoded RAW data into an accessible pixel array.
    /// Calls `libraw_raw2image()`.
    pub fn raw2image(&mut self) -> Result<()> {
        let code = unsafe { sys::libraw_raw2image(self.inner) };
        Error::check(code)
    }

    /// Free the image data allocated by `raw2image()`.
    /// Calls `libraw_free_image()`.
    pub fn free_image(&mut self) {
        unsafe { sys::libraw_free_image(self.inner) };
    }

    /// Run the full dcraw-compatible processing pipeline.
    /// Must be called after `unpack()`.
    /// Calls `libraw_dcraw_process()`.
    pub fn dcraw_process(&mut self) -> Result<()> {
        let code = unsafe { sys::libraw_dcraw_process(self.inner) };
        Error::check(code)
    }

    /// Produce an in-memory processed image.
    /// Calls `libraw_dcraw_make_mem_image()`.
    pub fn make_mem_image(&mut self) -> Result<crate::image::ProcessedImage> {
        let mut err: i32 = 0;
        let ptr = unsafe { sys::libraw_dcraw_make_mem_image(self.inner, &mut err) };
        if ptr.is_null() {
            return Err(if err != 0 {
                Error::from_raw(err)
            } else {
                Error::Unknown {
                    code: -1,
                    msg: "dcraw_make_mem_image returned null".into(),
                }
            });
        }
        Ok(crate::image::ProcessedImage { inner: ptr })
    }

    /// Produce an in-memory thumbnail image.
    /// Calls `libraw_dcraw_make_mem_thumb()`.
    pub fn make_mem_thumb(&mut self) -> Result<crate::image::ProcessedImage> {
        let mut err: i32 = 0;
        let ptr = unsafe { sys::libraw_dcraw_make_mem_thumb(self.inner, &mut err) };
        if ptr.is_null() {
            return Err(if err != 0 {
                Error::from_raw(err)
            } else {
                Error::Unknown {
                    code: -1,
                    msg: "dcraw_make_mem_thumb returned null".into(),
                }
            });
        }
        Ok(crate::image::ProcessedImage { inner: ptr })
    }

    /// Save processed image as TIFF/PPM to the given path.
    /// Calls `libraw_dcraw_ppm_tiff_writer()`.
    pub fn save_tiff<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        let code =
            unsafe { sys::libraw_dcraw_ppm_tiff_writer(self.inner, c_path.as_ptr()) };
        Error::check(code)
    }

    /// Save thumbnail to the given path.
    /// Calls `libraw_dcraw_thumb_writer()`.
    pub fn save_thumb<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        let code =
            unsafe { sys::libraw_dcraw_thumb_writer(self.inner, c_path.as_ptr()) };
        Error::check(code)
    }

    /// Adjust image sizes for raw inset crop (cropped sensors).
    /// Calls `libraw_adjust_to_raw_inset_crop()`.
    pub fn adjust_to_raw_inset_crop(&mut self, mask: u32, maxcrop: f32) -> Result<()> {
        let code = unsafe {
            sys::libraw_adjust_to_raw_inset_crop(self.inner, mask, maxcrop)
        };
        Error::check(code)
    }

    /// Get the name of the unpack function used for this file.
    /// Calls `libraw_unpack_function_name()`.
    pub fn unpack_function_name(&self) -> &str {
        unsafe {
            let ptr = sys::libraw_unpack_function_name(self.inner);
            if ptr.is_null() {
                return "";
            }
            std::ffi::CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    /// Get decoder info for the current file.
    /// Calls `libraw_get_decoder_info()`.
    pub fn decoder_info(&self) -> sys::libraw_decoder_info_t {
        let mut info = sys::libraw_decoder_info_t {
            decoder_name: std::ptr::null(),
            decoder_flags: 0,
        };
        unsafe {
            sys::libraw_get_decoder_info(self.inner, &mut info);
        }
        info
    }

    /// Adjust image sizes info only (no data load).
    /// Calls `libraw_adjust_sizes_info_only()`.
    pub fn adjust_sizes_info_only(&mut self) -> Result<()> {
        let code = unsafe { sys::libraw_adjust_sizes_info_only(self.inner) };
        Error::check(code)
    }

    // --- Helper queries ---

    /// Check if this is a Fuji image needing rotation.
    pub fn is_fuji_rotated(&self) -> bool {
        unsafe {
            (*self.inner).libraw_internal_data.internal_output_params.fuji_width != 0
        }
    }

    /// Check if this is a floating-point RAW.
    pub fn is_floating_point(&self) -> bool {
        unsafe { (*self.inner).idata.is_floating_point != 0 }
    }

    /// Check if floating-point data is available.
    pub fn have_fpdata(&self) -> bool {
        unsafe {
            !(*self.inner).rawdata.float_image.is_null()
                || !(*self.inner).rawdata.float3_image.is_null()
                || !(*self.inner).rawdata.float4_image.is_null()
        }
    }

    /// Check if the thumbnail is JPEG format.
    pub fn is_jpeg_thumb(&self) -> bool {
        unsafe {
            (*self.inner).thumbnail.tformat
                == sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG as i32
        }
    }
}
```

- [ ] **Step 2: Register `process` module in lib.rs**

Edit `src/lib.rs` — ensure `pub mod process;` is present (it should already be in the skeleton).

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles with ProcessedImage not yet defined — wait, it needs `image` module. Let's do a quick stub.

Create a minimal `src/image.rs` for compilation:

```rust
// src/image.rs
use crate::sys;

/// Processed image (in-memory result of dcraw processing).
pub struct ProcessedImage {
    pub(crate) inner: *mut sys::libraw_processed_image_t,
}
```

- [ ] **Step 4: Verify compilation**

Run: `cargo build`
Expected: Compiles. Dead code warnings are OK.

- [ ] **Step 5: Commit**

```bash
git add src/process.rs src/image.rs
git commit -m "feat: add processing pipeline (unpack, dcraw_process, make_mem_image, queries)"
```

---

### Task 8: ProcessedImage and stream support

**Files:**
- Replace: `src/image.rs`
- Create: `src/stream.rs`

- [ ] **Step 1: Write complete ProcessedImage type**

Replace `src/image.rs`:

```rust
// src/image.rs
use std::path::Path;

use crate::sys;

/// Image type returned by `make_mem_image` / `make_mem_thumb`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType {
    Jpeg,
    Bitmap,
}

/// Processed image in memory.
///
/// Owns a `libraw_processed_image_t*`. The underlying memory is freed
/// on drop via `libraw_dcraw_clear_mem()`.
pub struct ProcessedImage {
    pub(crate) inner: *mut sys::libraw_processed_image_t,
}

// SAFETY: Owns its data exclusively.
unsafe impl Send for ProcessedImage {}

impl ProcessedImage {
    /// Returns the type of image data (JPEG or raw bitmap).
    pub fn image_type(&self) -> ImageType {
        let t = unsafe { (*self.inner).type_ };
        if t == sys::LibRaw_image_formats_LIBRAW_IMAGE_JPEG as u32 {
            ImageType::Jpeg
        } else {
            ImageType::Bitmap
        }
    }

    /// Access the raw pixel data or JPEG byte stream.
    pub fn data(&self) -> &[u8] {
        let len = unsafe { (*self.inner).data_size as usize };
        unsafe { std::slice::from_raw_parts((*self.inner).data.as_ptr(), len) }
    }

    /// Image width in pixels (valid for bitmap type only).
    pub fn width(&self) -> u16 {
        unsafe { (*self.inner).width }
    }

    /// Image height in pixels (valid for bitmap type only).
    pub fn height(&self) -> u16 {
        unsafe { (*self.inner).height }
    }

    /// Number of color channels (valid for bitmap type only).
    pub fn colors(&self) -> u16 {
        unsafe { (*self.inner).colors }
    }

    /// Bits per sample (valid for bitmap type only).
    pub fn bits(&self) -> u16 {
        unsafe { (*self.inner).bits }
    }

    /// Save image data to a file.
    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::write(path, self.data())
    }
}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_dcraw_clear_mem(self.inner) };
    }
}
```

- [ ] **Step 2: Write stream module with open_bayer**

```rust
// src/stream.rs
use crate::error::{Error, Result};
use crate::sys;
use crate::LibRaw;

impl LibRaw {
    /// Open a raw Bayer pattern from raw sensor data.
    /// This is for data that has no file header — you must know the
    /// dimensions, margins, and Bayer pattern.
    #[allow(clippy::too_many_arguments)]
    pub fn open_bayer(
        &mut self,
        data: &[u8],
        raw_width: u16,
        raw_height: u16,
        left_margin: u16,
        top_margin: u16,
        right_margin: u16,
        bottom_margin: u16,
        procflags: u8,
        bayer_pattern: u8,
        unused_bits: u32,
        otherflags: u32,
        black_level: u32,
    ) -> Result<()> {
        let code = unsafe {
            sys::libraw_open_bayer(
                self.inner,
                data.as_ptr() as *mut u8,
                data.len() as u32,
                raw_width,
                raw_height,
                left_margin,
                top_margin,
                right_margin,
                bottom_margin,
                procflags,
                bayer_pattern,
                unused_bits,
                otherflags,
                black_level,
            )
        };
        Error::check(code)
    }
}
```

- [ ] **Step 3: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 4: Commit**

```bash
git add src/image.rs src/stream.rs
git commit -m "feat: add ProcessedImage with Drop and open_bayer stream support"
```

---

### Task 9: Callbacks

**Files:**
- Create: `src/callbacks.rs`

- [ ] **Step 1: Write callback module with working trampolines**

```rust
// src/callbacks.rs
use crate::sys;
use crate::LibRaw;

/// Action returned by a progress callback.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressAction {
    Continue = 0,
    Cancel = 1,
}

impl LibRaw {
    /// Set a custom EXIF parser callback.
    ///
    /// The callback receives all callback parameters from LibRaw's
    /// `exif_parser_callback`. This is an advanced API — see LibRaw docs
    /// for the meaning of each parameter.
    pub fn set_exifparser_handler<F>(&mut self, callback: F)
    where
        F: FnMut(i32, i32, i32, u32, i64) + Send + 'static,
    {
        unsafe extern "C" fn trampoline(
            context: *mut std::ffi::c_void,
            tag: i32,
            type_: i32,
            len: i32,
            ord: u32,
            _ifp: *mut std::ffi::c_void,
            base: i64,
        ) {
            let cb: &mut Box<dyn FnMut(i32, i32, i32, u32, i64) + Send> =
                &mut *(context as *mut _);
            cb(tag, type_, len, ord, base);
        }

        let boxed: Box<Box<dyn FnMut(i32, i32, i32, u32, i64) + Send>> =
            Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

        unsafe {
            (*self.inner).callbacks.exifparser_data = ptr;
            (*self.inner).callbacks.exif_cb = Some(std::mem::transmute(
                trampoline as usize as *const (),
            ));
        }
    }

    /// Set a custom makernotes parser callback.
    ///
    /// Same signature as EXIF parser callback but invoked for makernote tags.
    pub fn set_makernotes_handler<F>(&mut self, callback: F)
    where
        F: FnMut(i32, i32, i32, u32, i64) + Send + 'static,
    {
        unsafe extern "C" fn trampoline(
            context: *mut std::ffi::c_void,
            tag: i32,
            type_: i32,
            len: i32,
            ord: u32,
            _ifp: *mut std::ffi::c_void,
            base: i64,
        ) {
            let cb: &mut Box<dyn FnMut(i32, i32, i32, u32, i64) + Send> =
                &mut *(context as *mut _);
            cb(tag, type_, len, ord, base);
        }

        let boxed: Box<Box<dyn FnMut(i32, i32, i32, u32, i64) + Send>> =
            Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

        unsafe {
            (*self.inner).callbacks.makernotesparser_data = ptr;
            (*self.inner).callbacks.makernotes_cb = Some(std::mem::transmute(
                trampoline as usize as *const (),
            ));
        }
    }

    /// Set a custom data error handler.
    ///
    /// The callback receives `(file_name, offset)` when corrupted data
    /// is encountered. Note: LibRaw's data_callback returns void, so
    /// the callback cannot abort processing. Use a shared flag
    /// (e.g., `AtomicBool`) for cancellation if needed.
    pub fn set_dataerror_handler<F>(&mut self, callback: F)
    where
        F: FnMut(&str, i64) + Send + 'static,
    {
        unsafe extern "C" fn trampoline(
            data: *mut std::ffi::c_void,
            file: *const std::os::raw::c_char,
            offset: i64,
        ) {
            let cb: &mut Box<dyn FnMut(&str, i64) + Send> =
                &mut *(data as *mut _);
            let filename = if file.is_null() {
                ""
            } else {
                std::ffi::CStr::from_ptr(file).to_str().unwrap_or("")
            };
            cb(filename, offset);
        }

        let boxed: Box<Box<dyn FnMut(&str, i64) + Send>> =
            Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

        unsafe {
            (*self.inner).callbacks.datacb_data = ptr;
            (*self.inner).callbacks.data_cb = Some(std::mem::transmute(
                trampoline as usize as *const (),
            ));
        }
    }

    /// Set a progress callback.
    ///
    /// The callback receives `(stage, iteration, expected)`.
    /// Return `ProgressAction::Continue` to keep going or
    /// `ProgressAction::Cancel` to abort processing.
    pub fn set_progress_handler<F>(&mut self, callback: F)
    where
        F: FnMut(u32, i32, i32) -> ProgressAction + Send + 'static,
    {
        unsafe extern "C" fn trampoline(
            data: *mut std::ffi::c_void,
            stage: sys::LibRaw_progress,
            iteration: i32,
            expected: i32,
        ) -> i32 {
            let cb: &mut Box<dyn FnMut(u32, i32, i32) -> ProgressAction + Send> =
                &mut *(data as *mut _);
            match cb(stage as u32, iteration, expected) {
                ProgressAction::Continue => 0,
                ProgressAction::Cancel => 1,
            }
        }

        let boxed: Box<Box<dyn FnMut(u32, i32, i32) -> ProgressAction + Send>> =
            Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;

        unsafe {
            (*self.inner).callbacks.progresscb_data = ptr;
            (*self.inner).callbacks.progress_cb = Some(std::mem::transmute(
                trampoline as usize as *const (),
            ));
        }
    }
}
```

**Callback data cleanup note:** The `Box`-allocated closures are leaked when `LibRaw` is dropped — LibRaw's C++ destructor does not free user callback data pointers. For the initial implementation this is acceptable (the OS reclaims the memory on process exit). A future enhancement can add a cleanup wrapper.

- [ ] **Step 2: Verify it compiles**

Run: `cargo build`
Expected: Compiles (may have warnings about unused `context` in trampolines).

- [ ] **Step 3: Commit**

```bash
git add src/callbacks.rs
git commit -m "feat: add callback support (exif, makernotes, data error, progress)"
```

---

### Task 10: High-level facade API

**Files:**
- Create: `src/high_level.rs`

- [ ] **Step 1: Write high_level module**

```rust
// src/high_level.rs
use std::path::Path;

use crate::error::Result;
use crate::params::OutputParams;
use crate::process::*;
use crate::LibRaw;

/// Convenience methods on `LibRaw` for common workflows.
/// These are also available directly via `LibRaw::method(...)`.
impl LibRaw {
    /// Open a RAW file and return a ready-to-use context.
    ///
    /// Shortcut for `LibRaw::new()?.open_file(path)?`.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut raw = Self::new()?;
        raw.open_file(path)?;
        Ok(raw)
    }

    /// Create a context from an in-memory RAW buffer.
    ///
    /// Shortcut for `LibRaw::new()?.open_buffer(data)?`.
    pub fn open_from_buffer(buffer: &[u8]) -> Result<Self> {
        let mut raw = Self::new()?;
        raw.open_buffer(buffer)?;
        Ok(raw)
    }

    /// Run the default processing pipeline.
    ///
    /// Applies default output params, unpacks, and runs dcraw_process.
    /// Equivalent to: `set_params(OutputParams::default())`, `unpack()`, `dcraw_process()`.
    pub fn process_default(&mut self) -> Result<()> {
        self.set_params(&OutputParams::default());
        self.unpack()?;
        self.dcraw_process()
    }

    /// Run processing with custom output params.
    ///
    /// Equivalent to: `set_params(params)`, `unpack()`, `dcraw_process()`.
    pub fn process_with(&mut self, params: &OutputParams) -> Result<()> {
        self.set_params(params);
        self.unpack()?;
        self.dcraw_process()
    }

    /// Get the processed image as an in-memory image.
    ///
    /// Shortcut for `make_mem_image()`.
    pub fn to_image(&mut self) -> Result<crate::image::ProcessedImage> {
        self.make_mem_image()
    }

    /// Static convenience: open, process with defaults, save as TIFF.
    pub fn convert_to_tiff<P: AsRef<Path>>(
        input: P,
        output: P,
    ) -> Result<()> {
        let mut raw = Self::open(input)?;
        raw.process_default()?;
        raw.save_tiff(output)
    }
}
```

- [ ] **Step 2: Verify compilation**

Run: `cargo build`
Expected: Compiles.

- [ ] **Step 3: Write a simple integration test**

Create `tests/integration.rs`:

```rust
use libraw_rs::LibRaw;

/// Test that we can create a context and query version info.
/// This doesn't need a RAW file — it tests the lifecycle.
#[test]
fn test_context_create_and_version() {
    let raw = LibRaw::new().expect("should create context");
    // Version check: the lib should be loaded
    assert!(!raw.inner.is_null());
    // Drop should not crash
}

/// Test that OutputParams default creates valid struct.
#[test]
fn test_default_params() {
    let p = libraw_rs::OutputParams::default();
    assert_eq!(p.inner.output_color, 1); // sRGB
}
```

- [ ] **Step 4: Run integration tests**

Run: `cargo test`
Expected: Both tests pass.

- [ ] **Step 5: Commit**

```bash
git add src/high_level.rs tests/integration.rs
git commit -m "feat: add high-level facade API and integration tests"
```

---

### Task 11: Example program

**Files:**
- Create: `examples/simple.rs`

- [ ] **Step 1: Write example showing basic usage**

```rust
// examples/simple.rs
/// libraw-rs example: read metadata from a RAW file.
///
/// Usage: cargo run --example simple -- path/to/file.CR3
use std::env;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args()
        .nth(1)
        .expect("Usage: simple <raw-file>");

    let raw = libraw_rs::LibRaw::open(&path)?;

    // Read metadata
    let info = raw.image_info();
    println!("Camera:  {} {}", info.make(), info.model());
    println!("Software: {}", info.software());
    println!("Colors:   {}", info.colors());
    println!(
        "Filters:  {:#x} ({} Foveon)",
        info.filters(),
        if info.is_foveon() { "is" } else { "not" }
    );

    let sizes = raw.sizes();
    println!(
        "Size:     {}x{} (raw: {}x{})",
        sizes.width(),
        sizes.height(),
        sizes.raw_width(),
        sizes.raw_height()
    );

    let other = raw.image_other();
    println!("ISO:      {}", other.iso_speed());
    println!("Shutter:  1/{}s", (1.0 / other.shutter()) as i32);
    println!("Aperture: f/{:.1}", other.aperture);
    println!("Focal:    {}mm", other.focal_len);

    let lens = raw.lens_info();
    if !lens.model().is_empty() {
        println!(
            "Lens:     {} {} ({:.0}-{:.0}mm)",
            lens.make(),
            lens.model(),
            lens.min_focal(),
            lens.max_focal()
        );
    }

    Ok(())
}
```

- [ ] **Step 2: Verify it compiles**

Run: `cargo build --example simple`
Expected: Compiles.

- [ ] **Step 3: Commit**

```bash
git add examples/simple.rs
git commit -m "docs: add metadata reading example"
```

---

### Task 12: Remaining C API getters and utility functions

**Files:**
- Create: `src/utils.rs`
- Modify: `src/context.rs`
- Modify: `src/lib.rs`

- [ ] **Step 1: Create utils module with static functions**

```rust
// src/utils.rs
use std::ffi::CStr;

use crate::sys;

/// Get the LibRaw version string (e.g., "0.22.1-Release").
pub fn version() -> &'static str {
    unsafe {
        let ptr = sys::libraw_version();
        if ptr.is_null() {
            return "unknown";
        }
        CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
    }
}

/// Get the LibRaw version number as an integer (LIBRAW_VERSION macro value).
pub fn version_number() -> i32 {
    unsafe { sys::libraw_versionNumber() }
}

/// Get the number of cameras in LibRaw's built-in database.
pub fn camera_count() -> i32 {
    unsafe { sys::libraw_cameraCount() }
}

/// Get the list of supported camera names.
/// Returns a pointer to an array of C string pointers, null-terminated.
/// Each string is in "make/model" format.
pub fn camera_list() -> &'static [*const std::os::raw::c_char] {
    unsafe {
        let ptr = sys::libraw_cameraList();
        if ptr.is_null() {
            return &[];
        }
        let count = camera_count() as usize;
        std::slice::from_raw_parts(ptr, count + 1) // +1 for null terminator
    }
}

/// Query LibRaw capabilities bitmask.
pub fn capabilities() -> u32 {
    unsafe { sys::libraw_capabilities() }
}

/// Get a human-readable progress stage name.
pub fn strprogress(stage: u32) -> &'static str {
    unsafe {
        let ptr = sys::libraw_strprogress(std::mem::transmute(stage as i32));
        if ptr.is_null() {
            return "unknown";
        }
        CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
    }
}
```

- [ ] **Step 2: Add expert getters to LibRaw**

Add to `src/context.rs` after the lens_info method:

```rust
    /// Get the Bayer color filter at (row, col).
    /// Returns 0-3 for R,G1,B,G2, or 6 if no color filter array.
    pub fn color_at(&self, row: i32, col: i32) -> i32 {
        unsafe { sys::libraw_COLOR(self.inner, row, col) }
    }
```

Add to `src/process.rs` at the end of the impl block:

```rust
    pub fn raw_height(&self) -> i32 {
        unsafe { sys::libraw_get_raw_height(self.inner) }
    }

    pub fn raw_width(&self) -> i32 {
        unsafe { sys::libraw_get_raw_width(self.inner) }
    }

    pub fn iheight(&self) -> i32 {
        unsafe { sys::libraw_get_iheight(self.inner) }
    }

    pub fn iwidth(&self) -> i32 {
        unsafe { sys::libraw_get_iwidth(self.inner) }
    }

    pub fn cam_mul(&self, index: i32) -> f32 {
        unsafe { sys::libraw_get_cam_mul(self.inner, index) }
    }

    pub fn pre_mul(&self, index: i32) -> f32 {
        unsafe { sys::libraw_get_pre_mul(self.inner, index) }
    }

    pub fn rgb_cam(&self, index1: i32, index2: i32) -> f32 {
        unsafe { sys::libraw_get_rgb_cam(self.inner, index1, index2) }
    }

    pub fn color_maximum(&self) -> i32 {
        unsafe { sys::libraw_get_color_maximum(self.inner) }
    }
```

- [ ] **Step 3: Register utils module and update re-exports**

Add to `src/lib.rs`:
```rust
pub mod utils;
```

Update the `pub use` block to include:
```rust
pub use utils::{camera_count, camera_list, capabilities, version, version_number};
```

- [ ] **Step 4: Write unit tests**

```rust
// Add to src/utils.rs:
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        let v = version();
        assert!(v.starts_with("0.22"));
    }

    #[test]
    fn test_version_number() {
        let n = version_number();
        assert_eq!(n, (0 << 16) | (22 << 8) | 1);
    }

    #[test]
    fn test_camera_count() {
        let n = camera_count();
        assert!(n > 100, "LibRaw should support >100 cameras, got {n}");
    }

    #[test]
    fn test_capabilities() {
        let caps = capabilities();
        // Should have at least basic RAW decoding support
        assert!(caps > 0);
    }
}
```

- [ ] **Step 5: Run tests**

Run: `cargo test -p libraw-rs --lib utils`
Expected: All tests pass.

- [ ] **Step 6: Commit**

```bash
git add src/utils.rs src/context.rs src/process.rs src/lib.rs
git commit -m "feat: add utility functions (version, camera list, capabilities, getters)"
```

---

### Task 13: Final verification and cleanup

**Files:**
- Modify: `src/lib.rs` (re-export cleanup)

- [ ] **Step 1: Review the public API surface**

Read `src/lib.rs` again. Make sure all public items are properly re-exported:

```rust
mod sys;
pub mod callbacks;
pub mod context;
pub mod error;
pub mod image;
pub mod metadata;
pub mod params;
pub mod process;
pub mod stream;
mod high_level;

pub use context::LibRaw;
pub use error::{Error, Result};
pub use image::{ImageType, ProcessedImage};
pub use params::{HighlightMode, OutputColor, OutputParams};
```

- [ ] **Step 2: Run the full test suite**

Run: `cargo test`
Expected: All tests pass (unit + integration).

- [ ] **Step 3: Run clippy**

Run: `cargo clippy --all-targets -- -D warnings`
Expected: Zero warnings. If there are warnings, fix them.

- [ ] **Step 4: Verify `cargo doc` builds**

Run: `cargo doc --no-deps`
Expected: Documentation generated without errors.

- [ ] **Step 5: Final commit**

```bash
git add -A
git commit -m "chore: final cleanup, API re-exports, documentation"
```
