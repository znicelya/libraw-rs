use crate::sys;
use crate::LibRaw;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgressAction { Continue = 0, Cancel = 1 }

impl LibRaw {
    pub fn set_exifparser_handler<F>(&mut self, callback: F)
    where F: FnMut(i32, i32, i32, u32, i64) + Send + 'static
    {
        unsafe extern "C" fn trampoline(
            context: *mut std::ffi::c_void, tag: i32, type_: i32, len: i32,
            ord: u32, _ifp: *mut std::ffi::c_void, base: i64,
        ) {
            let cb: &mut Box<dyn FnMut(i32, i32, i32, u32, i64) + Send> =
                unsafe { &mut *(context as *mut _) };
            cb(tag, type_, len, ord, base);
        }
        let boxed: Box<Box<dyn FnMut(i32, i32, i32, u32, i64) + Send>> = Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        unsafe {
            sys::libraw_set_exifparser_handler(
                self.inner,
                Some(std::mem::transmute(trampoline as *const () as usize as *const ())),
                ptr,
            );
        }
    }

    pub fn set_makernotes_handler<F>(&mut self, callback: F)
    where F: FnMut(i32, i32, i32, u32, i64) + Send + 'static
    {
        unsafe extern "C" fn trampoline(
            context: *mut std::ffi::c_void, tag: i32, type_: i32, len: i32,
            ord: u32, _ifp: *mut std::ffi::c_void, base: i64,
        ) {
            let cb: &mut Box<dyn FnMut(i32, i32, i32, u32, i64) + Send> =
                unsafe { &mut *(context as *mut _) };
            cb(tag, type_, len, ord, base);
        }
        let boxed: Box<Box<dyn FnMut(i32, i32, i32, u32, i64) + Send>> = Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        unsafe {
            sys::libraw_set_makernotes_handler(
                self.inner,
                Some(std::mem::transmute(trampoline as *const () as usize as *const ())),
                ptr,
            );
        }
    }

    pub fn set_dataerror_handler<F>(&mut self, callback: F)
    where F: FnMut(&str, i64) + Send + 'static
    {
        unsafe extern "C" fn trampoline(
            data: *mut std::ffi::c_void, file: *const std::os::raw::c_char, offset: i64,
        ) {
            let cb: &mut Box<dyn FnMut(&str, i64) + Send> =
                unsafe { &mut *(data as *mut _) };
            let filename = if file.is_null() { "" }
                else { unsafe { std::ffi::CStr::from_ptr(file) }.to_str().unwrap_or("") };
            cb(filename, offset);
        }
        let boxed: Box<Box<dyn FnMut(&str, i64) + Send>> = Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        unsafe {
            sys::libraw_set_dataerror_handler(
                self.inner,
                Some(std::mem::transmute(trampoline as *const () as usize as *const ())),
                ptr,
            );
        }
    }

    pub fn set_progress_handler<F>(&mut self, callback: F)
    where F: FnMut(u32, i32, i32) -> ProgressAction + Send + 'static
    {
        unsafe extern "C" fn trampoline(
            data: *mut std::ffi::c_void, stage: sys::LibRaw_progress,
            iteration: i32, expected: i32,
        ) -> i32 {
            let cb: &mut Box<dyn FnMut(u32, i32, i32) -> ProgressAction + Send> =
                unsafe { &mut *(data as *mut _) };
            match cb(stage as u32, iteration, expected) {
                ProgressAction::Continue => 0,
                ProgressAction::Cancel => 1,
            }
        }
        let boxed: Box<Box<dyn FnMut(u32, i32, i32) -> ProgressAction + Send>> = Box::new(Box::new(callback));
        let ptr = Box::into_raw(boxed) as *mut std::ffi::c_void;
        unsafe {
            sys::libraw_set_progress_handler(
                self.inner,
                Some(std::mem::transmute(trampoline as *const () as usize as *const ())),
                ptr,
            );
        }
    }
}
