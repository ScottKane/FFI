use crate::interop::is_null::IsNull;
use crate::{unsafe_fn, unsafe_impl};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::slice;

/**
An initialized parameter passed by exclusive reference.
 */
#[repr(transparent)]
pub struct RefMut<'a, T: ?Sized>(*mut T, PhantomData<&'a mut T>);

impl<'a, T: ?Sized + RefUnwindSafe> UnwindSafe for RefMut<'a, T> {}

unsafe_impl!("The handle is semantically `&mut T`" => impl<'a, T: ?Sized> Send for RefMut<'a, T> where &'a mut T: Send {});
unsafe_impl!("The handle uses `ThreadBound` for synchronization" => impl<'a, T: ?Sized> Sync for RefMut<'a, T> where &'a mut T: Sync {});

impl<'a, T: ?Sized> RefMut<'a, T> {
    unsafe_fn!("The pointer must be nonnull and will remain valid" => pub fn as_mut(&mut self) -> &mut T {
        &mut *self.0
    });
}

impl<'a> RefMut<'a, u8> {
    unsafe_fn!("The pointer must be nonnull, the length is correct, and will remain valid" => pub fn as_bytes_mut(&mut self, len: usize) -> &mut [u8] {
        slice::from_raw_parts_mut(self.0, len)
    });
}

impl<'a, T: ?Sized + Sync> IsNull for RefMut<'a, T> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
