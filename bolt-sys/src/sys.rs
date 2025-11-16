#![allow(non_upper_case_globals)]
#![allow(non_camel_case_types)]
#![allow(non_snake_case)]
#![allow(unsafe_op_in_unsafe_fn)]

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

/// bt_Object mask field helpers
pub mod object_mask {
    pub const MARK_BIT: u64 = 0x1;
    pub const PTR_BITS: u64 = 0x00FF_FFFF_FFFF_FFFC;
    pub const TYPE_SHIFT: u32 = 56;
    pub const TYPE_MASK: u64 = 0xFF;

    #[inline]
    pub fn get_type(mask: u64) -> u32 {
        ((mask >> TYPE_SHIFT) & TYPE_MASK) as u32
    }

    #[inline]
    pub fn get_next_ptr(mask: u64) -> u64 {
        mask & PTR_BITS
    }

    #[inline]
    pub fn is_marked(mask: u64) -> bool {
        (mask & MARK_BIT) != 0
    }

    #[inline]
    pub fn set_mark(mask: &mut u64) {
        *mask |= MARK_BIT;
    }

    #[inline]
    pub fn clear_mark(mask: &mut u64) {
        *mask &= !MARK_BIT;
    }

    #[inline]
    pub fn set_type(mask: &mut u64, object_type: u32) {
        *mask = (*mask & !(TYPE_MASK << TYPE_SHIFT)) | ((object_type as u64) << TYPE_SHIFT);
    }

    #[inline]
    pub fn set_next_ptr(mask: &mut u64, ptr: u64) {
        *mask = (*mask & !PTR_BITS) | (ptr & PTR_BITS);
    }
}
