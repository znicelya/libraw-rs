use std::ffi::CString;
use std::path::Path;

use crate::error::{Error, Result};
use crate::sys;
use crate::LibRaw;

impl LibRaw {
    pub fn open_buffer(&mut self, buffer: &[u8]) -> Result<()> {
        let code = unsafe {
            sys::libraw_open_buffer(self.inner, buffer.as_ptr() as *const std::ffi::c_void, buffer.len())
        };
        Error::check(code)
    }

    pub fn unpack(&mut self) -> Result<()> {
        Error::check(unsafe { sys::libraw_unpack(self.inner) })
    }

    pub fn unpack_thumb(&mut self) -> Result<()> {
        Error::check(unsafe { sys::libraw_unpack_thumb(self.inner) })
    }

    pub fn unpack_thumb_ex(&mut self, index: i32) -> Result<()> {
        Error::check(unsafe { sys::libraw_unpack_thumb_ex(self.inner, index) })
    }

    pub fn subtract_black(&mut self) {
        unsafe { sys::libraw_subtract_black(self.inner) };
    }

    pub fn raw2image(&mut self) -> Result<()> {
        Error::check(unsafe { sys::libraw_raw2image(self.inner) })
    }

    pub fn free_image(&mut self) {
        unsafe { sys::libraw_free_image(self.inner) };
    }

    pub fn dcraw_process(&mut self) -> Result<()> {
        Error::check(unsafe { sys::libraw_dcraw_process(self.inner) })
    }

    pub fn make_mem_image(&mut self) -> Result<crate::image::ProcessedImage> {
        let mut err: i32 = 0;
        let ptr = unsafe { sys::libraw_dcraw_make_mem_image(self.inner, &mut err) };
        if ptr.is_null() {
            return Err(if err != 0 { Error::from_raw(err) } else {
                Error::Unknown { code: -1, msg: "make_mem_image returned null".into() }
            });
        }
        Ok(crate::image::ProcessedImage { inner: ptr })
    }

    pub fn make_mem_thumb(&mut self) -> Result<crate::image::ProcessedImage> {
        let mut err: i32 = 0;
        let ptr = unsafe { sys::libraw_dcraw_make_mem_thumb(self.inner, &mut err) };
        if ptr.is_null() {
            return Err(if err != 0 { Error::from_raw(err) } else {
                Error::Unknown { code: -1, msg: "make_mem_thumb returned null".into() }
            });
        }
        Ok(crate::image::ProcessedImage { inner: ptr })
    }

    pub fn save_tiff<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        Error::check(unsafe { sys::libraw_dcraw_ppm_tiff_writer(self.inner, c_path.as_ptr()) })
    }

    pub fn save_thumb<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let c_path = CString::new(path.as_ref().to_string_lossy().as_bytes())?;
        Error::check(unsafe { sys::libraw_dcraw_thumb_writer(self.inner, c_path.as_ptr()) })
    }

    pub fn adjust_to_raw_inset_crop(&mut self, mask: u32, maxcrop: f32) -> Result<()> {
        Error::check(unsafe { sys::libraw_adjust_to_raw_inset_crop(self.inner, mask, maxcrop) })
    }

    pub fn unpack_function_name(&self) -> &str {
        unsafe {
            let ptr = sys::libraw_unpack_function_name(self.inner);
            if ptr.is_null() { return ""; }
            std::ffi::CStr::from_ptr(ptr).to_str().unwrap_or("")
        }
    }

    pub fn decoder_info(&self) -> sys::libraw_decoder_info_t {
        let mut info = sys::libraw_decoder_info_t { decoder_name: std::ptr::null(), decoder_flags: 0 };
        unsafe { sys::libraw_get_decoder_info(self.inner, &mut info) };
        info
    }

    pub fn adjust_sizes_info_only(&mut self) -> Result<()> {
        Error::check(unsafe { sys::libraw_adjust_sizes_info_only(self.inner) })
    }

    // Helper queries using rawdata inspection (since internal fields aren't public)
    pub fn is_jpeg_thumb(&self) -> bool {
        unsafe {
            (*self.inner).thumbnail.tformat == sys::LibRaw_thumbnail_formats_LIBRAW_THUMBNAIL_JPEG as i32
        }
    }

    pub fn have_fpdata(&self) -> bool {
        unsafe {
            !(*self.inner).rawdata.float_image.is_null()
                || !(*self.inner).rawdata.float3_image.is_null()
                || !(*self.inner).rawdata.float4_image.is_null()
        }
    }

    // Expert getters via C API
    pub fn raw_height(&self) -> i32 { unsafe { sys::libraw_get_raw_height(self.inner) } }
    pub fn raw_width(&self) -> i32 { unsafe { sys::libraw_get_raw_width(self.inner) } }
    pub fn iheight(&self) -> i32 { unsafe { sys::libraw_get_iheight(self.inner) } }
    pub fn iwidth(&self) -> i32 { unsafe { sys::libraw_get_iwidth(self.inner) } }
    pub fn cam_mul(&self, index: i32) -> f32 { unsafe { sys::libraw_get_cam_mul(self.inner, index) } }
    pub fn pre_mul(&self, index: i32) -> f32 { unsafe { sys::libraw_get_pre_mul(self.inner, index) } }
    pub fn rgb_cam(&self, index1: i32, index2: i32) -> f32 { unsafe { sys::libraw_get_rgb_cam(self.inner, index1, index2) } }
    pub fn color_maximum(&self) -> i32 { unsafe { sys::libraw_get_color_maximum(self.inner) } }
}
