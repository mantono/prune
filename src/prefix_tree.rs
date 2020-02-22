use crate::prefix_tree::Node::Leaf;
use std::cmp::Ordering;

struct PrefixTree {
    filesystems: Node,
}

enum Node {
    Child(String, Vec<Node>),
    Leaf(String),
}

impl PrefixTree {
    fn new(paths: &Vec<&str>) -> PrefixTree {
        let mut paths: Vec<&str> = paths.clone();
        sort_paths(&mut paths);

        PrefixTree {
            // TODO: Initialize properly!
            filesystems: Leaf(String::from("/")),
        }
    }
}

trait PathResolver {
    fn resolve_path(path: &String) -> String;
}

/// Sort paths so paths with least amount of sub directories comes first,
/// and addtionally, so the root path `/` always comes first.
fn sort_paths(paths: &mut Vec<&str>) {
    paths.sort_by(|p0, p1| {
        let p0_len = p0.matches('/').count();
        let p1_len = p1.matches('/').count();
        if p0_len < p1_len || *p0 == "/" {
            Ordering::Less
        } else if p0_len > p1_len || *p1 == "/" {
            Ordering::Greater
        } else {
            Ordering::Equal
        }
    });
}

#[cfg(test)]
mod tests {

    #[test]
    fn test_sort_path() {
        let mut paths: Vec<&str> = vec![
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
        ];

        super::sort_paths(&mut paths);

        let expected: Vec<&str> = vec![
            "/",
            "/proc",
            "/sys",
            "/dev",
            "/run",
            "/tmp",
            "/home",
            "/boot",
            "/sys/kernel/security",
            "/sys/firmware/efi/efivars",
            "/sys/fs/cgroup/memory",
            "/sys/fs/cgroup/cpu,cpuacct",
            "/sys/fs/cgroup/freezer",
        ];

        assert_eq!(expected, paths);
    }
}
