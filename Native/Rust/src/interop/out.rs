use crate::interop::is_null::IsNull;
use crate::{unsafe_fn, unsafe_impl};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::ptr;
use std::slice;

/**
An uninitialized, assignable out parameter.
 */
#[repr(transparent)]
pub struct Out<'a, T: ?Sized>(*mut T, PhantomData<&'a mut T>);

impl<'a, T: ?Sized + RefUnwindSafe> UnwindSafe for Out<'a, T> {}

unsafe_impl!("The handle is semantically `&mut T`" => impl<'a, T: ?Sized> Send for Out<'a, T> where &'a mut T: Send {});
unsafe_impl!("The handle uses `ThreadBound` for synchronization" => impl<'a, T: ?Sized> Sync for Out<'a, T> where &'a mut T: Sync {});

impl<'a, T> Out<'a, T> {
    unsafe_fn!("The pointer must be nonnull and valid for writes" => pub fn init(&mut self, value: T) {
        ptr::write(self.0, value);
    });
}

impl<'a> Out<'a, u8> {
    unsafe_fn!("The pointer must be nonnull, not overlap the slice, must be valid for the length of the slice, and valid for writes" => pub fn init_bytes(&mut self, value: &[u8]) {
        ptr::copy_nonoverlapping(value.as_ptr(), self.0, value.len());
    });

    unsafe_fn!("The slice must never be read from and must be valid for the length of the slice" => pub fn as_uninit_bytes_mut(&mut self, len: usize) -> &mut [u8] {
        slice::from_raw_parts_mut(self.0, len)
    });
}

impl<'a, T: ?Sized> IsNull for Out<'a, T> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
