// Macros for wrapping all of the methods on bolt types
use std::borrow::Cow;
use std::ffi::{CStr, CString, NulError};

/// Helper trait to accept both &str and &CStr with zero overhead
///
/// # Panics
/// Panics if the string contains a null byte when converting from `&str` or `String`.
/// Use `CString` or `&CStr` directly if you want to invoke without allocating.
pub trait IntoCStr {
    fn as_c_str(&self) -> Result<Cow<'_, CStr>, NulError>;
}

impl IntoCStr for &str {
    fn as_c_str(&self) -> Result<Cow<'_, CStr>, NulError> {
        CString::new(*self).map(Cow::Owned)
    }
}

impl IntoCStr for &CStr {
    fn as_c_str(&self) -> Result<Cow<'_, CStr>, NulError> {
        Ok(Cow::Borrowed(self))
    }
}

impl IntoCStr for String {
    fn as_c_str(&self) -> Result<Cow<'_, CStr>, NulError> {
        CString::new(self.as_str()).map(Cow::Owned)
    }
}

impl IntoCStr for CString {
    fn as_c_str(&self) -> Result<Cow<'_, CStr>, NulError> {
        Ok(Cow::Borrowed(self.as_c_str()))
    }
}

/// Wraps a bolt method with only wrapper args and wrapper returns
///
/// # Usage
/// ```ignore
/// bt_def!(type_any -> Type);  // No args, returns Type
/// bt_def!(make_alias_type(str: &CStr, ty: Type) -> Type);  // With args
/// bt_def!(union_push_variant(uni: Type, variant: Type));  // No return
/// ```
///
/// Add documentation by placing doc comments above the macro invocation:
/// ```ignore
/// /// Gets the any type
/// bt_def!(type_any -> Type);
/// ```
#[macro_export]
macro_rules! bt_def {
    ($r_name:ident -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self) -> $ret {
                unsafe { $ret::from_raw_unchecked([<bt_ $r_name>](self.as_ptr())) }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ident),+) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) -> $ret {
                unsafe {
                    let out = [<bt_ $r_name>](self.as_ptr(), $($arg.as_ptr()),+);
                    $ret::from_raw_unchecked(out)
                }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ident),+)) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) {
                unsafe { [<bt_ $r_name>](self.as_ptr(), $($arg.as_ptr()),+) }
            }
        }
    };

    ($r_name:ident($arg:ident: &CStr) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: impl $crate::wrappers::IntoCStr) -> Result<$ret, $crate::Error> {
                let c_str = $arg.as_c_str()?;
                unsafe {
                    let out = [<bt_ $r_name>](self.as_ptr(), c_str.as_ptr());
                    Ok($ret::from_raw_unchecked(out))
                }
            }
        }
    };

    ($r_name:ident($arg1:ident: &CStr, $arg2:ident: $ty:ident) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg1: impl $crate::wrappers::IntoCStr, $arg2: $ty) -> Result<$ret, crate::Error> {
                let c_str = $arg1.as_c_str()?;
                unsafe {
                    let out = [<bt_ $r_name>](self.as_ptr(), c_str.as_ptr(), $arg2.as_ptr());
                    Ok($ret::from_raw_unchecked(out))
                }
            }
        }
    };
}

/// Wraps a bolt method with only wrapper args and primitive returns
#[macro_export]
macro_rules! bt_def_prim {
    ($r_name:ident -> bool) => {
        paste::paste! {
            pub fn $r_name(&mut self) -> bool {
                unsafe { [<bt_ $r_name>](self.as_ptr()) == $crate::sys::BT_TRUE as u8 }
            }
        }
    };

    ($r_name:ident -> $ret:ty) => {
        paste::paste! {
            pub fn $r_name(&mut self) -> $ret {
                unsafe { [<bt_ $r_name>](self.as_ptr()) }
            }
        }
    };

    ($r_name:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self) {
                unsafe { [<bt_ $r_name>](self.as_ptr()) }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ident),+) -> bool) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) -> bool {
                unsafe {
                    [<bt_ $r_name>](self.as_ptr(), $($arg.as_ptr()),+) == $crate::sys::BT_TRUE as u8
                }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ident),+) -> $ret:ty) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) -> $ret {
                unsafe { [<bt_ $r_name>](self.as_ptr(), $($arg.as_ptr()),+) }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ty),+)) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) {
                unsafe { [<bt_ $r_name>](self.as_ptr(), $($arg),+) }
            }
        }
    };
}

/// bt_def but with optional return types
#[macro_export]
macro_rules! bt_def_opt {
    ($r_name:ident -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self) -> Option<$ret> {
                unsafe { $ret::from_raw([<bt_ $r_name>](self.as_ptr())) }
            }
        }
    };

    ($r_name:ident($($arg:ident: $ty:ident),+) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $($arg: $ty),+) -> Option<$ret> {
                unsafe { $ret::from_raw([<bt_ $r_name>](self.as_ptr(), $($arg.as_ptr()),+)) }
            }
        }
    };

    ($r_name:ident($arg:ident: &CStr) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: impl $crate::wrappers::IntoCStr) -> Option<$ret> {
                let c_str = $arg.as_c_str();
                unsafe { $ret::from_raw([<bt_ $r_name>](self.as_ptr(), c_str.as_ptr())) }
            }
        }
    };

    ($r_name:ident($arg:ident: Value) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: Value) -> Option<$ret> {
                unsafe { $ret::from_raw([<bt_ $r_name>](self.as_ptr(), $arg.0)) }
            }
        }
    };
}

#[macro_export]
macro_rules! bt_def_userdata_field {
    ($field_type:ident) => {
        paste::paste! {
            pub fn [<userdata_type_field_ $field_type>](&mut self, type_: Type, name: impl $crate::wrappers::IntoCStr, offset: u32) -> Result<(), $crate::Error>  {
                let c_str = name.as_c_str()?;
                unsafe {
                    Ok($crate::sys::[<bt_userdata_type_field_ $field_type>](
                        self.as_ptr(),
                        type_.as_ptr(),
                        c_str.as_ptr(),
                        offset
                    ))
                }
            }
        }
    };
}

#[macro_export]
macro_rules! bt_def_bool {
    ($r_name:ident($arg:ident: &CStr, $bool_arg:ident: bool) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: impl $crate::wrappers::IntoCStr, $bool_arg: bool) -> Result<$ret, $crate::Error> {
                let c_str = $arg.as_c_str()?;
                unsafe {
                    Ok($ret::from_raw_unchecked($crate::sys::[<bt_ $r_name>](
                        self.as_ptr(),
                        c_str.as_ptr(),
                        $bool_arg as $crate::sys::bt_bool
                    )))
                }
            }
        }
    };

    ($r_name:ident(name: Value, $bool_arg:ident: bool) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, name: Value, $bool_arg: bool) -> Option<$ret> {
                unsafe {
                    let ptr = $crate::sys::[<bt_ $r_name>](
                        self.as_ptr(),
                        name.0,
                        $bool_arg as $crate::sys::bt_bool
                    );
                    $ret::from_raw(ptr)
                }
            }
        }
    };

    ($r_name:ident($arg:ident: &CStr) -> bool) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: impl $crate::wrappers::IntoCStr) -> bool {
                let c_str = $arg.as_c_str();
                unsafe {
                    $crate::sys::[<bt_ $r_name>](self.as_ptr(), c_str.as_ptr()) != 0
                }
            }
        }
    };
}

#[macro_export]
macro_rules! bt_def_slice {
    ($r_name:ident($arg:ident: &[Type]) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg: &[Type]) -> Option<$ret> {
                let mut collection: Vec<_> = $arg.iter().map(|c| c.as_ptr()).collect();
                unsafe {
                    let out = $crate::sys::[<bt_ $r_name>](
                        self.as_ptr(),
                        collection.as_mut_ptr(),
                        $arg.len()
                    );
                    $ret::from_raw(out)
                }
            }
        }
    };

    ($r_name:ident($arg1:ident: Type, $arg2:ident: &[Type]) -> $ret:ident) => {
        paste::paste! {
            pub fn $r_name(&mut self, $arg1: Type, $arg2: &[Type]) -> Option<$ret> {
                let mut collection: Vec<_> = $arg2
                    .iter()
                    .map(|c| c.as_ptr())
                    .take(u8::MAX as usize)
                    .collect();
                unsafe {
                    let out = $crate::sys::[<bt_ $r_name>](
                        self.as_ptr(),
                        $arg1.as_ptr(),
                        collection.as_mut_ptr(),
                        $arg2.len() as u8
                    );
                    $ret::from_raw(out)
                }
            }
        }
    };
}

#[macro_export]
macro_rules! define_wrapper {
    ($name:ident, $c_type:ty) => {
        #[derive(Debug, Clone)]
        #[repr(transparent)]
        pub struct $name {
            ptr: ::std::ptr::NonNull<$c_type>,
        }

        impl $name {
            #[inline]
            pub fn from_raw(ptr: *mut $c_type) -> Option<Self> {
                ::std::ptr::NonNull::new(ptr).map(|ptr| Self { ptr })
            }

            #[inline]
            pub unsafe fn from_raw_unchecked(ptr: *mut $c_type) -> Self {
                unsafe {
                    Self {
                        ptr: ::std::ptr::NonNull::new_unchecked(ptr),
                    }
                }
            }

            #[inline]
            pub fn as_ptr(&self) -> *mut $c_type {
                self.ptr.as_ptr()
            }
        }

        impl ::std::convert::AsRef<$c_type> for $name {
            #[inline]
            fn as_ref(&self) -> &$c_type {
                unsafe { self.ptr.as_ref() }
            }
        }

        impl ::std::convert::AsMut<$c_type> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut $c_type {
                unsafe { self.ptr.as_mut() }
            }
        }
    };
}

#[macro_export]
macro_rules! define_wrapper_with_drop {
    ($name:ident, $c_type:ty, $drop_fn:expr) => {
        #[derive(Debug)]
        #[repr(transparent)]
        pub struct $name {
            ptr: ::std::ptr::NonNull<$c_type>,
        }

        impl $name {
            #[inline]
            pub fn from_raw(ptr: *mut $c_type) -> Option<Self> {
                ::std::ptr::NonNull::new(ptr).map(|ptr| Self { ptr })
            }

            #[inline]
            pub unsafe fn from_raw_unchecked(ptr: *mut $c_type) -> Self {
                unsafe {
                    Self {
                        ptr: ::std::ptr::NonNull::new_unchecked(ptr),
                    }
                }
            }

            #[inline]
            pub fn as_ptr(&self) -> *mut $c_type {
                self.ptr.as_ptr()
            }

            #[inline]
            pub fn into_raw(self) -> *mut $c_type {
                let ptr = self.ptr.as_ptr();
                ::std::mem::forget(self);
                ptr
            }
        }

        impl ::std::convert::AsRef<$c_type> for $name {
            #[inline]
            fn as_ref(&self) -> &$c_type {
                unsafe { self.ptr.as_ref() }
            }
        }

        impl ::std::convert::AsMut<$c_type> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut $c_type {
                unsafe { self.ptr.as_mut() }
            }
        }

        impl ::std::ops::Drop for $name {
            fn drop(&mut self) {
                let drop_fn: fn(*mut $c_type) = $drop_fn;
                drop_fn(self.ptr.as_ptr());
            }
        }
    };
}

#[macro_export]
macro_rules! define_object_wrapper {
    ($name:ident, $c_type:ty) => {
        #[derive(Debug, Clone, Copy)]
        #[repr(transparent)]
        pub struct $name {
            ptr: ::std::ptr::NonNull<$c_type>,
        }

        impl $name {
            #[inline]
            pub fn from_raw(ptr: *mut $c_type) -> Option<Self> {
                ::std::ptr::NonNull::new(ptr).map(|ptr| Self { ptr })
            }

            #[inline]
            pub unsafe fn from_raw_unchecked(ptr: *mut $c_type) -> Self {
                unsafe {
                    Self {
                        ptr: ::std::ptr::NonNull::new_unchecked(ptr),
                    }
                }
            }

            #[inline]
            pub fn as_ptr(&self) -> *mut $c_type {
                self.ptr.as_ptr()
            }

            #[inline]
            pub fn as_object_ptr(&self) -> *mut $crate::sys::bt_Object {
                self.ptr.as_ptr() as *mut $crate::sys::bt_Object
            }

            #[inline]
            pub fn mask(&self) -> u64 {
                unsafe { (*self.as_object_ptr()).mask }
            }

            #[inline]
            pub fn object_type(&self) -> u32 {
                $crate::sys::object_mask::get_type(self.mask())
            }
        }

        impl ::std::convert::AsRef<$c_type> for $name {
            #[inline]
            fn as_ref(&self) -> &$c_type {
                unsafe { self.ptr.as_ref() }
            }
        }

        impl ::std::convert::AsMut<$c_type> for $name {
            #[inline]
            fn as_mut(&mut self) -> &mut $c_type {
                unsafe { self.ptr.as_mut() }
            }
        }
    };
}

#[macro_export]
macro_rules! define_wrappers {
    ($($name:ident => $c_type:ty),* $(,)?) => {
        $(
            $crate::define_wrapper!($name, $c_type);
        )*
    };
}

#[macro_export]
macro_rules! define_object_wrappers {
    ($($name:ident => $c_type:ty),* $(,)?) => {
        $(
            $crate::define_object_wrapper!($name, $c_type);
        )*
    };
}
