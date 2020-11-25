use std::fs::Metadata;
use std::io::ErrorKind;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{
    ffi::{OsStr, OsString},
    hash::Hasher,
};

#[derive(Debug, Eq)]
pub struct FsEntity {
    path: OsString,
    size: Option<u64>,
    mod_time: SystemTime,
    kind: FsKind,
}

impl FsEntity {
    pub fn from<T: Into<PathBuf>>(path: T) -> Result<FsEntity, std::io::Error> {
        let path: PathBuf = path.into();
        if !path.exists() {
            return Err(std::io::Error::new(
                ErrorKind::NotFound,
                "Entity does not exist",
            ));
        }

        let path: PathBuf = path.canonicalize()?;
        let metadata: Metadata = path.metadata()?;

        let kind: FsKind = if metadata.is_dir() {
            FsKind::Dir
        } else if metadata.is_file() {
            FsKind::File
        } else {
            FsKind::Unknown
        };

        let size: Option<u64> = match kind {
            FsKind::File => Some(metadata.len()),
            _ => None,
        };

        let mod_time: SystemTime = match metadata.modified() {
            Ok(time) => time,
            Err(_) => match metadata.created() {
                Ok(time) => time,
                Err(_) => SystemTime::UNIX_EPOCH,
            },
        };

        let entity = FsEntity {
            path: path.into_os_string(),
            size,
            mod_time,
            kind,
        };

        Ok(entity)
    }

    pub fn len(&self) -> u64 {
        self.size.unwrap_or(0)
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    pub fn is_file(&self) -> bool {
        self.kind == FsKind::File
    }

    pub fn is_dir(&self) -> bool {
        self.kind == FsKind::Dir
    }

    pub fn kind(&self) -> FsKind {
        self.kind
    }

    pub fn file_name(&self) -> Option<String> {
        match self.to_path_buf().file_name() {
            Some(filename) => match filename.to_str() {
                Some(filename) => Some(filename.to_string()),
                None => None,
            },
            None => None,
        }
    }

    pub fn last_modified(&self) -> SystemTime {
        self.mod_time
    }

    pub fn path(&self) -> &str {
        self.path.to_str().expect("Failed to convert path to &str")
    }

    pub fn parent(&self) -> Option<FsEntity> {
        match self.to_path_buf().parent() {
            Some(parent) => FsEntity::from(parent).ok(),
            None => None,
        }
    }
}

impl std::cmp::PartialEq for FsEntity {
    fn eq(&self, other: &Self) -> bool {
        self.path == other.path
    }
}

impl std::hash::Hash for FsEntity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.path.hash(state)
    }
}

impl Into<PathBuf> for FsEntity {
    fn into(self) -> PathBuf {
        PathBuf::from(&self.path)
    }
}

impl std::fmt::Display for FsEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}: {:?}", self.kind(), self.path)
    }
}

impl std::cmp::PartialOrd for FsEntity {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.path.partial_cmp(&other.path)
    }
}

impl std::cmp::Ord for FsEntity {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.path.cmp(&other.path)
    }
}

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
pub enum FsKind {
    File,
    Dir,
    Unknown,
}

#[cfg(test)]
mod tests {
    use crate::fs::FsEntity;
    use crate::fs::FsKind;

    #[test]
    fn test_create_fs_entity_file() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("test_dirs/sub_dir/file1");
        let entity: FsEntity = entity.unwrap();
        assert!(entity.len() > 0);
        assert_eq!(FsKind::File, entity.kind());
    }

    #[test]
    fn test_create_fs_entity_dir() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("test_dirs");
        let entity: FsEntity = entity.unwrap();
        assert_eq!(0, entity.len());
        assert_eq!(FsKind::Dir, entity.kind());
    }

    #[test]
    fn test_create_fs_entity_none_existing() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("foo");
        assert!(entity.is_err());
    }
}
