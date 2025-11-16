//! Thread type and all its methods

use bolt_sys::sys;

/// Safe wrapper around bt_Thread
#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Thread {
    ptr: ::std::ptr::NonNull<sys::bt_Thread>,
}

impl Thread {
    #[inline]
    pub fn from_raw(ptr: *mut sys::bt_Thread) -> Option<Self> {
        ::std::ptr::NonNull::new(ptr).map(|ptr| Self { ptr })
    }

    #[inline]
    pub unsafe fn from_raw_unchecked(ptr: *mut sys::bt_Thread) -> Self {
        unsafe {
            Self {
                ptr: ::std::ptr::NonNull::new_unchecked(ptr),
            }
        }
    }

    #[inline]
    pub fn as_ptr(&self) -> *mut sys::bt_Thread {
        self.ptr.as_ptr()
    }
}

impl ::std::convert::AsRef<sys::bt_Thread> for Thread {
    #[inline]
    fn as_ref(&self) -> &sys::bt_Thread {
        unsafe { self.ptr.as_ref() }
    }
}

impl ::std::convert::AsMut<sys::bt_Thread> for Thread {
    #[inline]
    fn as_mut(&mut self) -> &mut sys::bt_Thread {
        unsafe { self.ptr.as_mut() }
    }
}

impl Thread {
    pub fn return_val<T: crate::types::value::MakeBoltValue>(&mut self, val: &T) {
        unsafe { sys::bt_return(self.as_ptr(), val.make()) }
    }

    pub fn get_arg<T: crate::types::value::FromBoltValue>(
        &mut self,
        idx: u8,
    ) -> Result<T, crate::ArgError> {
        let val = unsafe {
            let len = sys::bt_argc(self.as_ptr());
            if len <= idx {
                return Err(crate::ArgError::IndexOutOfBounds { idx, len });
            }
            sys::bt_arg(self.as_ptr(), idx)
        };
        T::from(val)
    }

    pub unsafe fn get_arg_unchecked<T: crate::types::value::FromBoltValue>(
        &mut self,
        idx: u8,
    ) -> T {
        unsafe {
            let val = sys::bt_arg(self.as_ptr(), idx);
            T::from_unchecked(val)
        }
    }

    pub fn push<T: crate::types::value::MakeBoltValue>(&mut self, value: &T) {
        unsafe {
            sys::bt_push(self.as_ptr(), value.make());
        }
    }

    pub fn call(&mut self, argc: u8) {
        unsafe {
            sys::bt_call(self.as_ptr(), argc);
        }
    }

    pub fn get_returned<T: crate::types::value::FromBoltValue>(
        &self,
    ) -> Result<T, crate::ArgError> {
        unsafe {
            let val = sys::bt_get_returned(self.as_ptr());
            T::from(val)
        }
    }

    pub unsafe fn get_returned_unchecked<T: crate::types::value::FromBoltValue>(&self) -> T {
        unsafe {
            let val = sys::bt_get_returned(self.as_ptr());
            T::from_unchecked(val)
        }
    }

    pub fn argc(&self) -> u8 {
        unsafe { sys::bt_argc(self.as_ptr()) }
    }
}
