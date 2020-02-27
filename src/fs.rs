use std::path::PathBuf;

pub fn fs_boundaries(filesystems: &Vec<PathBuf>, path: &PathBuf) -> Vec<PathBuf> {
    filesystems.clone()
        .iter()
        .filter(|fs| fs.starts_with(path))
        .map(|fs| PathBuf::from(fs))
        .collect()
}

const LINUX_MOUNTS_FILE: &str = "/proc/mounts";

/// On Linux, read mounted file systems for /proc/mounts and cross reference
/// them with paths to search with, and filter out any overlaps.
///
/// Mac OS X should be similar. Have no idea how to solve Windows, yet.
pub fn filesystems() -> Result<Vec<PathBuf>, std::io::Error> {
    let mounts: String = std::fs::read_to_string(LINUX_MOUNTS_FILE)?;

    let mounts: Vec<PathBuf> = mounts
        .lines()
        .map(|line: &str| line.split_ascii_whitespace().skip(1).next().unwrap())
        .map(|p| PathBuf::from(p))
        .collect();

    Ok(mounts)
}

#[cfg(test)]
mod tests {
    use crate::fs::fs_boundaries;
    use std::path::PathBuf;

    #[test]
    fn test_fs_boundaries_no_boundary() {
        let filesystems: Vec<PathBuf> = vec![
            "/proc",
            "/sys",
            "/sys/firmware/efi/efivars",
            "/dev",
            "/run",
            "/",
            "/tmp",
            "/home",
            "/boot",
            "/sys/kernel/security",
            "/sys/fs/cgroup/memory",
            "/sys/fs/cgroup/cpu,cpuacct",
            "/sys/fs/cgroup/freezer",
        ].iter().map(|p| PathBuf::from(p)).collect();
        let path: PathBuf = PathBuf::from("/home/user");

        let boundaries: Vec<PathBuf> = fs_boundaries(&filesystems, &path);
        assert_eq!(true, boundaries.is_empty())
    }

    #[test]
    fn test_fs_boundaries_single_boundary() {
        let filesystems: Vec<PathBuf> = vec![
            "/proc",
            "/sys",
            "/sys/firmware/efi/efivars",
            "/dev",
            "/run",
            "/",
            "/tmp",
            "/home",
            "/boot",
            "/sys/kernel/security",
            "/sys/fs/cgroup/memory",
            "/sys/fs/cgroup/cpu,cpuacct",
            "/sys/fs/cgroup/freezer",
        ].iter().map(|p| PathBuf::from(p)).collect();

        let path = PathBuf::from("/sys/kernel");
        let boundaries: Vec<PathBuf> = fs_boundaries(&filesystems, &path);
        let expected = vec![PathBuf::from("/sys/kernel/security")];
        assert_eq!(expected, boundaries)
    }

    #[test]
    fn test_fs_boundaries_multiple_boundaries() {
        let filesystems: Vec<PathBuf> = vec![
            "/proc",
            "/sys",
            "/sys/firmware/efi/efivars",
            "/dev",
            "/run",
            "/",
            "/tmp",
            "/home",
            "/boot",
            "/sys/kernel/security",
            "/sys/fs/cgroup/memory",
            "/sys/fs/cgroup/cpu,cpuacct",
            "/sys/fs/cgroup/freezer",
        ].iter().map(|p| PathBuf::from(p)).collect();

        let path: PathBuf = PathBuf::from("/sys/fs");
        let boundaries: Vec<PathBuf> = fs_boundaries(&filesystems, &path);
        let expected = vec![
            PathBuf::from("/sys/fs/cgroup/memory"),
            PathBuf::from("/sys/fs/cgroup/cpu,cpuacct"),
            PathBuf::from("/sys/fs/cgroup/freezer")
        ];
        assert_eq!(expected, boundaries)
    }
}