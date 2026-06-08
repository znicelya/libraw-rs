use std::path::Path;

use crate::error::Result;
use crate::params::OutputParams;
use crate::LibRaw;

impl LibRaw {
    /// Open a RAW file (new + open_file).
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        let mut raw = Self::new()?;
        raw.open_file(path)?;
        Ok(raw)
    }

    /// Open from memory buffer.
    pub fn open_from_buffer(buffer: &[u8]) -> Result<Self> {
        let mut raw = Self::new()?;
        raw.open_buffer(buffer)?;
        Ok(raw)
    }

    /// Run default processing: set default params, unpack, dcraw_process.
    pub fn process_default(&mut self) -> Result<()> {
        self.set_params(&OutputParams::default());
        self.unpack()?;
        self.dcraw_process()
    }

    /// Run processing with custom params.
    pub fn process_with(&mut self, params: &OutputParams) -> Result<()> {
        self.set_params(params);
        self.unpack()?;
        self.dcraw_process()
    }

    /// Get processed image (shortcut for make_mem_image).
    pub fn to_image(&mut self) -> Result<crate::image::ProcessedImage> {
        self.make_mem_image()
    }

    /// Convenience: open + process + save as TIFF.
    pub fn convert_to_tiff<P: AsRef<Path>>(input: P, output: P) -> Result<()> {
        let mut raw = Self::open(input)?;
        raw.process_default()?;
        raw.save_tiff(output)
    }
}
