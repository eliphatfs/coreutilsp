use std::{io, path::PathBuf, fs::DirEntry};

pub trait WorkEntry {
    fn is_dir(&self) -> io::Result<bool>;
    fn path(&self) -> PathBuf;
}

impl WorkEntry for PathBuf {
    fn is_dir(&self) -> io::Result<bool> {
        Ok(self.symlink_metadata()?.is_dir())
    }
    fn path(&self) -> PathBuf {
        self.clone()
    }
}

impl WorkEntry for DirEntry {
    fn is_dir(&self) -> io::Result<bool> {
        Ok(self.file_type()?.is_dir())
    }
    fn path(&self) -> PathBuf {
        self.path()
    }
}
