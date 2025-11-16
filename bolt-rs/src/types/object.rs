use bolt_sys::sys;

use crate::ValueType;

use super::Object;

impl Object {
    pub fn value_type(&self) -> ValueType {
        match self.object_type() {
            sys::bt_ObjectType_BT_OBJECT_TYPE_TYPE => ValueType::Type,
            sys::bt_ObjectType_BT_OBJECT_TYPE_STRING => ValueType::String,
            sys::bt_ObjectType_BT_OBJECT_TYPE_MODULE => ValueType::Module,
            sys::bt_ObjectType_BT_OBJECT_TYPE_IMPORT => ValueType::Import,
            sys::bt_ObjectType_BT_OBJECT_TYPE_USERDATA => ValueType::UserData,
            sys::bt_ObjectType_BT_OBJECT_TYPE_ANNOTATION => ValueType::Annotation,
            sys::bt_ObjectType_BT_OBJECT_TYPE_FN => ValueType::Function,
            sys::bt_ObjectType_BT_OBJECT_TYPE_NATIVE_FN => ValueType::NativeFunction,
            sys::bt_ObjectType_BT_OBJECT_TYPE_CLOSURE => ValueType::Closure,
            sys::bt_ObjectType_BT_OBJECT_TYPE_ARRAY => ValueType::Array,
            sys::bt_ObjectType_BT_OBJECT_TYPE_TABLE => ValueType::Table,
            // Internal error but we should make it typesafe
            _ => ValueType::None,
        }
    }
}
