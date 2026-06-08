use std::ffi::CStr;
use thiserror::Error;

use crate::sys;

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
    pub(crate) fn from_raw(code: i32) -> Self {
        debug_assert_ne!(code, 0, "zero is not an error code");

        let msg = unsafe {
            let ptr = sys::libraw_strerror(code);
            if ptr.is_null() {
                String::from("unknown error")
            } else {
                CStr::from_ptr(ptr).to_string_lossy().into_owned()
            }
        };

        match code {
            -1 => Error::UnsupportedFeature(msg),
            -2 => Error::UnsupportedFormat,
            -3 => Error::UnsupportedFeature(msg),
            -4 => Error::OutOfOrderCall,
            -5 => Error::NoThumbnail,
            -6 => Error::UnsupportedThumbnail,
            -7 => Error::InputClosed,
            -8 => Error::NotImplemented,
            -9 => Error::UnsupportedFeature(msg),
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
