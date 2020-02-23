use crate::prefix_tree::Node::Leaf;
use std::cmp::Ordering;
use std::net::Shutdown::Read;
use std::str::FromStr;

struct PrefixTree {
    filesystems: Node,
}

#[derive(Eq, Debug)]
enum Node {
    Branch(String, Vec<Node>),
    Leaf(String),
}

enum Relationship {
    Ancestor,
    Sibling,
    Equal,
    Descendant
}

impl Node {
    fn path(&self) -> String {
        match self {
            Node::Branch(path, _) => path,
            Node::Leaf(path) => path
        }.parse().unwrap()
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Node::Branch(self_path, self_children) => match other {
                Node::Branch(other_path, other_children) => self_path == other_path && self_children == other_children,
                Node::Leaf(_) => false
            },
            Node::Leaf(self_path) => match other {
                Node::Branch(_, _) => false,
                Node::Leaf(other_path) => self_path == other_path
            }
        }
    }

    fn ne(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

fn connect(n0: Node, n1: Node) -> Node {
    match relationship(&n0, &n1) {
        Relationship::Equal => n0,
        Relationship::Ancestor => n0,
        Relationship::Descendant => n0,
        Relationship::Sibling => {
            let msg: String = format!("Unable to find common ancestor for paths {} and {}", n0.path(), n1.path());
            panic!(msg)
        }
    }
}

fn relationship(n0: &Node, n1: &Node) -> Relationship {
    let mut path0: String = dbg!(n0.path().as_str().chars().rev().collect());
    let mut path1: String = dbg!(n1.path().as_str().chars().rev().collect());
    while !path0.is_empty() && !path1.is_empty() {
        let c0: char = dbg!(path0.pop().unwrap());
        let c1: char = dbg!(path1.pop().unwrap());
        if dbg!(c1 != c0) {
            return Relationship::Sibling
        }
    }
    if path0.len() < path1.len() {
        Relationship::Ancestor
    } else if path0.len() > path1.len() {
        Relationship::Descendant
    } else {
        Relationship::Equal
    }
}

impl PrefixTree {
    pub fn new(paths: &Vec<&str>) -> PrefixTree {
        let mut paths: Vec<&str> = paths.clone();
        sort_paths(&mut paths);

        let root: Node = Node::Leaf(String::from("/"));

        let root: Node = paths
            .iter()
            .map(|p| Node::Leaf(p.to_string()))
            .fold(root, |n0, n1| connect(n0, n1));

        PrefixTree {
            filesystems: dbg!(root)
        }
    }
//
//    fn build_tree(paths: &mut Vec<&str>, node: &mut Node) -> Node {
//        paths.po
//    }
}

trait PathResolver {
    fn resolve_path(&self, path: &String) -> Option<String>;
}

impl PathResolver for PrefixTree {
    fn resolve_path(&self, path: &String) -> Option<String> {
        resolve_recur(&self.filesystems, path)
    }
}

fn resolve_recur(node: &Node, path: &String) -> Option<String> {
    match node {
        Node::Branch(fs, children) => None,
        Node::Leaf(fs) => None,
    }
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
    use crate::prefix_tree::{PrefixTree, Node};

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

    #[test]
    fn create_prefix_tree() {
        let mut paths: Vec<&str> = vec![
            "/proc",
            "/sys",
            "/sys/firmware",
            "/sys/fs",
            "/sys/fs/cgroup/cpu,cpuacct",
            "/dev",
            "/run",
            "/",
            "/tmp",
            "/home",
            "/boot",
        ];

        let tree = PrefixTree::new(&paths);

        assert_eq!("/", tree.filesystems.path());

        let expected_children: Vec<String> = vec![
            "/proc",
            "/sys",
            "/dev",
            "/run",
            "/tmp",
            "/home",
            "/boot"
        ].iter().map(|path| path.to_string()).collect();

        match tree.filesystems {
            Node::Branch(p, children) => {
                let children: Vec<String> = children.iter().map(|n| n.path()).collect();
                assert_eq!(expected_children, children)
            },
            Node::Leaf(_) => panic!("Fail")
        }
    }
}
