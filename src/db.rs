use std::{ffi, marker::PhantomData};

use bitflags::bitflags;

use crate::{DBEnv, sys};

pub struct Database<'env, K, V> {
    /// The raw MDB_dbi handle from LMDB. It's a u32 (unsigned int) in C.
    raw_dbi: sys::MDB_dbi,

    /// Keep track of the database name for debugging or re-opening purposes.
    db_name: Option<String>,

    /// PhantomData to tie the DBI's lifetime to the DBEnv it belongs to.
    _marker: PhantomData<(&'env DBEnv, K, V)>,
}

bitflags! {
    /// Flags for the database.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct DBFlags: ffi::c_uint {
        /// Keys are strings to be compared in reverse order, from the end
        /// of the strings to the beginning. By default, Keys are treated as strings and
        /// compared from beginning to end.
        const MDB_REVERSEKEY = sys::MDB_REVERSEKEY;

        /// Duplicate keys may be used in the database. (Or, from another perspective,
        /// keys may have multiple data items, stored in sorted order.) By default
        /// keys must be unique and may have only a single data item.
        const MDB_DUPSORT = sys::MDB_DUPSORT;

        /// Keys are binary integers in native byte order, either unsigned int
        /// or size_t, and will be sorted as such.
        /// The keys must all be of the same size.
        const MDB_INTEGERKEY = sys::MDB_INTEGERKEY;

        /// This flag may only be used in combination with `MDB_DUPSORT`. This option
        /// tells the library that the data items for this database are all the same
        /// size, which allows further optimizations in storage and retrieval. When
        /// all data items are the same size, the `MDB_GET_MULTIPLE`, `MDB_NEXT_MULTIPLE`
        /// and `MDB_PREV_MULTIPLE` cursor operations may be used to retrieve multiple
        /// items at once.
        const MDB_DUPFIXED = sys::MDB_DUPFIXED;

        /// This option specifies that duplicate data items are binary integers,
        /// similar to `MDB_INTEGERKEY` keys.
        const MDB_INTEGERDUP = sys::MDB_INTEGERDUP;

        /// This option specifies that duplicate data items should be compared as
        /// strings in reverse order.
        const MDB_REVERSEDUP = sys::MDB_REVERSEDUP;

        /// Create the named database if it doesn't exist. This option is not
        /// allowed in a read-only transaction or a read-only environment.
        const MDB_CREATE = sys::MDB_CREATE;
    }
}

impl Default for DBFlags {
    fn default() -> Self {
        DBFlags::MDB_CREATE
    }
}

impl<'env, K, V> Database<'env, K, V>
where
    K: AsRef<[u8]>,
    V: AsRef<[u8]>,
{
    pub(crate) fn from_dbi(raw_dbi: sys::MDB_dbi, db_name: Option<String>) -> Self {
        Self {
            raw_dbi,
            db_name,
            _marker: PhantomData,
        }
    }

    pub fn id(&self) -> u32 {
        self.raw_dbi
    }

    pub fn name(&self) -> Option<&str> {
        self.db_name.as_deref()
    }
}
