use std::{io, path::{Component, PathBuf}, fs::DirEntry};

pub trait WorkEntryPathExt {
    fn is_root(&self) -> bool;
    fn is_curdir_or_parent(&self) -> bool;
}

impl WorkEntryPathExt for PathBuf {
    fn is_root(&self) -> bool {
        return self.canonicalize().is_ok_and(|x| x.parent() == None);
    }

    fn is_curdir_or_parent(&self) -> bool {
        return match self.components().last() {
            Some(Component::CurDir) | Some(Component::ParentDir) => true,
            _ => false
        };
    }
}

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
