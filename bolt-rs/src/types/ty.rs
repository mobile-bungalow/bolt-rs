use super::Type;
use bolt_sys::sys::*;

impl Type {
    bt_def!(type_dealias -> Type);
    bt_def_prim!(is_alias -> bool);
    bt_def_prim!(type_is_equal(other: Type) -> bool);
    bt_def_prim!(union_get_length -> i32);
    bt_def_prim!(type_is_optional -> bool);
    bt_def_prim!(union_has_variant(variant: Type) -> i32);

    pub fn union_get_variant(&mut self, idx: u32) -> Type {
        unsafe { Type::from_raw_unchecked(bt_union_get_variant(self.as_ptr(), idx)) }
    }
}
