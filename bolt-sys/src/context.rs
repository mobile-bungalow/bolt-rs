use crate::{
    ArgError, ModuleError,
    bt_object::{Module, NativeFunction, Type},
    bt_value::{FromBoltValue, MakeBoltValue, MakeBoltValueWithContext, Value, ValueType},
    sys::{self, BT_TRUE, bt_arg, bt_argc, bt_make_native, bt_return},
};

use std::{
    alloc::Layout,
    ffi::{CString, c_void},
    ptr::NonNull,
};

type NativeFn = unsafe extern "C" fn(ctx: *mut sys::bt_Context, thread: *mut sys::bt_Thread);

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
            BoltContext { ctx }
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

    /// Resolves the type of a bt_Value
    pub fn resolve_type(&mut self, value: &Value) -> ValueType {
        todo!()
    }

    pub fn from_raw(ptr: *mut sys::bt_Context) -> Self {
        Self { ctx: ptr }
    }

    pub fn make_thread(&mut self) -> Option<BoltThread> {
        unsafe {
            let thread = sys::bt_make_thread(self.ctx);
            BoltThread::from_raw(thread)
        }
    }

    pub fn destroy_thread(&mut self, thread: BoltThread) {
        unsafe {
            sys::bt_destroy_thread(self.ctx, thread.thread.as_ptr());
        }
    }

    pub fn execute_on_thread<T: MakeBoltValue>(
        &mut self,
        thread: &mut BoltThread,
        callable: &T,
    ) -> bool {
        unsafe {
            sys::bt_execute_on_thread(
                self.ctx,
                thread.thread.as_ptr(),
                callable.make() as *mut sys::bt_Callable,
            ) != 0
        }
    }

    pub fn make_module(&mut self) -> Option<Module> {
        unsafe {
            let module_ptr = sys::bt_make_module(self.ctx);
            if module_ptr.is_null() {
                None
            } else {
                Some(Module {
                    ptr: std::ptr::NonNull::new_unchecked(module_ptr),
                    name: None,
                    path: None,
                })
            }
        }
    }

    pub fn make_native_function(
        &mut self,
        module: &Module,
        signature: &Type,
        callback: NativeFn,
    ) -> NativeFunction {
        let func = unsafe {
            bt_make_native(
                self.as_ptr(),
                module.ptr.as_ptr(),
                signature.as_raw(),
                Some(callback),
            )
        };

        NativeFunction::from_raw(func).expect("Deal with me later")
    }

    pub fn register_module(&mut self, name: &str, module: &Module) -> Result<(), ModuleError> {
        let name_value = name.make_with_context(self);
        unsafe {
            sys::bt_register_module(self.ctx, name_value, module.ptr.as_ptr());
        }
        Ok(())
    }

    pub fn find_module(&mut self, name: &str) -> Result<Module, ModuleError> {
        let name_value = name.make_with_context(self);
        unsafe {
            let module_ptr = sys::bt_find_module(self.ctx, name_value, 0); // 0 = don't suppress errors

            if module_ptr.is_null() {
                Err(ModuleError::NotFound(name.to_string()))
            } else {
                Ok(Module {
                    ptr: std::ptr::NonNull::new_unchecked(module_ptr),
                    name: Some(name.to_string()),
                    path: None,
                })
            }
        }
    }

    pub fn module_export(
        &mut self,
        module: &Module,
        signature: Type,
        name: &str,
        func: NativeFunction,
    ) {
        let c_name = CString::new(name).expect("Failed to create CString");
        unsafe {
            let func_ptr = func.as_raw();
            let func_value = sys::bt_value(&mut (*func_ptr).obj as *mut sys::bt_Object);
            let name_string = sys::bt_make_string(self.ctx, c_name.as_ptr());
            let name_value = sys::bt_value(name_string as *mut sys::bt_Object);

            sys::bt_module_export(
                self.ctx,
                module.ptr.as_ptr(),
                signature.as_raw(),
                name_value,
                func_value,
            );
        }
    }

    pub fn as_ptr(&self) -> *mut sys::bt_Context {
        self.ctx
    }

    pub fn open_all_std(&mut self) {
        unsafe {
            sys::boltstd_open_all(self.ctx);
        }
    }

    pub fn open_core(&mut self) {
        unsafe {
            sys::boltstd_open_core(self.ctx);
        }
    }

    pub fn open_arrays(&mut self) {
        unsafe {
            sys::boltstd_open_arrays(self.ctx);
        }
    }

    pub fn open_strings(&mut self) {
        unsafe {
            sys::boltstd_open_strings(self.ctx);
        }
    }

    pub fn open_tables(&mut self) {
        unsafe {
            sys::boltstd_open_tables(self.ctx);
        }
    }

    pub fn open_math(&mut self) {
        unsafe {
            sys::boltstd_open_math(self.ctx);
        }
    }

    pub fn open_io(&mut self) {
        unsafe {
            sys::boltstd_open_io(self.ctx);
        }
    }

    pub fn open_meta(&mut self) {
        unsafe {
            sys::boltstd_open_meta(self.ctx);
        }
    }

    pub fn open_regex(&mut self) {
        unsafe {
            sys::boltstd_open_regex(self.ctx);
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
    thread: NonNull<sys::bt_Thread>,
}

impl BoltThread {
    pub fn from_raw(thread: *mut sys::bt_Thread) -> Option<Self> {
        Some(Self {
            thread: NonNull::new(thread)?,
        })
    }

    pub fn return_val<T: MakeBoltValue>(&mut self, val: &T) {
        unsafe { bt_return(self.thread.as_mut(), val.make()) }
    }

    pub fn get_arg<T: FromBoltValue>(&mut self, idx: u8) -> Result<T, ArgError> {
        let val = unsafe {
            let len = bt_argc(self.thread.as_mut());
            if len <= idx {
                return Err(ArgError::IndexOutOfBounds { idx, len });
            }
            bt_arg(self.thread.as_mut(), idx)
        };
        T::from(val)
    }

    pub unsafe fn get_arg_unchecked<T: FromBoltValue>(&mut self, idx: u8) -> T {
        unsafe {
            let val = bt_arg(self.thread.as_mut(), idx);
            T::from_unchecked(val)
        }
    }

    pub fn push<T: MakeBoltValue>(&mut self, value: &T) {
        unsafe {
            sys::bt_push(self.thread.as_ptr(), value.make());
        }
    }

    pub fn call(&mut self, argc: u8) {
        unsafe {
            sys::bt_call(self.thread.as_ptr(), argc);
        }
    }

    pub fn get_returned<T: FromBoltValue>(&self) -> Result<T, ArgError> {
        unsafe {
            let val = sys::bt_get_returned(self.thread.as_ptr());
            T::from(val)
        }
    }

    pub unsafe fn get_returned_unchecked<T: FromBoltValue>(&self) -> T {
        unsafe {
            let val = sys::bt_get_returned(self.thread.as_ptr());
            T::from_unchecked(val)
        }
    }

    pub fn as_ptr(&self) -> *mut sys::bt_Thread {
        self.thread.as_ptr()
    }

    pub fn argc(&self) -> u8 {
        unsafe { bt_argc(self.thread.as_ptr()) }
    }
}
