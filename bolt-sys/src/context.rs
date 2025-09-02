use crate::sys::{self, BT_TRUE};

use std::{
    alloc::Layout,
    ffi::{CString, c_void},
};

impl BoltContext {
    /// Create a new BoltContext, optionally disabling default Rust handler overrides
    ///
    /// By default the wrapper library provides idiomatic overrides to handlers for any given compile target substituting malloc, free, realloc,
    /// for the current rust allocator - replacing print and write with `println` or `console.log` for web targets.
    /// Setting `disable_default_overrides` to true allows falling back to whatever was compiled into bolt as a default, which may be nothing depending on your target and build flags.
    pub fn new() -> Self {
        unsafe {
            let mut handlers = sys::bt_default_handlers();

            Self::override_handlers(&mut handlers);

            let mut ctx = std::ptr::null_mut();
            sys::bt_open(&mut ctx, &mut handlers);
            BoltContext {
                ctx,
                _handlers: handlers,
            }
        }
    }

    // Override the default handlers of malloc and print with a sensible
    // system default, unless the user has one.
    fn override_handlers(handlers: &mut sys::bt_Handlers) {
        unsafe extern "C" fn rust_alloc(size: usize) -> *mut c_void {
            unsafe { std::alloc::alloc(Layout::array::<u8>(size).unwrap_unchecked()) as _ }
        }

        unsafe extern "C" fn rust_free(ptr: *mut c_void) {
            if !ptr.is_null() {
                unsafe { std::alloc::dealloc(ptr as *mut u8, Layout::new::<u8>()) }
            }
        }

        unsafe extern "C" fn rust_realloc(ptr: *mut c_void, size: usize) -> *mut c_void {
            if ptr.is_null() {
                unsafe { std::alloc::alloc(Layout::array::<u8>(size).unwrap_unchecked()) as _ }
            } else {
                unsafe { std::alloc::realloc(ptr as *mut u8, Layout::new::<u8>(), size) as _ }
            }
        }

        unsafe extern "C" fn rust_write(_ctx: *mut sys::bt_Context, msg: *const std::ffi::c_char) {
            if !msg.is_null() {
                if let Ok(msg_str) = unsafe { std::ffi::CStr::from_ptr(msg) }.to_str() {
                    print!("{}", msg_str);
                }
            }
        }

        unsafe extern "C" fn rust_on_error(
            error_type: sys::bt_ErrorType,
            module: *const std::ffi::c_char,
            message: *const std::ffi::c_char,
            line: u16,
            col: u16,
        ) {
            let error_type_str = match error_type {
                sys::bt_ErrorType_BT_ERROR_PARSE => "Parse Error",
                sys::bt_ErrorType_BT_ERROR_COMPILE => "Compile Error",
                sys::bt_ErrorType_BT_ERROR_RUNTIME => "Runtime Error",
                _ => "Unknown Error",
            };

            let module_str = if !module.is_null() {
                unsafe { std::ffi::CStr::from_ptr(module) }
                    .to_str()
                    .unwrap_or("unknown")
            } else {
                "unknown"
            };

            let message_str = if !message.is_null() {
                unsafe { std::ffi::CStr::from_ptr(message) }
                    .to_str()
                    .unwrap_or("unknown error")
            } else {
                "unknown error"
            };

            eprintln!(
                "{} in {}: {} (line {}, col {})",
                error_type_str, module_str, message_str, line, col
            );
        }

        unsafe extern "C" fn rust_read_file(
            _ctx: *mut sys::bt_Context,
            path: *const std::ffi::c_char,
            out_handle: *mut *mut c_void,
        ) -> *mut std::ffi::c_char {
            let Some(path) = (if path.is_null() { None } else { Some(path) }) else {
                return std::ptr::null_mut();
            };
            let Some(out_handle) = (if out_handle.is_null() {
                None
            } else {
                Some(out_handle)
            }) else {
                return std::ptr::null_mut();
            };

            let Ok(path_str) = unsafe { std::ffi::CStr::from_ptr(path) }.to_str() else {
                return std::ptr::null_mut();
            };

            let Ok(file) = std::fs::File::open(path_str) else {
                return std::ptr::null_mut();
            };

            let boxed_file = Box::new(file);
            unsafe {
                *out_handle = Box::into_raw(boxed_file) as *mut _;
            }

            let Ok(contents) = std::fs::read_to_string(path_str) else {
                unsafe {
                    let _ = Box::from_raw(*out_handle);
                    *out_handle = std::ptr::null_mut();
                }
                return std::ptr::null_mut();
            };

            let Ok(c_string) = CString::new(contents) else {
                unsafe {
                    let _ = Box::from_raw(*out_handle);
                    *out_handle = std::ptr::null_mut();
                }
                return std::ptr::null_mut();
            };

            c_string.into_raw()
        }

        unsafe extern "C" fn rust_close_file(
            _ctx: *mut sys::bt_Context,
            _path: *const std::ffi::c_char,
            handle: *mut c_void,
        ) {
            if !handle.is_null() {
                unsafe {
                    let _file = Box::from_raw(handle as *mut std::fs::File);
                }
            }
        }

        unsafe extern "C" fn rust_free_source(
            _ctx: *mut sys::bt_Context,
            source: *mut std::ffi::c_char,
        ) {
            if !source.is_null() {
                unsafe {
                    let _ = CString::from_raw(source);
                }
            }
        }

        handlers.alloc = Some(rust_alloc);
        handlers.free = Some(rust_free);
        handlers.realloc = Some(rust_realloc);
        handlers.write = Some(rust_write);
        handlers.on_error = Some(rust_on_error);
        handlers.read_file = Some(rust_read_file);
        handlers.close_file = Some(rust_close_file);
        handlers.free_source = Some(rust_free_source);
    }
}

pub struct BoltContext {
    ctx: *mut sys::bt_Context,
    _handlers: sys::bt_Handlers,
}

impl BoltContext {
    pub fn run(&self, code: &str) -> Result<(), String> {
        let c_code = CString::new(code).map_err(|_| "Invalid code")?;
        unsafe {
            let result = sys::bt_run(self.ctx, c_code.as_ptr());
            if result as u32 == BT_TRUE {
                Ok(())
            } else {
                Err("Execution failed".to_string())
            }
        }
    }
}

impl Drop for BoltContext {
    fn drop(&mut self) {
        unsafe {
            sys::bt_close(self.ctx);
        }
    }
}

/// Wrapper and context holder for the current execution thread
pub struct BoltThread {
    thread: *mut sys::bt_Thread,
}

impl BoltThread {
    pub fn from_raw(thread: *mut sys::bt_Thread) -> Self {
        Self { thread }
    }
}
