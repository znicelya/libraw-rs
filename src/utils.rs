use std::ffi::CStr;
use crate::sys;

pub fn version() -> &'static str {
    unsafe {
        let ptr = sys::libraw_version();
        if ptr.is_null() { return "unknown"; }
        CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
    }
}

pub fn version_number() -> i32 { unsafe { sys::libraw_versionNumber() } }

pub fn camera_count() -> i32 { unsafe { sys::libraw_cameraCount() } }

pub fn camera_list() -> &'static [*const std::os::raw::c_char] {
    unsafe {
        let ptr = sys::libraw_cameraList();
        if ptr.is_null() { return &[]; }
        std::slice::from_raw_parts(ptr, camera_count() as usize + 1)
    }
}

pub fn capabilities() -> u32 { unsafe { sys::libraw_capabilities() } }

pub fn strprogress(stage: u32) -> &'static str {
    unsafe {
        let ptr = sys::libraw_strprogress(std::mem::transmute(stage as i32));
        if ptr.is_null() { return "unknown"; }
        CStr::from_ptr(ptr).to_str().unwrap_or("unknown")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_string() {
        assert!(version().starts_with("0.22"));
    }

    #[test]
    fn test_version_number() {
        assert_eq!(version_number(), (0 << 16) | (22 << 8) | 1);
    }

    #[test]
    fn test_camera_count() {
        let n = camera_count();
        assert!(n > 100, "LibRaw should support >100 cameras, got {n}");
    }

    #[test]
    fn test_capabilities() {
        // capabilities() may return 0 if certain features were disabled at compile time
        let _caps = capabilities();
        // Just verify it doesn't crash
    }
}
