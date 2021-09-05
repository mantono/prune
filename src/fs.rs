use std::{
    fs::Metadata,
    path::{Path, PathBuf},
};

use walkdir::DirEntry;

pub trait FsEntity {
    fn path(&self) -> &Path;
    fn metadata(&self) -> std::io::Result<Metadata>;
}

impl FsEntity for DirEntry {
    fn path(&self) -> &Path {
        DirEntry::path(&self)
    }

    fn metadata(&self) -> std::io::Result<Metadata> {
        DirEntry::metadata(&self).map_err(walkdir::Error::into)
    }
}

impl FsEntity for PathBuf {
    fn path(&self) -> &Path {
        PathBuf::as_path(&self)
    }

    fn metadata(&self) -> std::io::Result<Metadata> {
        std::fs::metadata(&self)
    }
}
