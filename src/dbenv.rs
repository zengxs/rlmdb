use std::{ffi, fs, path::PathBuf, ptr::NonNull};

use bitflags::bitflags;

use crate::{DBFlags, Database, LMDBError, Transaction, TransactionType, sys};

bitflags! {
    /// Flags for the database environment.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct EnvFlags: ffi::c_uint {
        const MDB_FIXEDMAP = sys::MDB_FIXEDMAP;
        const MDB_NOSUBDIR = sys::MDB_NOSUBDIR;
        const MDB_RDONLY = sys::MDB_RDONLY;
        const MDB_WRITEMAP = sys::MDB_WRITEMAP;
        const MDB_NOMETASYNC = sys::MDB_NOMETASYNC;
        const MDB_NOSYNC = sys::MDB_NOSYNC;
        const MDB_MAPASYNC = sys::MDB_MAPASYNC;
        const MDB_NOTLS = sys::MDB_NOTLS;
        const MDB_NOLOCK = sys::MDB_NOLOCK;
        const MDB_NORDAHEAD = sys::MDB_NORDAHEAD;
        const MDB_NOMEMINIT = sys::MDB_NOMEMINIT;
    }
}

impl Default for EnvFlags {
    fn default() -> Self {
        EnvFlags::MDB_NOSUBDIR
    }
}

pub struct DBEnv {
    ptr: NonNull<sys::MDB_env>,
}

#[allow(unused)]
impl DBEnv {
    pub(super) fn from_ptr(ptr: NonNull<sys::MDB_env>) -> Self {
        Self { ptr }
    }

    pub fn sync(&self, force: bool) -> Result<(), LMDBError> {
        let force = if force { 1 } else { 0 };

        let ret = unsafe { sys::mdb_env_sync(self.as_raw_ptr(), force) };
        LMDBError::from_mdb_error(ret)
    }

    pub fn stat(&self) -> Result<sys::MDB_stat, LMDBError> {
        todo!()
    }

    pub fn begin_txn(&self) -> Result<Transaction, LMDBError> {
        Transaction::new(self, None, TransactionType::ReadWrite)
    }

    pub fn begin_txn_read_only(&self) -> Result<Transaction, LMDBError> {
        Transaction::new(self, None, TransactionType::ReadOnly)
    }

    pub fn open_db<K, V>(
        &self,
        txn: &'_ Transaction,
        flags: Option<DBFlags>,
    ) -> Result<Database<K, V>, LMDBError>
    where
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.open_db_internal::<&str, K, V>(txn, None, flags)
    }

    pub fn open_named_db<S, K, V>(
        &self,
        txn: &'_ Transaction,
        name: S,
        flags: Option<DBFlags>,
    ) -> Result<Database<K, V>, LMDBError>
    where
        S: AsRef<str>,
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        self.open_db_internal(txn, Some(name), flags)
    }

    fn open_db_internal<S, K, V>(
        &self,
        txn: &'_ Transaction,
        name: Option<S>,
        flags: Option<DBFlags>,
    ) -> Result<Database<K, V>, LMDBError>
    where
        S: AsRef<str>,
        K: AsRef<[u8]>,
        V: AsRef<[u8]>,
    {
        let flags = flags.unwrap_or(DBFlags::default());

        let name_cstr = name
            .map(|n| {
                ffi::CString::new(n.as_ref()).map_err(|_| {
                    LMDBError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid database name",
                    ))
                })
            })
            .transpose()?;
        let name_ptr = name_cstr.as_ref().map_or(std::ptr::null(), |s| s.as_ptr());

        let mut dbi: sys::MDB_dbi = Default::default();

        let ret = unsafe { sys::mdb_dbi_open(txn.as_raw_ptr(), name_ptr, flags.bits(), &mut dbi) };
        LMDBError::from_mdb_error(ret)?;

        Ok(Database::from_dbi(
            dbi,
            name_cstr.map(|s| s.into_string().unwrap()),
        ))
    }

    pub fn drop_db<S>(&self, name: Option<S>) -> Result<(), LMDBError>
    where
        S: AsRef<str>,
    {
        todo!()
    }

    pub fn as_ptr(&self) -> NonNull<sys::MDB_env> {
        self.ptr
    }

    pub unsafe fn as_raw_ptr(&self) -> *mut sys::MDB_env {
        self.ptr.as_ptr()
    }
}

impl Drop for DBEnv {
    fn drop(&mut self) {
        unsafe {
            sys::mdb_env_close(self.ptr.as_ptr());
        }
    }
}

#[derive(Debug, Clone)]
pub struct DBEnvBuilder {
    db_path: PathBuf,

    file_mode: Option<fs::Permissions>,

    map_size: Option<usize>,

    max_readers: Option<usize>,

    max_dbs: Option<usize>,
}

impl DBEnvBuilder {
    pub fn new<P: Into<PathBuf>>(path: P) -> Self {
        Self {
            db_path: path.into(),
            file_mode: None,
            map_size: None,
            max_readers: None,
            max_dbs: None,
        }
    }

    pub fn set_file_mode(&mut self, mode: fs::Permissions) -> &mut Self {
        self.file_mode = Some(mode);
        self
    }

    pub fn set_map_size(&mut self, size: usize) -> &mut Self {
        self.map_size = Some(size);
        self
    }

    pub fn set_max_readers(&mut self, max_readers: usize) -> &mut Self {
        self.max_readers = Some(max_readers);
        self
    }

    pub fn set_max_dbs(&mut self, max_dbs: usize) -> &mut Self {
        self.max_dbs = Some(max_dbs);
        self
    }

    /// Builds the `DBEnv` with the specified flags.
    pub fn open(&self, flags: Option<EnvFlags>) -> Result<DBEnv, LMDBError> {
        let flags = flags.unwrap_or_else(|| EnvFlags::default());

        let path_cstr =
            ffi::CString::new(self.db_path.to_string_lossy().as_bytes()).map_err(|_| {
                LMDBError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    "Invalid path for LMDB environment",
                ))
            })?;

        let mut env_ptr: *mut sys::MDB_env = std::ptr::null_mut();

        let ret = unsafe { sys::mdb_env_create(&mut env_ptr) };
        LMDBError::from_mdb_error(ret)?;
        let env_ptr = NonNull::new(env_ptr).ok_or_else(|| {
            LMDBError::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "mdb_env_create succeeded but returned a null environment pointer",
            ))
        })?;

        if let Some(map_size) = self.map_size {
            let ret = unsafe { sys::mdb_env_set_mapsize(env_ptr.as_ptr(), map_size) };
            LMDBError::from_mdb_error(ret)?;
        }

        if let Some(max_readers) = self.max_readers {
            let ret = unsafe { sys::mdb_env_set_maxreaders(env_ptr.as_ptr(), max_readers as u32) };
            LMDBError::from_mdb_error(ret)?;
        }

        if let Some(max_dbs) = self.max_dbs {
            let ret = unsafe { sys::mdb_env_set_maxdbs(env_ptr.as_ptr(), max_dbs as u32) };
            LMDBError::from_mdb_error(ret)?;
        }

        let env = DBEnv::from_ptr(env_ptr);

        #[cfg(unix)]
        let file_mode = {
            use std::os::unix::fs::PermissionsExt;
            self.file_mode.as_ref().map(|p| p.mode()).unwrap_or(0o644)
        };
        #[cfg(not(unix))]
        let file_mode = 0;

        let ret = unsafe {
            sys::mdb_env_open(
                env.as_raw_ptr(),
                path_cstr.as_ptr(),
                flags.bits(),
                file_mode,
            )
        };
        LMDBError::from_mdb_error(ret)?;

        Ok(env)
    }
}
