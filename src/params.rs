use crate::sys;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputColor { Raw = 0, Srgb = 1, AdobeRgb = 2, WideGamut = 3, ProPhoto = 4, Xyz = 5 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HighlightMode { Clip = 0, Unclip = 1, Blend = 2, Rebuild = 3 }

#[derive(Debug, Clone)]
pub struct OutputParams {
    pub inner: sys::libraw_output_params_t,
}

impl Default for OutputParams {
    fn default() -> Self {
        let mut inner = unsafe { std::mem::zeroed::<sys::libraw_output_params_t>() };
        inner.greybox = [0, 0, 0xffff, 0];
        inner.aber = [0.0; 4];
        inner.user_mul = [0.0; 4];
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
        inner.output_profile = std::ptr::null_mut();
        inner.camera_profile = std::ptr::null_mut();
        inner.bad_pixels = std::ptr::null_mut();
        inner.dark_frame = std::ptr::null_mut();
        Self { inner }
    }
}

impl OutputParams {
    pub fn output_color(mut self, color: OutputColor) -> Self { self.inner.output_color = color as i32; self }
    pub fn output_bps(mut self, bps: u8) -> Self { self.inner.output_bps = bps as i32; self }
    pub fn highlight_mode(mut self, mode: HighlightMode) -> Self { self.inner.highlight = mode as i32; self }
    pub fn user_mul(mut self, index: usize, value: f32) -> Self { self.inner.user_mul[index] = value; self }
    pub fn use_auto_wb(mut self, on: bool) -> Self { self.inner.use_auto_wb = on as i32; self }
    pub fn use_camera_wb(mut self, on: bool) -> Self { self.inner.use_camera_wb = on as i32; self }
    pub fn use_camera_matrix(mut self, on: bool) -> Self { self.inner.use_camera_matrix = on as i32; self }
    pub fn brightness(mut self, bright: f32) -> Self { self.inner.bright = bright; self }
    pub fn no_auto_bright(mut self, on: bool) -> Self { self.inner.no_auto_bright = on as i32; self }
    pub fn auto_bright_thr(mut self, thr: f32) -> Self { self.inner.auto_bright_thr = thr; self }
    pub fn adjust_maximum_thr(mut self, thr: f32) -> Self { self.inner.adjust_maximum_thr = thr; self }
    pub fn gamma(mut self, power: f64, slope: f64) -> Self { self.inner.gamm = [power, slope, 0.0, 0.0, 0.0, 0.0]; self }
    pub fn use_fuji_rotate(mut self, on: bool) -> Self { self.inner.use_fuji_rotate = on as i32; self }
    pub fn half_size(mut self, on: bool) -> Self { self.inner.half_size = on as i32; self }
    pub fn output_tiff(mut self, on: bool) -> Self { self.inner.output_tiff = on as i32; self }
    pub fn user_flip(mut self, flip: i32) -> Self { self.inner.user_flip = flip; self }
    pub fn green_matching(mut self, on: bool) -> Self { self.inner.green_matching = on as i32; self }
    pub fn dcb_iterations(mut self, n: i32) -> Self { self.inner.dcb_iterations = n; self }
    pub fn dcb_enhance(mut self, on: bool) -> Self { self.inner.dcb_enhance_fl = on as i32; self }
    pub fn fbdd_noiserd(mut self, value: i32) -> Self { self.inner.fbdd_noiserd = value; self }
    pub fn exp_correc(mut self, on: bool) -> Self { self.inner.exp_correc = on as i32; self }
    pub fn exp_shift(mut self, shift: f32) -> Self { self.inner.exp_shift = shift; self }
    pub fn exp_preser(mut self, preser: f32) -> Self { self.inner.exp_preser = preser; self }
    pub fn no_auto_scale(mut self, on: bool) -> Self { self.inner.no_auto_scale = on as i32; self }
    pub fn no_interpolation(mut self, on: bool) -> Self { self.inner.no_interpolation = on as i32; self }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_params() {
        let p = OutputParams::default();
        assert_eq!(p.inner.output_color, 1);
        assert_eq!(p.inner.output_bps, 8);
        assert_eq!(p.inner.bright, 1.0);
    }

    #[test]
    fn test_builder_chain() {
        let p = OutputParams::default()
            .output_color(OutputColor::AdobeRgb)
            .output_bps(16)
            .highlight_mode(HighlightMode::Blend)
            .use_camera_wb(true)
            .brightness(1.2);
        assert_eq!(p.inner.output_color, 2);
        assert_eq!(p.inner.output_bps, 16);
        assert_eq!(p.inner.highlight, 2);
        assert_eq!(p.inner.use_camera_wb, 1);
        assert_eq!(p.inner.bright, 1.2);
    }
}
