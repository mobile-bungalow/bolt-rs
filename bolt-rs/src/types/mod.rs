//! Type wrappers around C API types
//!
//! This module provides safe NonNull-based wrappers around raw C pointers.
use bolt_sys::sys;

pub mod context;
pub mod object;
pub mod thread;
pub mod ty;
pub mod value;

pub use context::Context;
pub use thread::Thread;
pub use value::Value;

define_wrappers! {
    Handlers => sys::bt_Handlers,
    GC => sys::bt_GC,
    Parser => sys::bt_Parser,
    Compiler => sys::bt_Compiler,
}

define_object_wrappers! {
    Object => sys::bt_Object,
    Type => sys::bt_Type,
    BoltString => sys::bt_String,
    Module => sys::bt_Module,
    ModuleImport => sys::bt_ModuleImport,
    BoltFn => sys::bt_Fn,
    NativeFn => sys::bt_NativeFn,
    Closure => sys::bt_Closure,
    Array => sys::bt_Array,
    Table => sys::bt_Table,
    Userdata => sys::bt_Userdata,
    Annotation => sys::bt_Annotation,
}
