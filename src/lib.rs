#![warn(clippy::all, missing_docs)]

use std::{
    io::{self, Read, Write, Result},
    fs::{self, File},
    cell::{RefCell, Ref},
    path::{Path, PathBuf},
    time::SystemTime,
};

#[derive(Debug)]
pub struct Cached<T: Cache> {
    last_modified: RefCell<SystemTime>,
    path: Box<Path>,
    inner: RefCell<T>,
}

impl<T: Cache + Default> Cached<T> {
    #[inline]
    pub fn new<P: Into<PathBuf>>(path: P) -> Result<Self> {
        Self::new_with(T::default, path)
    }
}

impl<T: Cache> Cached<T> {
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
    pub fn save(&self) -> Result<()> {
        let mut file = File::create(&self.path)?;

        self.inner.borrow().save(&mut file)?;
        *self.last_modified.borrow_mut() = file.metadata()?.modified()?;
        Ok(())
    }
    pub fn get(&self) -> Result<Ref<T>> {
        self.check_load()?;
        Ok(self.inner.borrow())
    }
    pub fn get_mut(&mut self) -> Result<&mut T> {
        self.check_load()?;
        Ok(self.inner.get_mut())
    }
}

impl<T: Cache> Drop for Cached<T> {
    fn drop(&mut self) {
        self.save().unwrap()
    }
} 

pub trait Cache: Sized {
    fn save<W: Write>(&self, write: W) -> Result<()>;
    fn load<R: Read>(reader: R) -> Result<Self>;
}