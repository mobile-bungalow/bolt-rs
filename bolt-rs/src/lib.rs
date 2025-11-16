#[macro_use]
mod wrappers;
pub mod types;

mod error;

pub use error::{ArgError, Error, ModuleError};
pub use types::value::{
    CallSignature, FromBoltValue, MakeBoltValue, MakeBoltValueWithContext, ScalarTypeSignature,
    TypeSignature, Value, ValueType,
};
pub use types::{Context, Thread};
pub use wrappers::IntoCStr;

// Re-export bolt-sys for raw C interface
pub use bolt_derive::*;
pub use bolt_sys::sys;
