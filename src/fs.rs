use std::fs::Metadata;
use std::path::PathBuf;
use std::time::SystemTime;
use std::{ffi::OsString, hash::Hasher};

#[derive(Debug, Eq)]
pub struct FsEntity {
    path: OsString,
    size: u64,
    mod_time: SystemTime,
}

impl FsEntity {
    pub fn from<T: Into<PathBuf>>(path: T) -> Result<FsEntity, std::io::Error> {
        FsEntity::from_path_buf(path.into())
    }

    #[inline]
    pub fn from_path_buf(path: PathBuf) -> Result<FsEntity, std::io::Error> {
        let metadata: Metadata = path.metadata()?;
        let size: u64 = metadata.len();

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
        };

        Ok(entity)
    }

    pub fn len(&self) -> u64 {
        self.size
    }

    pub fn to_path_buf(&self) -> PathBuf {
        PathBuf::from(&self.path)
    }

    pub fn matches(&self, regex: &regex::Regex) -> bool {
        match self.path.to_str() {
            Some(p) => regex.is_match(p),
            None => false,
        }
    }

    pub fn last_modified(&self) -> SystemTime {
        self.mod_time
    }

    pub fn path(&self) -> Option<&str> {
        self.path.to_str()
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

impl From<FsEntity> for PathBuf {
    fn from(entity: FsEntity) -> Self {
        PathBuf::from(entity.path)
    }
}

impl std::fmt::Display for FsEntity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.path)
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

#[cfg(test)]
mod tests {
    use crate::fs::FsEntity;

    #[test]
    fn test_create_fs_entity_file() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("test_dirs/sub_dir/file1");
        let entity: FsEntity = entity.unwrap();
        assert!(entity.len() > 0);
    }

    #[test]
    fn test_create_fs_entity_dir() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("test_dirs");
        let entity: FsEntity = entity.unwrap();
        assert_eq!(0, entity.len());
    }

    #[test]
    fn test_create_fs_entity_none_existing() {
        let entity: Result<FsEntity, std::io::Error> = FsEntity::from("foo");
        assert!(entity.is_err());
    }
}
