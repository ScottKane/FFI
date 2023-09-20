use crate::interop::is_null::IsNull;
use crate::{unsafe_fn, unsafe_impl};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};
use std::slice;

/**
An initialized parameter passed by shared reference.
 */
#[repr(transparent)]
pub struct Ref<'a, T: ?Sized>(*const T, PhantomData<&'a T>);

impl<'a, T: ?Sized + RefUnwindSafe> UnwindSafe for Ref<'a, T> {}

unsafe_impl!("The handle is semantically `&mut T`" => impl<'a, T: ?Sized> Send for Ref<'a, T> where &'a T: Send {});
unsafe_impl!("The handle uses `ThreadBound` for synchronization" => impl<'a, T: ?Sized> Sync for Ref<'a, T> where &'a T: Sync {});

impl<'a, T: ?Sized> Ref<'a, T> {
    unsafe_fn!("The pointer must be nonnull and will remain valid" => pub fn as_ref(&self) -> &T {
        &*self.0
    });
}

impl<'a> Ref<'a, u8> {
    unsafe_fn!("The pointer must be nonnull, the length is correct, and will remain valid" => pub fn as_bytes(&self, len: usize) -> &[u8] {
        slice::from_raw_parts(self.0, len)
    });
}

impl<'a, T: ?Sized> IsNull for Ref<'a, T> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
