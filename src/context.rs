use std::ffi::CString;
use std::path::Path;

use crate::error::{Error, Result};
use crate::sys;

/// LibRaw processing context.
pub struct LibRaw {
    pub inner: *mut sys::libraw_data_t,
}

unsafe impl Send for LibRaw {}

impl LibRaw {
    pub fn new() -> Result<Self> {
        Self::with_flags(0)
    }

    pub fn with_flags(flags: u32) -> Result<Self> {
        let inner = unsafe { sys::libraw_init(flags) };
        if inner.is_null() {
            return Err(Error::OutOfMemory);
        }
        Ok(Self { inner })
    }

    pub fn open_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        let code = unsafe { sys::libraw_open_file(self.inner, c_path.as_ptr()) };
        Error::check(code)
    }

    pub fn recycle(&mut self) {
        unsafe { sys::libraw_recycle(self.inner) };
    }

    pub fn recycle_datastream(&mut self) {
        unsafe { sys::libraw_recycle_datastream(self.inner) };
    }

    /// Apply output parameters.
    pub fn set_params(&mut self, params: &crate::params::OutputParams) {
        unsafe { (*self.inner).params = params.inner };
    }

    /// Raw mutable access to output params.
    pub fn params_mut(&mut self) -> &mut sys::libraw_output_params_t {
        unsafe { &mut (*self.inner).params }
    }

    /// Get image identification info (requires `open_file`).
    pub fn image_info(&self) -> crate::metadata::ImageInfo<'_> {
        crate::metadata::ImageInfo { inner: unsafe { &(*self.inner).idata } }
    }

    /// Get lens info (requires `open_file`).
    pub fn lens_info(&self) -> crate::metadata::LensInfo<'_> {
        crate::metadata::LensInfo { inner: unsafe { &(*self.inner).lens } }
    }

    /// Get other image metadata (requires `open_file`).
    pub fn image_other(&self) -> crate::metadata::ImageOther<'_> {
        crate::metadata::ImageOther { inner: unsafe { &(*self.inner).other } }
    }

    /// Get image sizes (requires `open_file`).
    pub fn sizes(&self) -> crate::metadata::ImageSizes<'_> {
        crate::metadata::ImageSizes { inner: unsafe { &(*self.inner).sizes } }
    }

    /// Get the Bayer color filter at (row, col).
    pub fn color_at(&self, row: i32, col: i32) -> i32 {
        unsafe { sys::libraw_COLOR(self.inner, row, col) }
    }
}

impl Drop for LibRaw {
    fn drop(&mut self) {
        unsafe { sys::libraw_close(self.inner) };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_creates_context() {
        let raw = LibRaw::new();
        assert!(raw.is_ok());
        let raw = raw.unwrap();
        assert!(!raw.inner.is_null());
    }

    #[test]
    fn test_with_flags_zero() {
        let raw = LibRaw::with_flags(0);
        assert!(raw.is_ok());
    }

    #[test]
    fn test_version_string() {
        let ver = unsafe {
            let ptr = sys::libraw_version();
            std::ffi::CStr::from_ptr(ptr).to_string_lossy().into_owned()
        };
        assert!(ver.starts_with("0.22"));
    }
}
