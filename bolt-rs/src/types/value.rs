use super::*;
use bolt_sys::sys;
use std::ffi::CString;

use crate::{ArgError, Context};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(transparent)]
pub struct Value(pub sys::bt_Value);

impl Value {
    #[inline]
    pub fn from_raw(val: sys::bt_Value) -> Self {
        Self(val)
    }

    #[inline]
    pub fn as_raw(&self) -> sys::bt_Value {
        self.0
    }

    #[inline]
    pub fn is_number(&self) -> bool {
        unsafe { sys::bt_is_number(self.0) != 0 }
    }

    #[inline]
    pub fn is_bool(&self) -> bool {
        unsafe { sys::bt_is_bool(self.0) != 0 }
    }

    #[inline]
    pub fn is_null(&self) -> bool {
        unsafe { sys::bt_is_null(self.0) != 0 }
    }

    #[inline]
    pub fn is_enum(&self) -> bool {
        unsafe { sys::bt_is_enum_val(self.0) != 0 }
    }

    #[inline]
    pub fn is_object(&self) -> bool {
        unsafe { sys::bt_is_object(self.0) != 0 }
    }

    #[inline]
    pub fn as_number(&self) -> Option<f64> {
        if self.is_number() {
            Some(unsafe { sys::bt_get_number(self.0) })
        } else {
            None
        }
    }

    #[inline]
    pub fn as_bool(&self) -> Option<bool> {
        if self.is_bool() {
            Some(unsafe { sys::bt_get_bool(self.0) != 0 })
        } else {
            None
        }
    }

    #[inline]
    pub fn as_enum(&self) -> Option<u32> {
        if self.is_enum() {
            Some(unsafe { sys::bt_get_enum_val(self.0) })
        } else {
            None
        }
    }

    #[inline]
    pub fn as_object(&self) -> Option<Object> {
        if self.is_object() {
            Object::from_raw(unsafe { sys::bt_object(self.0) })
        } else {
            None
        }
    }
}

impl From<sys::bt_Value> for Value {
    fn from(val: sys::bt_Value) -> Self {
        Self(val)
    }
}

impl From<Value> for sys::bt_Value {
    fn from(val: Value) -> Self {
        val.0
    }
}

/// Non-scalar compound types that need more information to create reflection info
pub trait TypeSignature: Sized {
    fn make_type(&self, ctx: &mut Context) -> Type;
}

/// Scalar and simple types only need to know which type they are to produce reflection info
pub trait ScalarTypeSignature: Sized {
    fn make_type(ctx: &mut Context) -> Type;
}

/// Types which can be extracted from a Bolt value
pub trait FromBoltValue: Sized {
    fn from(val: sys::bt_Value) -> Result<Self, ArgError>;
    unsafe fn from_unchecked(val: sys::bt_Value) -> Self;
}

/// Types which can be boxed into bolt values for use in function calls and return values
/// without help from the context.
pub trait MakeBoltValue: Sized {
    fn make(&self) -> sys::bt_Value;
}

/// Types which can be boxed into bolt values for use in function calls and return values
/// but that need help from the context.
pub trait MakeBoltValueWithContext: Sized {
    fn make_with_context(&self, ctx: &mut Context) -> sys::bt_Value;
}

#[derive(Debug, Clone)]
pub struct CallSignature {
    pub args: Vec<Type>,
    pub return_ty: Type,
}

impl CallSignature {
    pub fn make_type(&self, ctx: &mut Context) -> Type {
        unsafe {
            let mut arg_ptrs: Vec<_> = self.args.iter().map(|t| t.as_ptr()).collect();

            let type_ptr = sys::bt_make_signature_type(
                ctx.as_ptr(),
                self.return_ty.as_ptr(),
                arg_ptrs.as_mut_ptr(),
                self.args.len() as u8,
            );

            Type::from_raw(type_ptr).expect("Failed to create signature type")
        }
    }
}

#[derive(Debug, Clone)]
pub enum ValueType {
    Null,
    Bool,
    Number,
    Enum,
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

impl ValueType {
    /// A slow exhaustive check to see what type a bt_Value is
    pub fn from_value(val: sys::bt_Value) -> Self {
        let value = Value::from_raw(val);

        if value.is_null() {
            return ValueType::Null;
        }
        if value.is_bool() {
            return ValueType::Bool;
        }
        if value.is_number() {
            return ValueType::Number;
        }
        if value.is_enum() {
            return ValueType::Enum;
        }
        if let Some(obj) = value.as_object() {
            return obj.value_type();
        }

        ValueType::None
    }
}

// Scalar implementations
impl ScalarTypeSignature for f64 {
    fn make_type(ctx: &mut Context) -> Type {
        unsafe {
            let type_ptr = sys::bt_type_number(ctx.as_ptr());
            Type::from_raw(type_ptr).expect("Failed to get number type")
        }
    }
}

impl FromBoltValue for f64 {
    fn from(val: sys::bt_Value) -> Result<Self, ArgError> {
        unsafe {
            if sys::bt_is_number(val) != 0 {
                Ok(sys::bt_get_number(val))
            } else {
                Err(ArgError::TypeGuard {
                    expected: ValueType::Number,
                    actual: ValueType::from_value(val),
                })
            }
        }
    }

    unsafe fn from_unchecked(val: sys::bt_Value) -> Self {
        unsafe { sys::bt_get_number(val) }
    }
}

impl MakeBoltValue for f64 {
    fn make(&self) -> sys::bt_Value {
        unsafe { sys::bt_make_number(*self) }
    }
}

// String implementations
impl MakeBoltValueWithContext for &str {
    fn make_with_context(&self, ctx: &mut Context) -> sys::bt_Value {
        unsafe {
            let c_str = CString::new(*self).unwrap_or_else(|_| CString::new("").unwrap());
            let string_obj = sys::bt_make_string(ctx.as_ptr(), c_str.as_ptr());
            sys::bt_value(string_obj as *mut sys::bt_Object)
        }
    }
}

impl MakeBoltValueWithContext for String {
    fn make_with_context(&self, ctx: &mut Context) -> sys::bt_Value {
        self.as_str().make_with_context(ctx)
    }
}

impl MakeBoltValueWithContext for CString {
    fn make_with_context(&self, ctx: &mut Context) -> sys::bt_Value {
        unsafe {
            let string_obj = sys::bt_make_string(ctx.as_ptr(), self.as_ptr());
            sys::bt_value(string_obj as *mut sys::bt_Object)
        }
    }
}

impl MakeBoltValueWithContext for &std::ffi::CStr {
    fn make_with_context(&self, ctx: &mut Context) -> sys::bt_Value {
        unsafe {
            let string_obj = sys::bt_make_string(ctx.as_ptr(), self.as_ptr());
            sys::bt_value(string_obj as *mut sys::bt_Object)
        }
    }
}

// Type wrapper implementations
impl FromBoltValue for Type {
    fn from(val: sys::bt_Value) -> Result<Self, ArgError> {
        unsafe {
            if sys::bt_is_object(val) == 0 {
                return Err(ArgError::TypeGuard {
                    expected: ValueType::Type,
                    actual: ValueType::from_value(val),
                });
            }

            let obj_ptr = sys::bt_object(val);
            Type::from_raw(obj_ptr as *mut sys::bt_Type).ok_or(ArgError::TypeGuard {
                expected: ValueType::Type,
                actual: ValueType::None,
            })
        }
    }

    unsafe fn from_unchecked(val: sys::bt_Value) -> Self {
        unsafe {
            let obj_ptr = sys::bt_object(val);
            Type::from_raw_unchecked(obj_ptr as *mut sys::bt_Type)
        }
    }
}

impl MakeBoltValue for Type {
    fn make(&self) -> sys::bt_Value {
        unsafe { sys::bt_value(self.as_ptr() as *mut sys::bt_Object) }
    }
}

// Module wrapper implementations
impl FromBoltValue for Module {
    fn from(val: sys::bt_Value) -> Result<Self, ArgError> {
        unsafe {
            if sys::bt_is_object(val) == 0 {
                return Err(ArgError::TypeGuard {
                    expected: ValueType::Module,
                    actual: ValueType::from_value(val),
                });
            }

            let obj_ptr = sys::bt_object(val);
            Module::from_raw(obj_ptr as *mut sys::bt_Module).ok_or(ArgError::TypeGuard {
                expected: ValueType::Module,
                actual: ValueType::None,
            })
        }
    }

    unsafe fn from_unchecked(val: sys::bt_Value) -> Self {
        unsafe {
            let obj_ptr = sys::bt_object(val);
            Module::from_raw_unchecked(obj_ptr as *mut sys::bt_Module)
        }
    }
}

impl MakeBoltValue for Module {
    fn make(&self) -> sys::bt_Value {
        unsafe { sys::bt_value(self.as_ptr() as *mut sys::bt_Object) }
    }
}
