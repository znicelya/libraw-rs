use crate::error::{Error, Result};
use crate::sys;
use crate::LibRaw;

impl LibRaw {
    /// Open raw Bayer pattern from sensor data.
    #[allow(clippy::too_many_arguments)]
    pub fn open_bayer(
        &mut self, data: &[u8],
        raw_width: u16, raw_height: u16,
        left_margin: u16, top_margin: u16,
        right_margin: u16, bottom_margin: u16,
        procflags: u8, bayer_pattern: u8,
        unused_bits: u32, otherflags: u32, black_level: u32,
    ) -> Result<()> {
        let code = unsafe {
            sys::libraw_open_bayer(
                self.inner, data.as_ptr() as *mut u8, data.len() as u32,
                raw_width, raw_height, left_margin, top_margin,
                right_margin, bottom_margin, procflags, bayer_pattern,
                unused_bits, otherflags, black_level,
            )
        };
        Error::check(code)
    }
}
