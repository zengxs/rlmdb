use std::io;

use crate::sys;

#[derive(Debug, thiserror::Error)]
pub enum LMDBError {
    /// LMDB function returned an error.
    #[error(transparent)]
    MDB(#[from] MDBError),

    /// An underlying I/O error occurred (mapped from standard C errno).
    #[error(transparent)]
    Io(#[from] io::Error),
}

/// LMDB ffi error type.
/// This is used to convert LMDB error codes into Rust errors.
#[derive(Debug, thiserror::Error)]
pub enum MDBError {
    /// key/data pair already exists
    #[error("MDB_KEYEXIST: Key/data pair already exists")]
    KeyExists,

    /// key/data pair not found (EOF)
    #[error("MDB_NOTFOUND: No matching key/data pair found")]
    NotFound,

    /// Requested page not found - this usually indicates corruption
    #[error("MDB_PAGE_NOTFOUND: Requested page not found")]
    PageNotFound,

    /// Located page was wrong type
    #[error("MDB_CORRUPTED: Located page was wrong type")]
    Corrupted,

    /// Update of meta page failed or environment had fatal error
    #[error("MDB_PANIC: Update of meta page failed or environment had fatal error")]
    Panic,

    /// Environment version mismatch
    #[error("MDB_VERSION_MISMATCH: Database environment version mismatch")]
    VersionMismatch,

    /// File is not a valid LMDB file
    #[error("MDB_INVALID: File is not an LMDB file")]
    Invalid,

    /// Environment mapsize reached
    #[error("MDB_MAP_FULL: Environment mapsize limit reached")]
    MapFull,

    /// Environment maxdbs reached
    #[error("MDB_DBS_FULL: Environment maxdbs limit reached")]
    DbsFull,

    /// Environment maxreaders reached
    #[error("MDB_READERS_FULL: Environment maxreaders limit reached")]
    ReadersFull,

    /// Too many TLS keys in use - Windows only
    #[error("MDB_TLS_FULL: Thread-local storage keys full - too many environments open")]
    TlsFull,

    /// Txn has too many dirty pages
    #[error("MDB_TXN_FULL: Transaction has too many dirty pages - transaction too big")]
    TxnFull,

    /// Cursor stack too deep - internal error
    #[error("MDB_CURSOR_FULL: Internal error - cursor stack limit reached")]
    CursorFull,

    /// Page has not enough space - internal error
    #[error("MDB_PAGE_FULL: Internal error - page has no more space")]
    PageFull,

    /// Database contents grew beyond environment mapsize
    #[error("MDB_MAP_RESIZED: Database contents grew beyond environment mapsize")]
    MapResized,

    /// Operation and DB incompatible, or DB type changed. This can mean:
    /// * The operation expects an `MDB_DUPSORT` / `MDB_DUPFIXED` database.
    /// * Opening a named DB when the unnamed DB has `MDB_DUPSORT` / `MDB_INTEGERKEY`.
    /// * Accessing a data record as a database, or vice versa.
    /// * The database was dropped and recreated with different flags.
    #[error("MDB_INCOMPATIBLE: Operation and DB incompatible, or DB flags changed")]
    Incompatible,

    /// Invalid reuse of reader locktable slot
    #[error("MDB_BAD_RSLOT: Invalid reuse of reader locktable slot")]
    BadRslot,

    /// Transaction must abort, has a child, or is invalid
    #[error("MDB_BAD_TXN: Transaction must abort, has a child, or is invalid")]
    BadTxn,

    /// Unsupported size of key/DB name/data, or wrong `DUPFIXED` size
    #[error("MDB_BAD_VALSIZE: Unsupported size of key/DB name/data, or wrong DUPFIXED size")]
    BadValSize,

    /// The specified DBI was changed unexpectedly
    #[error("MDB_BAD_DBI: The specified DBI handle was closed/changed unexpectedly")]
    BadDbi,
}

impl LMDBError {
    pub fn from_mdb_error(err_code: i32) -> Result<(), Self> {
        if err_code == sys::MDB_SUCCESS as i32 {
            Ok(())
        } else {
            let mdb_err = match err_code {
                sys::MDB_KEYEXIST => MDBError::KeyExists,
                sys::MDB_NOTFOUND => MDBError::NotFound,
                sys::MDB_PAGE_NOTFOUND => MDBError::PageNotFound,
                sys::MDB_CORRUPTED => MDBError::Corrupted,
                sys::MDB_PANIC => MDBError::Panic,
                sys::MDB_VERSION_MISMATCH => MDBError::VersionMismatch,
                sys::MDB_INVALID => MDBError::Invalid,
                sys::MDB_MAP_FULL => MDBError::MapFull,
                sys::MDB_DBS_FULL => MDBError::DbsFull,
                sys::MDB_READERS_FULL => MDBError::ReadersFull,
                sys::MDB_TLS_FULL => MDBError::TlsFull,
                sys::MDB_TXN_FULL => MDBError::TxnFull,
                sys::MDB_CURSOR_FULL => MDBError::CursorFull,
                sys::MDB_PAGE_FULL => MDBError::PageFull,
                sys::MDB_MAP_RESIZED => MDBError::MapResized,
                sys::MDB_INCOMPATIBLE => MDBError::Incompatible,
                sys::MDB_BAD_RSLOT => MDBError::BadRslot,
                sys::MDB_BAD_TXN => MDBError::BadTxn,
                sys::MDB_BAD_VALSIZE => MDBError::BadValSize,
                sys::MDB_BAD_DBI => MDBError::BadDbi,
                _ => {
                    let io_err = io::Error::from_raw_os_error(err_code);
                    return Err(LMDBError::Io(io_err));
                }
            };
            Err(LMDBError::MDB(mdb_err))
        }
    }
}
