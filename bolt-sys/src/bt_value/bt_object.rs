use crate::{
    ArgError, BoltContext,
    bt_value::{CallSignature, FromBoltValue, MakeBoltValue, MakeBoltValueWithContext, Value, ValueType},
    sys::{self, *},
};
use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    ptr::NonNull,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObjectType {
    None,
    Type,
    String,
    Module,
    Import,
    Function,
    NativeFunction,
    Closure,
    Array,
    Table,
    UserData,
    Annotation,
}

impl ObjectType {
    /// Create an ObjectType from a bt_Object mask field
    pub fn from_mask(mask: u64) -> Self {
        let type_value = sys::object_mask::get_type(mask);
        Self::from(type_value)
    }
}

#[derive(Debug)]
pub enum BoltObject {
    /// No object or invalid object
    None,
    /// Type object
    Type(Type),
    /// String object containing UTF-8 text
    String(BoltString),
    /// Compiled bolt module
    Module(Module),
    /// Module import reference
    Import(Import),
    /// Bolt-defined function
    Function(Function),
    /// Native function reference
    NativeFunction(NativeFunction),
    /// Closure with captured upvalues
    Closure(Closure),
    /// Dynamic array
    Array(Array),
    /// Key-value table/map
    Table(Table),
    /// Opaque user data
    UserData(UserData),
    /// Type or field annotation
    Annotation(Annotation),
}

#[derive(Debug, Copy, Clone)]
pub struct Type {
    ptr: NonNull<bt_Type>,
}

impl Type {
    pub fn from_raw(ptr: *mut bt_Type) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
        })
    }

    pub fn as_raw(&self) -> *mut bt_Type {
        self.ptr.as_ptr()
    }
}

#[derive(Debug)]
pub struct BoltString {
    ptr: NonNull<bt_String>,
    /// Cached string value for efficiency
    value: Option<String>,
}

#[derive(Debug)]
pub struct Module {
    pub ptr: NonNull<bt_Module>,
    /// Module name if available
    pub name: Option<String>,
    /// Module path if available
    pub path: Option<String>,
}

impl Module {
    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn path(&self) -> Option<&str> {
        self.path.as_deref()
    }
}

#[derive(Debug)]
pub struct Import {
    ptr: NonNull<bt_ModuleImport>,
    /// Import name
    name: Option<String>,
}

#[derive(Debug)]
pub struct Function {
    ptr: NonNull<bt_Fn>,
    /// Function signature type if available
    signature: Option<NonNull<bt_Type>>,
}

#[derive(Debug)]
pub struct NativeFunction {
    ptr: NonNull<bt_NativeFn>,
    /// Function type signature if available
    type_: Option<NonNull<bt_Type>>,
}

impl NativeFunction {
    pub fn from_raw(ptr: *mut bt_NativeFn) -> Option<Self> {
        Some(Self {
            ptr: NonNull::new(ptr)?,
            type_: None,
        })
    }

    pub fn as_raw(&self) -> *mut bt_NativeFn {
        self.ptr.as_ptr()
    }
}

#[derive(Debug)]
pub struct Closure {
    ptr: NonNull<bt_Closure>,
    /// Number of upvalues
    num_upvalues: u32,
}

#[derive(Debug)]
pub struct Array {
    ptr: NonNull<bt_Array>,
    /// Array length
    length: u32,
    /// Array capacity
    capacity: u32,
}

#[derive(Debug)]
pub struct Table {
    ptr: NonNull<bt_Table>,
    /// Number of key-value pairs
    length: u16,
    /// Table capacity
    capacity: u16,
    /// Whether the table uses inline storage
    is_inline: bool,
}

#[derive(Debug)]
pub struct UserData {
    ptr: NonNull<bt_Userdata>,
    /// Size of user data
    size: usize,
    /// Associated type if available
    type_: Option<NonNull<bt_Type>>,
}

#[derive(Debug)]
pub struct Annotation {
    ptr: NonNull<bt_Annotation>,
    /// Annotation name
    name: Option<String>,
    /// Next annotation in chain if present
    next: Option<NonNull<bt_Annotation>>,
}

impl BoltObject {
    /// Create a BoltObject from a raw bt_Object pointer
    /// Bolt objects
    pub unsafe fn from_raw(ptr: *mut bt_Object) -> Option<Self> {
        if ptr.is_null() {
            return None;
        }

        let mask = unsafe { (*ptr).mask };
        let object_type = ObjectType::from_mask(mask);

        match object_type {
            ObjectType::None => Some(BoltObject::None),
            ObjectType::Type => {
                NonNull::new(ptr as *mut bt_Type).map(|ptr| BoltObject::Type(Type { ptr }))
            }
            ObjectType::String => NonNull::new(ptr as *mut bt_String)
                .map(|ptr| BoltObject::String(BoltString { ptr, value: None })),
            ObjectType::Module => NonNull::new(ptr as *mut bt_Module).map(|ptr| {
                BoltObject::Module(Module {
                    ptr,
                    name: None,
                    path: None,
                })
            }),
            ObjectType::Import => NonNull::new(ptr as *mut bt_ModuleImport)
                .map(|ptr| BoltObject::Import(Import { ptr, name: None })),
            ObjectType::Function => NonNull::new(ptr as *mut bt_Fn).map(|ptr| {
                let fn_obj = unsafe { &*ptr.as_ptr() };
                BoltObject::Function(Function {
                    ptr,
                    signature: NonNull::new(fn_obj.signature),
                })
            }),
            ObjectType::NativeFunction => NonNull::new(ptr as *mut bt_NativeFn).map(|ptr| {
                let native_fn = unsafe { &*ptr.as_ptr() };
                BoltObject::NativeFunction(NativeFunction {
                    ptr,
                    type_: NonNull::new(native_fn.type_),
                })
            }),
            ObjectType::Closure => NonNull::new(ptr as *mut bt_Closure).map(|ptr| {
                let closure = unsafe { &*ptr.as_ptr() };
                BoltObject::Closure(Closure {
                    ptr,
                    num_upvalues: closure.num_upv,
                })
            }),
            ObjectType::Array => NonNull::new(ptr as *mut bt_Array).map(|ptr| {
                let array = unsafe { &*ptr.as_ptr() };
                BoltObject::Array(Array {
                    ptr,
                    length: array.length,
                    capacity: array.capacity,
                })
            }),
            ObjectType::Table => NonNull::new(ptr as *mut bt_Table).map(|ptr| {
                let table = unsafe { &*ptr.as_ptr() };
                BoltObject::Table(Table {
                    ptr,
                    length: table.length,
                    capacity: table.capacity,
                    is_inline: table.is_inline != 0,
                })
            }),
            ObjectType::UserData => NonNull::new(ptr as *mut bt_Userdata).map(|ptr| {
                let userdata = unsafe { &*ptr.as_ptr() };
                BoltObject::UserData(UserData {
                    ptr,
                    size: userdata.size,
                    type_: NonNull::new(userdata.type_),
                })
            }),
            ObjectType::Annotation => NonNull::new(ptr as *mut bt_Annotation).map(|ptr| {
                let annotation = unsafe { &*ptr.as_ptr() };
                BoltObject::Annotation(Annotation {
                    ptr,
                    name: None,
                    next: NonNull::new(annotation.next),
                })
            }),
        }
    }

    /// Get the object type of this BoltObject
    pub fn object_type(&self) -> ObjectType {
        match self {
            BoltObject::None => ObjectType::None,
            BoltObject::Type(_) => ObjectType::Type,
            BoltObject::String(_) => ObjectType::String,
            BoltObject::Module(_) => ObjectType::Module,
            BoltObject::Import(_) => ObjectType::Import,
            BoltObject::Function(_) => ObjectType::Function,
            BoltObject::NativeFunction(_) => ObjectType::NativeFunction,
            BoltObject::Closure(_) => ObjectType::Closure,
            BoltObject::Array(_) => ObjectType::Array,
            BoltObject::Table(_) => ObjectType::Table,
            BoltObject::UserData(_) => ObjectType::UserData,
            BoltObject::Annotation(_) => ObjectType::Annotation,
        }
    }

    /// Check if this is a None/invalid object
    pub fn is_none(&self) -> bool {
        matches!(self, BoltObject::None)
    }

    /// Get the raw pointer to the underlying object
    ///
    /// # Safety
    /// The returned pointer is only valid as long as the BoltObject exists
    pub unsafe fn as_raw_ptr(&self) -> *mut bt_Object {
        match self {
            BoltObject::None => std::ptr::null_mut(),
            BoltObject::Type(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::String(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Module(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Import(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Function(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::NativeFunction(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Closure(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Array(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Table(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::UserData(obj) => obj.ptr.as_ptr() as *mut bt_Object,
            BoltObject::Annotation(obj) => obj.ptr.as_ptr() as *mut bt_Object,
        }
    }
}

pub struct Object {
    ptr: NonNull<bt_Object>,
}

// FromBoltValue implementations for object types
impl FromBoltValue for BoltObject {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        unsafe {
            if bt_is_object(val) != 0 {
                let obj_ptr = bt_object(val);
                BoltObject::from_raw(obj_ptr).ok_or(ArgError::TypeGuard {
                    expected: ValueType::None,
                    actual: ValueType::None,
                })
            } else {
                Err(ArgError::TypeGuard {
                    expected: ValueType::None,
                    actual: ValueType::None,
                })
            }
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            let obj_ptr = bt_object(val);
            BoltObject::from_raw(obj_ptr).unwrap_or(BoltObject::None)
        }
    }
}

impl MakeBoltValue for BoltObject {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.as_raw_ptr()) }
    }
}

// Implementations for specific object types
impl FromBoltValue for Type {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::Type(type_obj) => Ok(type_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::Type,
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::Type(type_obj) => type_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for Type {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl FromBoltValue for BoltString {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::String(string_obj) => Ok(string_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::String,
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::String(string_obj) => string_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for BoltString {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl FromBoltValue for Module {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::Module(module_obj) => Ok(module_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::Module,
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::Module(module_obj) => module_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for Module {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl FromBoltValue for Function {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::Function(fn_obj) => Ok(fn_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::Function(CallSignature { args: vec![], return_ty: Type::from_raw(std::ptr::null_mut()).unwrap_or_else(|| panic!("null type")) }),
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::Function(fn_obj) => fn_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for Function {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl FromBoltValue for Array {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::Array(array_obj) => Ok(array_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::Array(vec![]),
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::Array(array_obj) => array_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for Array {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl FromBoltValue for Table {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        let obj = <BoltObject as FromBoltValue>::from(val)?;
        match obj {
            BoltObject::Table(table_obj) => Ok(table_obj),
            _ => Err(ArgError::TypeGuard {
                expected: ValueType::Table(HashMap::new()),
                actual: ValueType::None,
            }),
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe {
            match BoltObject::from_unchecked(val) {
                BoltObject::Table(table_obj) => table_obj,
                _ => unreachable!(),
            }
        }
    }
}

impl MakeBoltValue for Table {
    fn make(&self) -> bt_Value {
        unsafe { bt_value(self.ptr.as_ptr() as *mut bt_Object) }
    }
}

impl MakeBoltValueWithContext for &str {
    fn make_with_context(&self, ctx: &mut BoltContext) -> bt_Value {
        unsafe {
            let c_str = CString::new(*self).unwrap_or_else(|_| CString::new("").unwrap());
            let string_obj = bt_make_string(ctx.as_ptr(), c_str.as_ptr());
            bt_value(string_obj as *mut bt_Object)
        }
    }
}

impl MakeBoltValueWithContext for String {
    fn make_with_context(&self, ctx: &mut BoltContext) -> bt_Value {
        self.as_str().make_with_context(ctx)
    }
}

impl MakeBoltValueWithContext for CString {
    fn make_with_context(&self, ctx: &mut BoltContext) -> bt_Value {
        unsafe {
            let string_obj = bt_make_string(ctx.as_ptr(), self.as_ptr());
            bt_value(string_obj as *mut bt_Object)
        }
    }
}

impl MakeBoltValueWithContext for &CStr {
    fn make_with_context(&self, ctx: &mut BoltContext) -> bt_Value {
        unsafe {
            let string_obj = bt_make_string(ctx.as_ptr(), self.as_ptr());
            bt_value(string_obj as *mut bt_Object)
        }
    }
}
