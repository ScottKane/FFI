/*
The handles here are wrappers for a shared `&T` and an exclusive `&mut T`.

They protect from data races, but don't protect from use-after-free bugs.
The caller is expected to maintain that invariant, which in .NET can be
achieved using `SafeHandle`s.
*/

use crate::interop::is_null::IsNull;
use crate::{unsafe_block, unsafe_fn, unsafe_impl};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

/**
A shared handle that can be accessed concurrently by multiple threads.

The interior value can be treated like `&T`.

Consumers must ensure a handle is not used again after it has been deallocated.
 */
#[repr(transparent)]
pub struct HandleShared<'a, T: ?Sized>(*const T, PhantomData<&'a T>);

unsafe_impl!("The handle is semantically `&T`" => impl<'a, T: ?Sized> Send for HandleShared<'a, T> where &'a T: Send {});
unsafe_impl!("The handle is semantically `&T`" => impl<'a, T: ?Sized> Sync for HandleShared<'a, T> where &'a T: Sync {});

impl<'a, T: ?Sized + RefUnwindSafe> UnwindSafe for HandleShared<'a, T> {}

impl<'a, T> HandleShared<'a, T>
where
    HandleShared<'a, T>: Send + Sync,
{
    pub fn alloc(value: T) -> Self
    where
        T: 'static,
    {
        let v = Box::new(value);
        HandleShared(Box::into_raw(v), PhantomData)
    }

    pub fn as_ref(&self) -> &T {
        unsafe_block!("We own the interior value" => &*self.0)
    }

    unsafe_fn!("There are no other live references and the handle won't be used again" =>
    pub fn dealloc<R>(handle: Self, f: impl FnOnce(T) -> R) -> R {
        let v = Box::from_raw(handle.0 as *mut T);
        f(*v)
    });
}

impl<'a, T: ?Sized + Sync> IsNull for HandleShared<'a, T> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
