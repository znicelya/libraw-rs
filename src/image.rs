use std::path::Path;

use crate::sys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageType { Jpeg, Bitmap }

/// In-memory processed image result.
pub struct ProcessedImage {
    pub(crate) inner: *mut sys::libraw_processed_image_t,
}

unsafe impl Send for ProcessedImage {}

impl ProcessedImage {
    pub fn image_type(&self) -> ImageType {
        let t = unsafe { (*self.inner).type_ };
        if t == sys::LibRaw_image_formats_LIBRAW_IMAGE_JPEG as i32 { ImageType::Jpeg }
        else { ImageType::Bitmap }
    }

    pub fn data(&self) -> &[u8] {
        let len = unsafe { (*self.inner).data_size as usize };
        unsafe { std::slice::from_raw_parts((*self.inner).data.as_ptr(), len) }
    }

    pub fn width(&self) -> u16 { unsafe { (*self.inner).width } }
    pub fn height(&self) -> u16 { unsafe { (*self.inner).height } }
    pub fn colors(&self) -> u16 { unsafe { (*self.inner).colors } }
    pub fn bits(&self) -> u16 { unsafe { (*self.inner).bits } }

    pub fn save<P: AsRef<Path>>(&self, path: P) -> std::io::Result<()> {
        std::fs::write(path, self.data())
    }
}

impl Drop for ProcessedImage {
    fn drop(&mut self) {
        unsafe { sys::libraw_dcraw_clear_mem(self.inner) };
    }
}
