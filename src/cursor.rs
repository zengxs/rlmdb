use std::ptr::NonNull;

use crate::Transaction;

pub struct Cursor<'txn, K, V> {
    #[allow(dead_code)]
    ptr: NonNull<crate::sys::MDB_cursor>,

    _marker: std::marker::PhantomData<(&'txn Transaction<'txn>, K, V)>,
}

impl<'txn, K, V> Cursor<'txn, K, V>
where
    K: AsRef<[u8]> + for<'a> From<&'a [u8]>,
    V: AsRef<[u8]> + for<'a> From<&'a [u8]>,
{
    // TODO
}
