use crate::image::QuatImage;
use crate::error::QuatImageError;
use std::os::raw::{c_int, c_uint, c_char};
use std::cell::RefCell;
use std::ffi::CString;
use std::ptr;
use std::panic::AssertUnwindSafe;

thread_local! {
    static LAST_ERROR: RefCell<Option<String>> = RefCell::new(None);
}

fn set_last_error(err: QuatImageError) {
    LAST_ERROR.with(|e| {
        *e.borrow_mut() = Some(err.to_string());
    });
}

pub struct QuatImageHandle {
    inner: QuatImage,
}

// Pomocna funkcija za bezbedno izvršavanje FFI poziva
fn safe_ffi<F, T>(f: F, error_val: T) -> T
where
    F: FnOnce() -> Result<T, QuatImageError>,
{
    match std::panic::catch_unwind(AssertUnwindSafe(f)) {
        Ok(Ok(val)) => val,
        Ok(Err(err)) => {
            set_last_error(err);
            error_val
        }
        Err(_) => {
            // Panika se desila unutar Rust koda
            let _ = LAST_ERROR.with(|e| {
                *e.borrow_mut() = Some("Došlo je do panike u Rust kodu".to_string());
            });
            error_val
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_new(width: c_uint, height: c_uint) -> *mut QuatImageHandle {
    safe_ffi(|| {
        let img = QuatImage::new(width as usize, height as usize)?;
        Ok(Box::into_raw(Box::new(QuatImageHandle { inner: img })))
    }, ptr::null_mut())
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_free(handle: *mut QuatImageHandle) {
    if !handle.is_null() {
        unsafe {
            let _ = Box::from_raw(handle);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_width(handle: *const QuatImageHandle) -> c_int {
    if handle.is_null() { return -1; }
    unsafe { (*handle).inner.width() as c_int }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_height(handle: *const QuatImageHandle) -> c_int {
    if handle.is_null() { return -1; }
    unsafe { (*handle).inner.height() as c_int }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_get_pixel(
    handle: *const QuatImageHandle,
    x: c_uint,
    y: c_uint,
    result: *mut u8,
) -> c_int {
    if handle.is_null() || result.is_null() { return -1; }
    safe_ffi(|| {
        let val = unsafe { (*handle).inner.get_pixel(x as usize, y as usize)? };
        unsafe { *result = val; }
        Ok(0)
    }, -1)
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_get_last_error_message() -> *const c_char {
    LAST_ERROR.with(|e| {
        match &*e.borrow() {
            Some(err_str) => {
                match CString::new(err_str.clone()) {
                    Ok(c_str) => c_str.into_raw(),
                    Err(_) => ptr::null(),
                }
            }
            None => ptr::null(),
        }
    })
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_free_error_message(ptr: *mut c_char) {
    if !ptr.is_null() {
        unsafe {
            let _ = CString::from_raw(ptr);
        }
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_data_len(handle: *const QuatImageHandle) -> c_int {
    if handle.is_null() { return -1; }
    unsafe { (*handle).inner.as_bytes().len() as c_int }
}

#[unsafe(no_mangle)]
pub extern "C" fn quat_image_copy_bytes(
    handle: *const QuatImageHandle,
    dest: *mut u8,
    dest_cap: c_int,
) -> c_int {
    if handle.is_null() || dest.is_null() { return -1; }
    safe_ffi(|| {
        let bytes = unsafe { (*handle).inner.as_bytes() };
        if (dest_cap as usize) < bytes.len() {
            return Err(QuatImageError::OutOfBounds);
        }
        unsafe {
            std::ptr::copy_nonoverlapping(bytes.as_ptr(), dest, bytes.len());
        }
        Ok(bytes.len() as c_int)
    }, -1)
}