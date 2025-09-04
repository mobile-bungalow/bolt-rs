use crate::ArgError;
use crate::sys::*;

use super::FromBoltValue;
use super::MakeBoltValue;
use super::ScalarTypeSignature;
use super::TypeSignature;
use super::bt_object::Type;

// Implement TypeSignature for f64 so we can get its type
impl ScalarTypeSignature for f64 {
    fn make_type(ctx: &mut crate::BoltContext) -> Type {
        unsafe {
            let type_ptr = crate::sys::bt_type_number(ctx.as_ptr());
            Type::from_raw(type_ptr).expect("Failed to get number type")
        }
    }
}

impl FromBoltValue for f64 {
    fn from(val: bt_Value) -> Result<Self, ArgError> {
        unsafe {
            if bt_is_number(val) != 0 {
                Ok(bt_get_number(val))
            } else {
                Err(ArgError::TypeGuard {
                    expected: crate::bt_value::ValueType::Number,
                    actual: crate::bt_value::ValueType::None,
                })
            }
        }
    }

    unsafe fn from_unchecked(val: bt_Value) -> Self {
        unsafe { bt_get_number(val) }
    }
}

impl MakeBoltValue for f64 {
    fn make(&self) -> bt_Value {
        unsafe { bt_make_number(*self) }
    }
}
