#![allow(dead_code)]

mod db;
mod interop;

use crate::db::data::Data;
use crate::db::result::DbResult;
use crate::db::store;
use crate::interop::handle_exclusive::HandleExclusive;
use crate::interop::handle_shared::HandleShared;
use crate::interop::out::Out;
use crate::interop::r#ref::Ref;
use crate::interop::read::into_fixed_buffer;
use crate::interop::thread_bound;
use libc::size_t;

#[macro_use]
extern crate rental;

#[repr(transparent)]
pub struct DbKey([u8; 16]);

#[repr(C)]
pub struct DbStore {
    inner: store::Store,
}

pub type DbStoreHandle<'a> = HandleShared<'a, DbStore>;

#[repr(C)]
pub struct DbReader {
    inner: thread_bound::DeferredCleanup<store::reader::Reader>,
}

pub type DbReaderHandle<'a> = HandleExclusive<'a, DbReader>;

#[repr(C)]
pub struct DbWriter {
    inner: store::writer::Writer,
}

pub type DbWriterHandle<'a> = HandleExclusive<'a, DbWriter>;

#[repr(C)]
pub struct DbDeleter {
    inner: store::deleter::Deleter,
}

pub type DbDeleterHandle<'a> = HandleExclusive<'a, DbDeleter>;

ffi_no_catch! {
    fn db_last_result(
        message_buf: Out<u8>,
        message_buf_len: size_t,
        actual_message_len: Out<size_t>,
        result: Out<DbResult>
    ) -> DbResult {
        DbResult::with_last_result(|last_result| {
            let (value, error) = last_result.unwrap_or((DbResult::ok(), None));

            unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => result.init(value));

            if let Some(error) = error {
                let error = error.as_bytes();

                unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => actual_message_len.init(error.len()));

                if message_buf_len < error.len() {
                    return DbResult::buffer_too_small();
                }

                unsafe_block!("The buffer is valid for writes and the length is within the buffer" => message_buf.init_bytes(error));
            } else {
                unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => actual_message_len.init(0));
            }

            DbResult::ok()
        })
    }
}

ffi! {
    fn db_store_open(path: Ref<u8>, path_len: size_t, store: Out<DbStoreHandle>) -> DbResult {
        let path_slice = unsafe_block!("The path lives as long as `db_store_open` and the length is within the path" => path.as_bytes(path_len));

        let path = std::str::from_utf8(path_slice);
        let path = match path {
            Ok(value) => value,
            Err(e) => return DbResult::from(e),
        };

        let handle = match store::Store::open(path) {
            Ok(value) => DbStoreHandle::alloc(DbStore {
                inner: value,
            }),
            Err(e) => return DbResult::from(e),
        };

        unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => store.init(handle));

        DbResult::ok()
    }

    fn db_store_close(store: DbStoreHandle) -> DbResult {
        unsafe_block!("The upstream caller guarantees the handle will not be accessed after being freed" => DbStoreHandle::dealloc(store, |mut store| {
            match store.inner.close() {
                Ok(_) => DbResult::ok(),
                Err(e) => DbResult::from(e),
            }
        }))
    }

    fn db_read_begin(
        store: DbStoreHandle,
        reader: Out<DbReaderHandle>
    ) -> DbResult {
        let store = store.as_ref();

        let handle = match store.inner.read_begin() {
            Ok(value) =>DbReaderHandle::alloc(DbReader {
                inner: thread_bound::DeferredCleanup::new(value),
            }),
            Err(e) => return DbResult::from(e),
        };

        unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => reader.init(handle));

        DbResult::ok()
    }

    fn db_read_next(
        reader: DbReaderHandle,
        key: Out<DbKey>,
        value_buf: Out<u8>,
        value_buf_len: size_t,
        actual_value_len: Out<size_t>
    ) -> DbResult {
        let reader = reader.as_mut();

        let buf = unsafe_block!("The buffer lives as long as `db_read_next`, the length is within the buffer and the buffer won't be read before initialization" => value_buf.as_uninit_bytes_mut(value_buf_len));

        'read_event: loop {
            let read_result = reader.inner.with_current(|mut current| {
                into_fixed_buffer(&mut current, buf, &mut key, &mut actual_value_len)
            });

            match read_result {
                Some(result) if result.is_ok() => {
                    match reader.inner.move_next() {
                        Ok(_) => return DbResult::ok(),
                        Err(e) => return DbResult::from(e),
                    }
                },
                Some(result) => return result,
                None => {
                    match reader.inner.move_next() {
                        Ok(result) => {
                            if result {
                                continue 'read_event;
                            } else {
                                return DbResult::done();
                            }
                        }
                        Err(e) => {
                            return DbResult::from(e);
                        }
                    }
                }
            }
        }
    }

    fn db_read_end(reader: DbReaderHandle) -> DbResult {
        unsafe_block!("The upstream caller guarantees the handle will not be accessed after being freed" => DbReaderHandle::dealloc(reader, |mut reader| {
            match reader.inner.complete() {
                Ok(_) => DbResult::ok(),
                Err(e) => DbResult::from(e),
            }
        }))
    }

    fn db_write_begin(
        store: DbStoreHandle,
        writer: Out<DbWriterHandle>
    ) -> DbResult {
        let store = store.as_ref();

        let handle = match store.inner.write_begin() {
            Ok(value) => DbWriterHandle::alloc(DbWriter {
                inner: value,
            }),
            Err(e) => return DbResult::from(e),
        };

        unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => writer.init(handle));

        DbResult::ok()
    }

    fn db_write_set(
        writer: DbWriterHandle,
        key: Ref<DbKey>,
        value: Ref<u8>,
        value_len: size_t
    ) -> DbResult {
        let writer = writer.as_mut();

        let key = unsafe_block!("The key pointer lives as long as `db_write_set` and points to valid data" => key.as_ref());
        let value_slice = unsafe_block!("The buffer lives as long as `db_write_set` and the length is within the buffer" => value.as_bytes(value_len));

        let data = Data {
            key: db::data::Key::from_bytes(key.0),
            payload: value_slice,
        };

        match writer.inner.set(data) {
            Ok(_) => DbResult::ok(),
            Err(e) => DbResult::from(e),
        }
    }

    fn db_write_end(writer: DbWriterHandle) -> DbResult {
        unsafe_block!("The upstream caller guarantees the handle will not be accessed after being freed" => DbWriterHandle::dealloc(writer, |mut writer| {
            match writer.inner.complete() {
                Ok(_) => DbResult::ok(),
                Err(e) => DbResult::from(e),
            }
        }))
    }

    fn db_delete_begin(
        store: DbStoreHandle,
        deleter: Out<DbDeleterHandle>
    ) -> DbResult {
        let store = store.as_ref();

        let handle = match store.inner.delete_begin() {
            Ok(value) => DbDeleterHandle::alloc(DbDeleter {
                inner: value,
            }),
            Err(e) => return DbResult::from(e),
        };

        unsafe_block!("The out pointer is valid and not mutably aliased elsewhere" => deleter.init(handle));

        DbResult::ok()
    }

    fn db_delete_remove(
        deleter: DbDeleterHandle,
        key: Ref<DbKey>
    ) -> DbResult {
        let deleter = deleter.as_mut();

        let key = unsafe_block!("The key pointer lives as long as `db_delete_remove` and points to valid data" => key.as_ref());

        match deleter.inner.remove(db::data::Key::from_bytes(key.0)) {
            Ok(_) => DbResult::ok(),
            Err(e) => DbResult::from(e),
        }
    }

    fn db_delete_end(deleter: DbDeleterHandle) -> DbResult {
        unsafe_block!("The upstream caller guarantees the handle will not be accessed after being freed" => DbDeleterHandle::dealloc(deleter, |mut deleter| {
            match deleter.inner.complete() {
                Ok(_) => DbResult::ok(),
                Err(e) => DbResult::from(e),
            }
        }))
    }
}
