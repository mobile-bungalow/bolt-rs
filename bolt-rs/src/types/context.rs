//! Context type and all its methods
//!
//! This module contains the Context wrapper around bt_Context with both low-level
//! C API wrappers and high-level ergonomic methods.

use super::*;
use crate::{Error, wrappers::IntoCStr};
use bolt_sys::sys::{self, *};

/// Safe wrapper around bt_Context
#[derive(Debug, Clone)]
pub struct Context {
    ptr: ::std::ptr::NonNull<sys::bt_Context>,
}

impl Context {
    #[inline]
    pub fn from_raw(ptr: *mut sys::bt_Context) -> Option<Self> {
        ::std::ptr::NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    #[inline]
    pub unsafe fn from_raw_unchecked(ptr: *mut sys::bt_Context) -> Self {
        unsafe {
            Self {
                ptr: ::std::ptr::NonNull::new_unchecked(ptr),
            }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut sys::bt_Context {
        self.ptr.as_ptr()
    }
}

impl ::std::convert::AsRef<sys::bt_Context> for Context {
    #[inline]
    fn as_ref(&self) -> &sys::bt_Context {
        unsafe { self.ptr.as_ref() }
    }
}

impl ::std::convert::AsMut<sys::bt_Context> for Context {
    #[inline]
    fn as_mut(&mut self) -> &mut sys::bt_Context {
        unsafe { self.ptr.as_mut() }
    }
}

impl Context {
    bt_def!(type_any -> Type);
    bt_def!(type_null -> Type);
    bt_def!(type_number -> Type);
    bt_def!(type_bool -> Type);
    bt_def!(type_string -> Type);
    bt_def!(type_array -> Type);
    bt_def!(type_table -> Type);
    bt_def!(type_type -> Type);
    bt_def!(make_alias_type(str: &CStr, ty: Type) -> Type);
    bt_def!(make_userdata_type(name: &CStr) -> Type);
    bt_def!(make_union -> Type);
    bt_def!(make_or_extend_union(uni: Type, variant: Type) -> Type);
    bt_def!(make_array_type(inner: Type) -> Type);
    bt_def!(make_map(key: Type, value: Type) -> Type);
    bt_def_slice!(make_union_from(types: &[Type]) -> Type);
    bt_def_slice!(make_signature_type(ret: Type, types: &[Type]) -> Type);
    bt_def!(make_signature_vararg(original: Type, vararg: Type) -> Type);
    bt_def_bool!(make_tableshape_type(name: &CStr, sealed: bool) -> Type);

    pub fn make_enum_type(
        &mut self,
        name: impl IntoCStr,
        is_sealed: bool,
    ) -> Result<Type, crate::Error> {
        let c_str = name.as_c_str()?;
        unsafe {
            let name_slice = sys::bt_StrSlice {
                source: c_str.as_ptr(),
                length: c_str.to_bytes().len() as u16,
            };
            let out = Type::from_raw_unchecked(sys::bt_make_enum_type(
                self.as_ptr(),
                name_slice,
                is_sealed as sys::bt_bool,
            ));

            Ok(out)
        }
    }

    pub fn make_primitive_type<F>(
        &mut self,
        _callback: F,
        name: impl IntoCStr,
    ) -> Result<Type, crate::Error>
    where
        F: Fn(Type, Type) -> bool,
    {
        const { assert!(std::mem::size_of::<F>() == 0) }

        unsafe extern "C" fn wrapper<F>(right: *mut bt_Type, left: *mut bt_Type) -> bt_bool
        where
            F: Fn(Type, Type) -> bool,
        {
            println!("Help!!");
            unsafe {
                let callback: F = std::mem::MaybeUninit::uninit().assume_init();
                if callback(
                    Type::from_raw_unchecked(right),
                    Type::from_raw_unchecked(left),
                ) {
                    sys::BT_TRUE as u8
                } else {
                    sys::BT_FALSE as u8
                }
            }
        }

        let c_str = name.as_c_str()?;
        unsafe {
            let out =
                sys::bt_make_primitive_type(self.as_ptr(), c_str.as_ptr(), Some(wrapper::<F>));

            Ok(Type::from_raw_unchecked(out))
        }
    }

    bt_def!(union_push_variant(uni: Type, variant: Type));
    bt_def!(type_make_nullable(to_nullable: Type) -> Type);
    bt_def!(type_remove_nullable(to_unnull: Type) -> Type);
    bt_def!(tableshape_set_parent(tshp: Type, parent: Type));

    pub fn tableshape_add_layout(&mut self, tshp: Type, key_type: Type, key: Value, type_: Type) {
        unsafe {
            sys::bt_tableshape_add_layout(
                self.as_ptr(),
                tshp.as_ptr(),
                key_type.as_ptr(),
                key.0,
                type_.as_ptr(),
            )
        }
    }

    pub fn tableshape_set_field_annotations(
        &mut self,
        tshp: Type,
        key: Value,
        annotations: Annotation,
    ) {
        unsafe {
            sys::bt_tableshape_set_field_annotations(
                self.as_ptr(),
                tshp.as_ptr(),
                key.0,
                annotations.as_ptr(),
            )
        }
    }

    bt_def!(type_get_proto(ty: Type) -> Table);

    pub fn find_type(&mut self, name: Value) -> Option<Type> {
        unsafe {
            let ptr = sys::bt_find_type(self.as_ptr(), name.0);
            Type::from_raw(ptr)
        }
    }

    pub fn type_is_methodic(signature: Type, ty: Type) -> bool {
        unsafe { sys::bt_type_is_methodic(signature.as_ptr(), ty.as_ptr()) == BT_TRUE as u8 }
    }

    pub fn type_get_field(&mut self, tshp: Type, key: Value) -> Option<Value> {
        unsafe {
            let mut value = std::mem::zeroed();
            let result = sys::bt_type_get_field(self.as_ptr(), tshp.as_ptr(), key.0, &mut value);
            if result != 0 {
                Some(Value::from_raw(value))
            } else {
                None
            }
        }
    }

    pub fn type_get_field_type(&mut self, tshp: Type, key: Value) -> Option<Type> {
        unsafe {
            let ptr = sys::bt_type_get_field_type(self.as_ptr(), tshp.as_ptr(), key.0);
            Type::from_raw(ptr)
        }
    }

    pub fn type_add_field(&mut self, type_: Type, value_type: Type, name: Value, value: Value) {
        unsafe {
            sys::bt_type_add_field(
                self.as_ptr(),
                type_.as_ptr(),
                value_type.as_ptr(),
                name.0,
                value.0,
            )
        }
    }

    pub fn type_set_field(&mut self, type_: Type, name: Value, value: Value) {
        unsafe { sys::bt_type_set_field(self.as_ptr(), type_.as_ptr(), name.0, value.0) }
    }

    pub fn register_type(&mut self, name: Value, type_: Type) {
        unsafe { sys::bt_register_type(self.as_ptr(), name.0, type_.as_ptr()) }
    }

    pub fn register_prelude(&mut self, name: Value, type_: Type, value: Value) {
        unsafe { sys::bt_register_prelude(self.as_ptr(), name.0, type_.as_ptr(), value.0) }
    }

    pub fn enum_push_option(
        &mut self,
        enum_: Type,
        name: impl IntoCStr,
        value: Value,
    ) -> Result<(), crate::Error> {
        let c_str = name.as_c_str()?;
        unsafe {
            let name_slice = sys::bt_StrSlice {
                source: c_str.as_ptr(),
                length: c_str.to_bytes().len() as u16,
            };
            sys::bt_enum_push_option(self.as_ptr(), enum_.as_ptr(), name_slice, value.0);
        }
        Ok(())
    }

    pub fn enum_contains(&mut self, enum_: Type, value: Value) -> Value {
        unsafe {
            Value::from_raw(sys::bt_enum_contains(
                self.as_ptr(),
                enum_.as_ptr(),
                value.0,
            ))
        }
    }

    pub fn enum_get(&mut self, enum_: Type, name: BoltString) -> Value {
        unsafe {
            Value::from_raw(sys::bt_enum_get(
                self.as_ptr(),
                enum_.as_ptr(),
                name.as_ptr(),
            ))
        }
    }

    pub fn make_userdata(
        &mut self,
        type_: Type,
        data: *mut std::ffi::c_void,
        size: u32,
    ) -> Userdata {
        unsafe {
            Userdata::from_raw_unchecked(sys::bt_make_userdata(
                self.as_ptr(),
                type_.as_ptr(),
                data,
                size,
            ))
        }
    }

    pub fn userdata_type_push_field(
        &mut self,
        type_: Type,
        name: impl IntoCStr,
        offset: u32,
        field_type: Type,
        getter: sys::bt_UserdataFieldGetter,
        setter: sys::bt_UserdataFieldSetter,
    ) -> Result<(), crate::Error> {
        let c_str = name.as_c_str()?;
        unsafe {
            sys::bt_userdata_type_push_field(
                self.as_ptr(),
                type_.as_ptr(),
                c_str.as_ptr(),
                offset,
                field_type.as_ptr(),
                getter,
                setter,
            );
            Ok(())
        }
    }

    bt_def_userdata_field!(float);
    bt_def_userdata_field!(double);
    bt_def_userdata_field!(int8);
    bt_def_userdata_field!(int16);
    bt_def_userdata_field!(int32);
    bt_def_userdata_field!(int64);
    bt_def_userdata_field!(uint8);
    bt_def_userdata_field!(uint16);
    bt_def_userdata_field!(uint32);
    bt_def_userdata_field!(uint64);
    bt_def_userdata_field!(string);
    bt_def_userdata_field!(bool);

    bt_def!(make_string(s: &CStr) -> BoltString);
    bt_def!(make_string_hashed(s: &CStr) -> BoltString);
    bt_def!(string_concat(a: BoltString, b: BoltString) -> BoltString);
    bt_def!(remove_interned(str: BoltString));

    pub fn get_or_make_interned(&mut self, s: impl IntoCStr) -> Result<BoltString, crate::Error> {
        let c_str = s.as_c_str()?;
        unsafe {
            Ok(BoltString::from_raw_unchecked(
                sys::bt_get_or_make_interned(
                    self.as_ptr(),
                    c_str.as_ptr(),
                    c_str.to_bytes().len() as u32,
                ),
            ))
        }
    }

    pub fn string_append_cstr(
        &mut self,
        a: BoltString,
        b: impl IntoCStr,
    ) -> Result<BoltString, crate::Error> {
        let c_str = b.as_c_str()?;
        unsafe {
            Ok(BoltString::from_raw_unchecked(sys::bt_string_append_cstr(
                self.as_ptr(),
                a.as_ptr(),
                c_str.as_ptr(),
            )))
        }
    }

    pub fn make_string_len(
        &mut self,
        s: impl IntoCStr,
        len: u32,
    ) -> Result<BoltString, crate::Error> {
        let c_str = s.as_c_str()?;
        unsafe {
            Ok(BoltString::from_raw_unchecked(sys::bt_make_string_len(
                self.as_ptr(),
                c_str.as_ptr(),
                len,
            )))
        }
    }

    pub fn make_string_hashed_len(
        &mut self,
        s: impl IntoCStr,
        len: u32,
    ) -> Result<BoltString, crate::Error> {
        let c_str = s.as_c_str()?;
        unsafe {
            Ok(BoltString::from_raw_unchecked(
                sys::bt_make_string_hashed_len(self.as_ptr(), c_str.as_ptr(), len),
            ))
        }
    }

    pub fn make_string_empty(&mut self, len: u32) -> BoltString {
        unsafe { BoltString::from_raw_unchecked(sys::bt_make_string_empty(self.as_ptr(), len)) }
    }

    pub fn to_string(&mut self, value: Value) -> BoltString {
        unsafe { BoltString::from_raw_unchecked(sys::bt_to_string(self.as_ptr(), value.0)) }
    }

    pub fn to_string_inplace(&mut self, buffer: &mut [u8], value: Value) -> i32 {
        unsafe {
            sys::bt_to_string_inplace(
                self.as_ptr(),
                buffer.as_mut_ptr() as *mut i8,
                buffer.len() as u32,
                value.0,
            )
        }
    }

    pub fn make_array(&mut self, capacity: u32) -> Array {
        unsafe { Array::from_raw_unchecked(sys::bt_make_array(self.as_ptr(), capacity)) }
    }

    pub fn array_push(&mut self, arr: Array, value: Value) -> u64 {
        unsafe { sys::bt_array_push(self.as_ptr(), arr.as_ptr(), value.0) }
    }

    pub fn array_set(&mut self, arr: Array, index: u64, value: Value) -> bool {
        unsafe { sys::bt_array_set(self.as_ptr(), arr.as_ptr(), index, value.0) != 0 }
    }

    pub fn array_get(&mut self, arr: Array, index: u64) -> Value {
        unsafe { Value::from_raw(sys::bt_array_get(self.as_ptr(), arr.as_ptr(), index)) }
    }

    bt_def!(make_table_from_proto(prototype: Type) -> Table);

    pub fn make_table(&mut self, initial_size: u16) -> Table {
        unsafe { Table::from_raw_unchecked(sys::bt_make_table(self.as_ptr(), initial_size)) }
    }

    pub fn table_set(&mut self, tbl: Table, key: Value, value: Value) -> bool {
        unsafe { sys::bt_table_set(self.as_ptr(), tbl.as_ptr(), key.0, value.0) != 0 }
    }

    pub fn get(&mut self, obj: Object, key: Value) -> Value {
        unsafe { Value::from_raw(sys::bt_get(self.as_ptr(), obj.as_ptr(), key.0)) }
    }

    pub fn set(&mut self, obj: Object, key: Value, value: Value) {
        unsafe { sys::bt_set(self.as_ptr(), obj.as_ptr(), key.0, value.0) }
    }

    pub fn allocate(&mut self, full_size: u32, obj_type: sys::bt_ObjectType) -> Option<Object> {
        unsafe {
            let ptr = sys::bt_allocate(self.as_ptr(), full_size, obj_type);
            Object::from_raw(ptr)
        }
    }

    pub fn free(&mut self, obj: Object) {
        unsafe { sys::bt_free(self.as_ptr(), obj.as_ptr()) }
    }

    bt_def!(make_annotation(name: BoltString) -> Annotation);
    bt_def_opt!(annotation_next(annotation: Annotation, next_name: BoltString) -> Annotation);

    pub fn annotation_push(&mut self, annotation: Annotation, value: Value) {
        unsafe { sys::bt_annotation_push(self.as_ptr(), annotation.as_ptr(), value.0) }
    }

    bt_def!(make_module -> Module);
    bt_def_bool!(find_module(name: Value, suppress_errors: bool) -> Module);

    pub fn register_module(&mut self, name: Value, module: Module) {
        unsafe { sys::bt_register_module(self.as_ptr(), name.0, module.as_ptr()) }
    }

    pub fn module_export(&mut self, module: Module, type_: Type, key: Value, value: Value) {
        unsafe {
            sys::bt_module_export(
                self.as_ptr(),
                module.as_ptr(),
                type_.as_ptr(),
                key.0,
                value.0,
            )
        }
    }

    pub fn module_export_native(
        &mut self,
        module: Module,
        name: impl IntoCStr,
        proc: sys::bt_NativeProc,
        ret_type: Type,
        args: &[Type],
    ) -> Result<(), crate::Error> {
        let c_str = name.as_c_str()?;
        unsafe {
            let arg_ptrs: Vec<*mut sys::bt_Type> = args.iter().map(|t| t.as_ptr()).collect();
            sys::bt_module_export_native(
                self.as_ptr(),
                module.as_ptr(),
                c_str.as_ptr(),
                proc,
                ret_type.as_ptr(),
                arg_ptrs.as_ptr() as *mut *mut sys::bt_Type,
                args.len() as u8,
            );
        }
        Ok(())
    }

    pub fn append_module_path(&mut self, spec: impl IntoCStr) -> Result<(), crate::Error> {
        let c_str = spec.as_c_str();
        unsafe {
            sys::bt_append_module_path(self.as_ptr(), c_str?.as_ptr());
        }
        Ok(())
    }

    pub fn compile_module(
        &mut self,
        source: impl IntoCStr,
        mod_name: impl IntoCStr,
    ) -> Result<Module, crate::Error> {
        let source_c = source.as_c_str()?;
        let name_c = mod_name.as_c_str()?;
        unsafe {
            let ptr = sys::bt_compile_module(self.as_ptr(), source_c.as_ptr(), name_c.as_ptr());
            Module::from_raw(ptr).ok_or(Error::bolt("Module failed to compile"))
        }
    }

    pub fn make_native(
        &mut self,
        module: Module,
        signature: Type,
        proc: sys::bt_NativeProc,
    ) -> NativeFn {
        unsafe {
            NativeFn::from_raw_unchecked(sys::bt_make_native(
                self.as_ptr(),
                module.as_ptr(),
                signature.as_ptr(),
                proc,
            ))
        }
    }

    bt_def!(make_thread -> Thread);
    bt_def!(destroy_thread(thread: Thread));

    bt_def_prim!(gc_pause);
    bt_def_prim!(gc_unpause);
    bt_def_prim!(pop_root);
    bt_def!(push_root(root: Object));
    bt_def!(grey_obj(obj: Object));
    bt_def_prim!(add_ref(obj: Object) -> u32);
    bt_def_prim!(remove_ref(obj: Object) -> u32);

    bt_def_prim!(gc_get_next_cycle -> usize);
    bt_def_prim!(gc_set_next_cycle(next_cycle: usize));
    bt_def_prim!(gc_get_min_size -> usize);
    bt_def_prim!(gc_set_min_size(min_size: usize));
    bt_def_prim!(gc_get_grey_cap -> u32);
    bt_def_prim!(gc_set_grey_cap(grey_cap: u32));
    bt_def_prim!(gc_get_growth_pct -> usize);
    bt_def_prim!(gc_set_growth_pct(growth_pct: usize));
    bt_def_prim!(gc_get_pause_growth_pct -> usize);
    bt_def_prim!(gc_set_pause_growth_pct(growth_pct: usize));
    bt_def!(destroy_gc(gc: GC));

    pub fn make_gc(&mut self) {
        unsafe { sys::bt_make_gc(self.as_ptr()) }
    }

    pub fn gc_alloc(&mut self, size: usize) -> *mut std::ffi::c_void {
        unsafe { sys::bt_gc_alloc(self.as_ptr(), size) }
    }

    pub fn gc_realloc(
        &mut self,
        ptr: *mut std::ffi::c_void,
        old_size: usize,
        new_size: usize,
    ) -> *mut std::ffi::c_void {
        unsafe { sys::bt_gc_realloc(self.as_ptr(), ptr, old_size, new_size) }
    }

    pub fn gc_free(&mut self, ptr: *mut std::ffi::c_void, size: usize) {
        unsafe { sys::bt_gc_free(self.as_ptr(), ptr, size) }
    }

    /// Create a new Context with Rust-based handlers for allocation, I/O, and error reporting
    pub fn new() -> Self {
        unsafe {
            let mut handlers = sys::bt_default_handlers();
            Self::override_handlers(&mut handlers);
            let mut ctx = std::ptr::null_mut();
            sys::bt_open(&mut ctx, &mut handlers);
            Context::from_raw(ctx).expect("Failed to create context")
        }
    }

    fn override_handlers(handlers: &mut sys::bt_Handlers) {
        unsafe extern "C" fn rust_alloc(size: usize) -> *mut std::ffi::c_void {
            unsafe {
                std::alloc::alloc(std::alloc::Layout::array::<u8>(size).unwrap_unchecked()) as _
            }
        }

        unsafe extern "C" fn rust_free(ptr: *mut std::ffi::c_void) {
            if !ptr.is_null() {
                unsafe { std::alloc::dealloc(ptr as *mut u8, std::alloc::Layout::new::<u8>()) }
            }
        }

        unsafe extern "C" fn rust_realloc(
            ptr: *mut std::ffi::c_void,
            size: usize,
        ) -> *mut std::ffi::c_void {
            if ptr.is_null() {
                unsafe {
                    std::alloc::alloc(std::alloc::Layout::array::<u8>(size).unwrap_unchecked()) as _
                }
            } else {
                unsafe {
                    std::alloc::realloc(ptr as *mut u8, std::alloc::Layout::new::<u8>(), size) as _
                }
            }
        }

        unsafe extern "C" fn rust_write(_ctx: *mut sys::bt_Context, msg: *const std::ffi::c_char) {
            if !msg.is_null()
                && let Ok(msg_str) = unsafe { std::ffi::CStr::from_ptr(msg) }.to_str() {
                    print!("{}", msg_str);
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
            out_handle: *mut *mut std::ffi::c_void,
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

            let Ok(c_string) = std::ffi::CString::new(contents) else {
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
            handle: *mut std::ffi::c_void,
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
                    let _ = std::ffi::CString::from_raw(source);
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

    /// Open all standard library modules
    pub fn open_all_std(&mut self) {
        unsafe {
            sys::boltstd_open_all(self.as_ptr());
        }
    }

    /// Open the core standard library module
    pub fn open_core(&mut self) {
        unsafe {
            sys::boltstd_open_core(self.as_ptr());
        }
    }

    /// Open the arrays standard library module
    pub fn open_arrays(&mut self) {
        unsafe {
            sys::boltstd_open_arrays(self.as_ptr());
        }
    }

    /// Open the strings standard library module
    pub fn open_strings(&mut self) {
        unsafe {
            sys::boltstd_open_strings(self.as_ptr());
        }
    }

    /// Open the tables standard library module
    pub fn open_tables(&mut self) {
        unsafe {
            sys::boltstd_open_tables(self.as_ptr());
        }
    }

    /// Open the math standard library module
    pub fn open_math(&mut self) {
        unsafe {
            sys::boltstd_open_math(self.as_ptr());
        }
    }

    /// Open the I/O standard library module
    pub fn open_io(&mut self) {
        unsafe {
            sys::boltstd_open_io(self.as_ptr());
        }
    }

    /// Open the meta-programming standard library module
    pub fn open_meta(&mut self) {
        unsafe {
            sys::boltstd_open_meta(self.as_ptr());
        }
    }

    /// Open the regex standard library module
    pub fn open_regex(&mut self) {
        unsafe {
            sys::boltstd_open_regex(self.as_ptr());
        }
    }

    pub fn run(&mut self, code: impl crate::IntoCStr) -> Result<(), crate::Error> {
        unsafe {
            if sys::bt_run(self.as_ptr(), code.as_c_str()?.as_ptr()) == BT_TRUE as u8 {
                Ok(())
            } else {
                Err(Error::bolt("Execution failed"))
            }
        }
    }

    pub fn create_module(&mut self, name: &str) -> Result<Module, crate::ModuleError> {
        use crate::types::value::MakeBoltValueWithContext;

        let module = self.make_module();
        let name_value = name.make_with_context(self);
        self.register_module(Value::from_raw(name_value), module);
        Ok(module)
    }

    pub fn get_module(&mut self, name: &str) -> Result<Module, crate::ModuleError> {
        use crate::types::value::MakeBoltValueWithContext;

        let name_value = name.make_with_context(self);
        self.find_module(Value::from_raw(name_value), false)
            .ok_or_else(|| crate::ModuleError::NotFound(name.to_string()))
    }
}

impl Drop for Context {
    fn drop(&mut self) {
        unsafe {
            sys::bt_close(self.as_ptr());
        }
    }
}
