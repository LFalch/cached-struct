#![warn(clippy::all, missing_docs)]
//! An implementation of types that are cached in a file

use std::{
    io::{self, Read, Write, Result},
    fs::{self, File},
    cell::{RefCell, Ref},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug)]
/// The wrapper type that handles the caching
pub struct Cached<T: Cache> {
    last_modified: RefCell<SystemTime>,
    path: Box<Path>,
    inner: RefCell<T>,
}

impl<T: Cache + Default> Cached<T> {
    #[inline]
    /// Make a new instance using the type's default function
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        Self::new_with(T::default, path)
    }
}

impl<T: Cache> Cached<T> {
    /// Make a new instance using a custom default function
    pub fn new_with<F: FnOnce() -> T, P: Into<PathBuf>>(default: F, path: P) -> Result<Self> {
        let ret = Self {
            last_modified: RefCell::new(SystemTime::UNIX_EPOCH),
            path: path.into().into_boxed_path(),
            inner: RefCell::new(default()),
        };
        ret.check_load().map(|()| ret)
    }
    fn check_load(&self) -> Result<()> {
        let file_last_modified = match fs::metadata(&self.path) {
            Ok(m) => m.modified()?,
            Err(ref e) if e.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e),
        };

        let mut last_modified = self.last_modified.borrow_mut();

        if *last_modified < file_last_modified {
            let file = File::open(&self.path)?;

            *self.inner.borrow_mut() = T::load(&file)?;
            *last_modified = file_last_modified;
        }

        Ok(())
    }
    fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        self.inner.borrow().save(&mut file)?;
        *self.last_modified.borrow_mut() = file.metadata()?.modified()?;
        Ok(())
    }
    /// Get a reference to the inner type
    pub fn get(&self) -> Result<Ref<T>> {
        self.check_load()?;
        Ok(self.inner.borrow())
    }
    /// Applies the given closure to a mutable reference to the inner value
    /// and automatically saves the state to the cache file afterwards
    /// 
    /// **Note**: Currently, it might still return an error, even if the closure was run.
    pub fn do_mut<R, F: FnOnce(&mut T) -> R>(&mut self, f: F) -> Result<R> {
        self.check_load()?;
        let r = f(self.inner.get_mut());
        self.save()?;
        Ok(r)
    }
    /// Consumes the instance, and returns the inner `T`.
    pub fn into_inner(self) -> T {
        let Cached{inner, ..} = self;
        inner.into_inner()
    }
}

/// Trait for the functions on how the cache is saved and loaded
pub trait Cache: Sized {
    /// Write data such that if reading the data would yield the same structure 
    fn save<W: Write>(&self, write: W) -> Result<()>;
    /// Load data that corresponds to the way it's saved
    fn load<R: Read>(reader: R) -> Result<Self>;
}