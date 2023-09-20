use crate::interop::option::OptionMutExt;
use failure::Fail;
use std::{
    any::Any,
    cell::RefCell,
    fmt::Write,
    panic::{catch_unwind, UnwindSafe},
    sync::atomic::{AtomicU32, Ordering},
};

static LAST_ERR_ID: AtomicU32 = AtomicU32::new(0);

fn next_err_id() -> u32 {
    LAST_ERR_ID.fetch_add(1, Ordering::SeqCst)
}

thread_local! {
    static LAST_RESULT: RefCell<Option<LastResult>> = RefCell::new(None);
}

/**
The result of making a call across an FFI boundary.

The result may indicate success or an error.
If an error is returned, the thread-local `last_result` can be inspected for more details.
 */
#[repr(C)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DbResult {
    kind: Kind,
    id: u32,
}

#[repr(u32)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Kind {
    Ok,

    Done,
    BufferTooSmall,

    ArgumentNull,
    InternalError,
}

impl DbResult {
    pub fn ok() -> Self {
        DbResult {
            kind: Kind::Ok,
            id: 0,
        }
    }

    pub fn is_ok(&self) -> bool {
        self.kind == Kind::Ok
    }

    pub fn done() -> Self {
        DbResult {
            kind: Kind::Done,
            id: 0,
        }
    }

    pub fn is_done(&self) -> bool {
        self.kind == Kind::Done
    }

    pub fn buffer_too_small() -> Self {
        DbResult {
            kind: Kind::BufferTooSmall,
            id: next_err_id(),
        }
    }

    pub fn is_buffer_too_small(&self) -> bool {
        self.kind == Kind::BufferTooSmall
    }

    pub fn argument_null() -> Self {
        DbResult {
            kind: Kind::ArgumentNull,
            id: next_err_id(),
        }
    }

    pub fn is_argument_null(&self) -> bool {
        self.kind == Kind::ArgumentNull
    }

    pub fn internal_error() -> Self {
        DbResult {
            kind: Kind::InternalError,
            id: next_err_id(),
        }
    }

    pub fn is_internal_error(&self) -> bool {
        self.kind == Kind::InternalError
    }

    pub fn as_err(&self) -> Option<&'static str> {
        match self.kind {
            Kind::Ok | Kind::Done => None,
            Kind::ArgumentNull => Some("a required argument was null"),
            Kind::BufferTooSmall => Some("a supplied buffer was too small"),
            Kind::InternalError => Some("an internal error occurred"),
        }
    }

    pub fn context(self, e: impl Fail) -> Self {
        assert!(
            self.as_err().is_some(),
            "context can only be attached to errors"
        );

        let err = Some(format_error(&e));

        LAST_RESULT.with(|last_result| {
            *last_result.borrow_mut() = Some(LastResult { value: self, err });
        });

        self
    }

    pub fn catch(f: impl FnOnce() -> Self + UnwindSafe) -> Self {
        LAST_RESULT.with(|last_result| {
            {
                *last_result.borrow_mut() = None;
            }

            match catch_unwind(f) {
                Ok(db_result) => {
                    let extract_err = || db_result.as_err().map(Into::into);

                    // Always set the last result so it matches what's returned.
                    // This `Ok` branch doesn't necessarily mean the result is ok,
                    // only that there wasn't a panic.
                    last_result
                        .borrow_mut()
                        .map_mut(|last_result| {
                            last_result.value = db_result;
                            last_result.err.or_else_mut(extract_err);
                        })
                        .get_or_insert_with(|| LastResult {
                            value: db_result,
                            err: extract_err(),
                        })
                        .value
                }
                Err(e) => {
                    let extract_panic =
                        || extract_panic(&e).map(|s| format!("internal panic with '{}'", s));

                    // Set the last error to the panic message if it's not already set
                    last_result
                        .borrow_mut()
                        .map_mut(|last_result| {
                            last_result.err.or_else_mut(extract_panic);
                        })
                        .get_or_insert_with(|| LastResult {
                            value: DbResult::internal_error(),
                            err: extract_panic(),
                        })
                        .value
                }
            }
        })
    }

    pub fn with_last_result<R>(f: impl FnOnce(Option<(DbResult, Option<&str>)>) -> R) -> R {
        LAST_RESULT.with(|last_result| {
            let last_result = last_result.borrow();

            let last_result = last_result.as_ref().map(|last_result| {
                let msg = last_result
                    .value
                    .as_err()
                    .and_then(|_| last_result.err.as_ref().map(|msg| msg.as_ref()));

                (last_result.value, msg)
            });

            f(last_result)
        })
    }
}

/**
Map error types that are convertible into `Error` into `DbResult`s.

This is so we can use `?` on `Result<T, E: Fail>` in FFI functions.
The error state will be serialized and stored in a thread-local that can be queried later.
 */
impl<E> From<E> for DbResult
where
    E: Fail,
{
    fn from(e: E) -> Self {
        DbResult::internal_error().context(e)
    }
}

#[derive(Debug)]
struct LastResult {
    value: DbResult,
    err: Option<String>,
}

fn format_error(err: &dyn Fail) -> String {
    let mut error_string = String::new();

    let mut causes = Some(err).into_iter().chain(err.iter_causes());

    if let Some(cause) = causes.next() {
        let _ = writeln!(error_string, "{}.", cause);
    }

    let mut next = causes.next();
    while next.is_some() {
        let cause = next.unwrap();
        let _ = writeln!(error_string, "   caused by: {}", cause);
        next = causes.next();
    }

    if let Some(backtrace) = err.backtrace() {
        let _ = writeln!(error_string, "backtrace: {}", backtrace);
    }

    error_string
}

fn extract_panic(err: &Box<dyn Any + Send + 'static>) -> Option<String> {
    if let Some(err) = err.downcast_ref::<String>() {
        Some(err.clone())
    } else if let Some(err) = err.downcast_ref::<&'static str>() {
        Some((*err).to_owned())
    } else {
        None
    }
}
