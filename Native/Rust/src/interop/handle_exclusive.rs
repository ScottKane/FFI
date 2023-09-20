use crate::interop::is_null::IsNull;
use crate::interop::thread_bound::ThreadBound;
use crate::{unsafe_block, unsafe_fn, unsafe_impl};
use std::marker::PhantomData;
use std::panic::{RefUnwindSafe, UnwindSafe};

/**
A non-shared handle that cannot be accessed by multiple threads.

The interior value can be treated like `&mut T`.

The handle is bound to the thread that it was created on to ensure
there's no possibility for data races. Note that, if reverse PInvoke is supported
then it's possible to mutably alias the handle from the same thread if the reverse
call can re-enter the FFI using the same handle. This is technically undefined behaviour.

The handle _can_ be deallocated from a different thread than the one that created it.

Consumers must ensure a handle is not used again after it has been deallocated.
 */
#[repr(transparent)]
pub struct HandleExclusive<'a, T: ?Sized>(*mut ThreadBound<T>, PhantomData<&'a mut T>);

unsafe_impl!("The handle is semantically `&mut T`" => impl<'a, T: ?Sized> Send for HandleExclusive<'a, T> where &'a mut ThreadBound<T>: Send {});
unsafe_impl!("The handle uses `ThreadBound` for synchronization" => impl<'a, T: ?Sized> Sync for HandleExclusive<'a, T> where &'a mut ThreadBound<T>: Sync {});

impl<'a, T: ?Sized + RefUnwindSafe> UnwindSafe for HandleExclusive<'a, T> {}

impl<'a, T> HandleExclusive<'a, T>
where
    HandleExclusive<'a, T>: Send + Sync,
{
    pub fn alloc(value: T) -> Self
    where
        T: 'static,
    {
        let v = Box::new(ThreadBound::new(value));
        HandleExclusive(Box::into_raw(v), PhantomData)
    }

    pub fn as_mut(&mut self) -> &mut T {
        unsafe_block!("We own the interior value" => &mut *(*self.0).get_raw())
    }

    unsafe_fn!("There are no other live references and the handle won't be used again" =>
    pub fn dealloc<R>(handle: Self, f: impl FnOnce(T) -> R) -> R
    where
        T: Send,
    {
        let v = Box::from_raw(handle.0);
        f(v.into_inner())
    });
}

impl<'a, T: ?Sized> IsNull for HandleExclusive<'a, T> {
    fn is_null(&self) -> bool {
        self.0.is_null()
    }
}
