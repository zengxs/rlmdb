use std::{
    ffi, fmt,
    marker::PhantomData,
    mem::{self, ManuallyDrop},
    ptr::NonNull,
};

use bitflags::bitflags;

use crate::{DBEnv, db::Database, sys};

pub struct Transaction<'env> {
    ptr: ManuallyDrop<NonNull<sys::MDB_txn>>,

    _marker: PhantomData<&'env DBEnv>,

    pub txn_type: TransactionType,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum TransactionType {
    ReadOnly,
    ReadWrite,
}

bitflags! {
    /// Flags for the transaction.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub(crate) struct TransactionFlags: ffi::c_uint {
        const MDB_RDONLY = sys::MDB_RDONLY;
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct PutFlags: ffi::c_uint {
        /// Enter the new key/data pair only if it does not
        /// already appear in the database. This flag may only be specified
        /// if the database was opened with `MDB_DUPSORT`. The function will
        /// return `MDB_KEYEXIST` if the key/data pair already appears in the
        /// database.
        const MDB_NODUPDATA = sys::MDB_NODUPDATA;

        /// enter the new key/data pair only if the key
        /// does not already appear in the database. The function will return
        /// `MDB_KEYEXIST` if the key already appears in the database, even if
        /// the database supports duplicates (`MDB_DUPSORT`). The **data**
        /// parameter will be set to point to the existing item.
        const MDB_NOOVERWRITE = sys::MDB_NOOVERWRITE;

        /// Reserve space for data of the given size, but
        /// don't copy the given data. Instead, return a pointer to the
        /// reserved space, which the caller can fill in later - before
        /// the next update operation or the transaction ends. This saves
        /// an extra memcpy if the data is being generated later.
        /// LMDB does nothing else with this memory, the caller is expected
        /// to modify all of the space requested. This flag must not be
        /// specified if the database was opened with `MDB_DUPSORT`.
        const MDB_RESERVE = sys::MDB_RESERVE;

        /// Append the given key/data pair to the end of the
        /// database. This option allows fast bulk loading when keys are
        /// already known to be in the correct order. Loading unsorted keys
        /// with this flag will cause a `MDB_KEYEXIST` error.
        const MDB_APPEND = sys::MDB_APPEND;

        /// As above, but for sorted dup data
        const MDB_APPENDDUP = sys::MDB_APPENDDUP;
    }
}

impl Default for PutFlags {
    fn default() -> Self {
        PutFlags::empty()
    }
}

#[allow(unused)]
impl<'env> Transaction<'env> {
    pub(crate) fn new(
        env: &'env DBEnv,
        parent: Option<&Transaction<'env>>,
        txn_type: TransactionType,
    ) -> Result<Self, crate::LMDBError> {
        let mut txn_ptr: *mut sys::MDB_txn = std::ptr::null_mut();

        let flags = match txn_type {
            TransactionType::ReadOnly => TransactionFlags::MDB_RDONLY.bits(),
            TransactionType::ReadWrite => TransactionFlags::empty().bits(), // No flags for read-write transactions
        };

        let parent_ptr = parent.map_or(std::ptr::null_mut(), |p| unsafe { p.as_raw_ptr() });

        let ret =
            unsafe { sys::mdb_txn_begin(env.as_ptr().as_ptr(), parent_ptr, flags, &mut txn_ptr) };
        crate::LMDBError::from_mdb_error(ret)?;

        // Ensure the pointer is not null and convert it to NonNull
        let ptr = NonNull::new(txn_ptr).ok_or_else(|| {
            crate::LMDBError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "mdb_txn_begin succeeded but returned a null transaction pointer",
            ))
        })?;

        Ok(Transaction {
            ptr: ManuallyDrop::new(ptr),
            _marker: PhantomData,
            txn_type,
        })
    }

    pub fn commit(mut self) -> Result<(), crate::LMDBError> {
        let ptr = unsafe { ManuallyDrop::take(&mut self.ptr) };
        let ret = unsafe { sys::mdb_txn_commit(ptr.as_ptr()) };

        // Prevent double drop/commit/abort
        mem::forget(self);

        crate::LMDBError::from_mdb_error(ret)
    }

    pub fn abort(mut self) {
        let ptr = unsafe { ManuallyDrop::take(&mut self.ptr) };
        unsafe { sys::mdb_txn_abort(ptr.as_ptr()) };

        // Prevent double drop/commit/abort
        mem::forget(self);
    }

    pub fn get<K, V>(&self, db: &'env Database<K, V>, key: K) -> Result<Option<V>, crate::LMDBError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]> + for<'a> From<&'a [u8]>,
    {
        let mut key = sys::MDB_val {
            mv_size: key.as_ref().len(),
            mv_data: key.as_ref().as_ptr() as *mut _,
        };
        let mut data = sys::MDB_val {
            mv_size: 0,
            mv_data: std::ptr::null_mut(),
        };

        let ret = unsafe { sys::mdb_get(self.as_raw_ptr(), db.id(), &mut key, &mut data) };
        crate::LMDBError::from_mdb_error(ret)?;

        let value_slice =
            unsafe { std::slice::from_raw_parts(data.mv_data as *const u8, data.mv_size) };
        Ok(Some(V::from(value_slice)))
    }

    pub fn put<K, V>(
        &self,
        db: &'env Database<K, V>,
        key: K,
        data: V,
        flags: Option<PutFlags>,
    ) -> Result<(), crate::LMDBError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let flags = flags.unwrap_or(PutFlags::default());
        let mut key = sys::MDB_val {
            mv_size: key.as_ref().len(),
            mv_data: key.as_ref().as_ptr() as *mut _,
        };
        let mut value = sys::MDB_val {
            mv_size: data.as_ref().len(),
            mv_data: data.as_ref().as_ptr() as *mut _,
        };

        let ret = unsafe {
            sys::mdb_put(
                self.as_raw_ptr(),
                db.id(),
                &mut key,
                &mut value,
                flags.bits(),
            )
        };
        crate::LMDBError::from_mdb_error(ret)
    }

    pub fn delete<K, V>(
        &self,
        db: &'env Database<K, V>,
        key: K,
        data: Option<V>,
    ) -> Result<(), crate::LMDBError>
    where
        K: AsRef<[u8]> + for<'a> From<&'a [u8]>,
        V: AsRef<[u8]> + for<'a> From<&'a [u8]>,
    {
        let mut key = sys::MDB_val {
            mv_size: key.as_ref().len(),
            mv_data: key.as_ref().as_ptr() as *mut _,
        };
        let mut data = match data {
            Some(d) => Some(sys::MDB_val {
                mv_size: d.as_ref().len(),
                mv_data: d.as_ref().as_ptr() as *mut _,
            }),
            None => None,
        };
        let data_ptr = data.as_mut().map_or(std::ptr::null_mut(), |d| d as *mut _);

        let ret = unsafe { sys::mdb_del(self.as_raw_ptr(), db.id(), &mut key, data_ptr) };
        crate::LMDBError::from_mdb_error(ret)
    }

    pub fn cursor<K, V>(
        &self,
        db: &'env Database<K, V>,
    ) -> Result<sys::MDB_cursor, crate::LMDBError> {
        todo!()
    }

    pub unsafe fn as_raw_ptr(&self) -> *mut sys::MDB_txn {
        self.ptr.as_ptr()
    }
}

impl<'env> Drop for Transaction<'env> {
    fn drop(&mut self) {
        unsafe { sys::mdb_txn_abort(self.as_raw_ptr()) }
    }
}

impl<'env> fmt::Debug for Transaction<'env> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Transaction")
            .field("ptr", &self.ptr.as_ptr())
            .field("type", &self.txn_type)
            .finish()
    }
}
