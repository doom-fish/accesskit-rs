use core::ffi::c_void;
use core::fmt;
use core::ptr::NonNull;

use crate::ffi;

pub struct Object(NonNull<c_void>);

impl Object {
    pub unsafe fn from_retained(object: *mut c_void) -> Option<Self> {
        NonNull::new(object).map(Self)
    }

    pub unsafe fn retain(object: *mut c_void) -> Option<Self> {
        let object = NonNull::new(object)?;
        unsafe { ffi::accesskit_rs_retain(object.as_ptr()) };
        Some(Self(object))
    }

    pub const fn as_ptr(&self) -> *mut c_void {
        self.0.as_ptr()
    }
}

impl Clone for Object {
    fn clone(&self) -> Self {
        unsafe { ffi::accesskit_rs_retain(self.as_ptr()) };
        Self(self.0)
    }
}

impl Drop for Object {
    fn drop(&mut self) {
        unsafe { ffi::accesskit_rs_release(self.as_ptr()) };
    }
}

impl fmt::Debug for Object {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Object").field(&self.0).finish()
    }
}
