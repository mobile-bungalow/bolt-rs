use crate::bt_value::ValueType;

#[derive(Debug)]
pub enum ArgError {
    TypeGuard {
        expected: ValueType,
        actual: ValueType,
    },
    TypeGuardEnum {
        actual: ValueType,
    },
    IndexOutOfBounds {
        idx: u8,
        len: u8,
    },
}

#[derive(Debug)]
pub enum ModuleError {
    InvalidName(String),
    AlreadyRegistered(String),
    NotFound(String),
}
