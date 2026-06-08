use std::os::raw::c_char;
use crate::sys;

fn cstr_to_str(bytes: &[c_char]) -> &str {
    let bytes: &[u8] = unsafe { std::slice::from_raw_parts(bytes.as_ptr() as *const u8, bytes.len()) };
    let end = bytes.iter().position(|&b| b == 0).unwrap_or(bytes.len());
    std::str::from_utf8(&bytes[..end]).unwrap_or("")
}

/// Read-only access to image identification params (available after `open_file`).
pub struct ImageInfo<'a> { pub(crate) inner: &'a sys::libraw_iparams_t }

impl<'a> ImageInfo<'a> {
    pub fn make(&self) -> &str { cstr_to_str(&self.inner.make) }
    pub fn model(&self) -> &str { cstr_to_str(&self.inner.model) }
    pub fn software(&self) -> &str { cstr_to_str(&self.inner.software) }
    pub fn normalized_make(&self) -> &str { cstr_to_str(&self.inner.normalized_make) }
    pub fn normalized_model(&self) -> &str { cstr_to_str(&self.inner.normalized_model) }
    pub fn maker_index(&self) -> u32 { self.inner.maker_index }
    pub fn raw_count(&self) -> u32 { self.inner.raw_count }
    pub fn dng_version(&self) -> u32 { self.inner.dng_version }
    pub fn is_foveon(&self) -> bool { self.inner.is_foveon != 0 }
    pub fn colors(&self) -> i32 { self.inner.colors }
    pub fn filters(&self) -> u32 { self.inner.filters }
    pub fn cdesc(&self) -> &str { cstr_to_str(&self.inner.cdesc) }
}

/// Read-only access to lens metadata.
pub struct LensInfo<'a> { pub(crate) inner: &'a sys::libraw_lensinfo_t }

impl<'a> LensInfo<'a> {
    pub fn min_focal(&self) -> f32 { self.inner.MinFocal }
    pub fn max_focal(&self) -> f32 { self.inner.MaxFocal }
    pub fn max_ap4_min_focal(&self) -> f32 { self.inner.MaxAp4MinFocal }
    pub fn max_ap4_max_focal(&self) -> f32 { self.inner.MaxAp4MaxFocal }
    pub fn exif_max_ap(&self) -> f32 { self.inner.EXIF_MaxAp }
    pub fn make(&self) -> &str { cstr_to_str(&self.inner.LensMake) }
    pub fn model(&self) -> &str { cstr_to_str(&self.inner.Lens) }
    pub fn serial(&self) -> &str { cstr_to_str(&self.inner.LensSerial) }
    pub fn focal_length_in_35mm(&self) -> u16 { self.inner.FocalLengthIn35mmFormat }
}

/// Read-only access to image-level metadata (ISO, shutter, etc.).
pub struct ImageOther<'a> { pub(crate) inner: &'a sys::libraw_imgother_t }

impl<'a> ImageOther<'a> {
    pub fn iso_speed(&self) -> f32 { self.inner.iso_speed }
    pub fn shutter(&self) -> f32 { self.inner.shutter }
    pub fn aperture(&self) -> f32 { self.inner.aperture }
    pub fn focal_len(&self) -> f32 { self.inner.focal_len }
    pub fn timestamp(&self) -> i64 { self.inner.timestamp as i64 }
    pub fn shot_order(&self) -> u32 { self.inner.shot_order }
    pub fn desc(&self) -> &str { cstr_to_str(&self.inner.desc) }
    pub fn artist(&self) -> &str { cstr_to_str(&self.inner.artist) }
}

/// Read-only access to image size dimensions.
pub struct ImageSizes<'a> { pub(crate) inner: &'a sys::libraw_image_sizes_t }

impl<'a> ImageSizes<'a> {
    pub fn raw_width(&self) -> u16 { self.inner.raw_width }
    pub fn raw_height(&self) -> u16 { self.inner.raw_height }
    pub fn width(&self) -> u16 { self.inner.width }
    pub fn height(&self) -> u16 { self.inner.height }
    pub fn top_margin(&self) -> u16 { self.inner.top_margin }
    pub fn left_margin(&self) -> u16 { self.inner.left_margin }
    pub fn iwidth(&self) -> u16 { self.inner.iwidth }
    pub fn iheight(&self) -> u16 { self.inner.iheight }
    pub fn pixel_aspect(&self) -> f64 { self.inner.pixel_aspect }
    pub fn flip(&self) -> i32 { self.inner.flip }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cstr_to_str_normal() {
        let buf: [c_char; 8] = [
            b'h' as c_char, b'e' as c_char, b'l' as c_char, b'l' as c_char,
            b'o' as c_char, 0 as c_char, b'x' as c_char, b'x' as c_char,
        ];
        assert_eq!(cstr_to_str(&buf), "hello");
    }

    #[test]
    fn test_cstr_to_str_no_null() {
        let buf: [c_char; 2] = [b'h' as c_char, b'i' as c_char];
        assert_eq!(cstr_to_str(&buf), "hi");
    }

    #[test]
    fn test_cstr_to_str_empty() {
        let buf = [0 as c_char; 10];
        assert_eq!(cstr_to_str(&buf), "");
    }
}
