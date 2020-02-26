use std::cmp::Ordering;
use std::net::Shutdown::Read;
use std::str::FromStr;
use std::iter::empty;
use std::collections::VecDeque;
use std::ops::Deref;
use std::borrow::BorrowMut;
use regex::Regex;

struct Trie {
    filesystems: Node,
}

#[derive(Eq, Debug)]
enum Node {
    Root(Vec<Node>),
    Branch(String, Vec<Node>)
}

enum Relationship {
    Ancestor,
    Sibling,
    Equal,
    Descendant
}

fn parts_from_path(path: &str) -> VecDeque<&str> {
    let dir_entry_pattern = Regex::new("/[^/\0]*").unwrap();
    dir_entry_pattern.find_iter(path).map(|m| m.as_str()).collect()
}

impl Node {
    fn add_str(&mut self, path: &str) {
        let parts: VecDeque<&str> = parts_from_path(path);
        dbg!(&parts);
//        let start: Option<&str> = path
//            .split_terminator("/")
//            .collect::<Vec<&str>>()
//            .first();

        self.add_parts(parts);
    }

    fn add_parts(&mut self, mut parts: VecDeque<&str>) {
        dbg!(&parts);
        let next: &str = match parts.pop_front() {
            Some(s) => s,
            None => return
        };

        if self.has_path(next) {
            self.add_parts(parts)
        } else if self.has_child(next) {
            self.insert_into(next, parts).unwrap();
            //self.children().iter().find(|n| n.has_path(next)).unwrap().add_parts(parts);
        } else {
            let mut new_node = Node::Branch(String::from(next), vec![]);
            new_node.add_parts(parts);
            self.children().push(new_node);
        }
    }

    fn insert_into(&mut self, child: &str, mut parts: VecDeque<&str>) -> Result<(), &str> {
        let child: Option<&mut Node> = self.get_child(child);
        match child {
            Some(c) => Ok(c.add_parts(parts)),
            None => Err("Unable to fild child with correct path")
        }
    }

    fn get_child(&mut self, child: &str) -> Option<&mut Node> {
        self.children().iter_mut().find(|n| n.has_path(child))
    }

    fn path(&self) -> Option<String> {
        match self {
            Node::Root(_) => None,
            Node::Branch(path, _) => Some(path.to_string())
        }
    }

    fn children(&mut self) -> &mut Vec<Node> {
        match self {
            Node::Root(children) => children,
            Node::Branch(_, children) => children
        }
    }

    fn has_child(&mut self, entry: &str) -> bool {
        self.children().iter().any(|n| n.path() == Some(entry.to_string()))
    }

//    fn child_has_path(&self, entry: &str) -> bool {
//        self.children().iter().any(has_path(entry))
//    }

    fn has_path(&self, path: &str) -> bool {
        self.path() == Some(String::from(path))
    }
}

impl PartialEq for Node {
    fn eq(&self, other: &Self) -> bool {
        match self {
            Node::Root(_) => match other {
                Node::Root(_) => true,
                _ => false
            }
            Node::Branch(self_path, self_children) => match other {
                Node::Branch(other_path, other_children) => self_path == other_path && self_children == other_children,
                _ => false
            }
        }
    }

    fn ne(&self, other: &Self) -> bool {
        unimplemented!()
    }
}

//fn connect(n0: Node, n1: Node) -> Node {
//    match relationship(&n0, &n1) {
//        Relationship::Equal => n0,
//        Relationship::Ancestor => n0,
//        Relationship::Descendant => n0,
//        Relationship::Sibling => {
//            let msg: String = format!("Unable to find common ancestor for paths {:?} and {:?}", n0.path(), n1.path());
//            panic!(msg)
//        }
//    }
//}
//
//fn relationship(n0: &Node, n1: &Node) -> Relationship {
//    let mut path0: String = dbg!(n0.path().as_str().chars().rev().collect());
//    let mut path1: String = dbg!(n1.path().as_str().chars().rev().collect());
//    while !path0.is_empty() && !path1.is_empty() {
//        let c0: char = dbg!(path0.pop().unwrap());
//        let c1: char = dbg!(path1.pop().unwrap());
//        if dbg!(c1 != c0) {
//            return Relationship::Sibling
//        }
//    }
//    if path0.len() < path1.len() {
//        Relationship::Ancestor
//    } else if path0.len() > path1.len() {
//        Relationship::Descendant
//    } else {
//        Relationship::Equal
//    }
//}

impl Trie {
    pub fn new(paths: &Vec<&str>) -> Trie {
        let mut paths: Vec<&str> = paths.clone();
        sort_paths(&mut paths);

        let mut root: Node = Node::Root(vec![]);
        for fs in paths {
            dbg!(fs);
            root.add_str(fs)
        }

//        let root: Node = paths
//            .iter()
//            .map(|p| Node::Leaf(p.to_string()))
//            .fold(root, |n0, n1| connect(n0, n1));

        Trie {
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

impl PathResolver for Trie {
    fn resolve_path(&self, path: &String) -> Option<String> {
        resolve_recur(&self.filesystems, path)
    }
}

fn resolve_recur(node: &Node, path: &String) -> Option<String> {
    match node {
        Node::Root(children) => None,
        Node::Branch(fs, children) => None
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
    use crate::trie::{Trie, Node, parts_from_path};
    use std::collections::VecDeque;

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
    fn test_parts_from_path_with_just_root() {
        let mut parts: VecDeque<&str> = parts_from_path("/");
        assert_eq!(1, parts.len());
        assert_eq!(Some("/"), parts.pop_front());
    }

    #[test]
    fn test_parts_from_path_with_just_one_dir_after_root() {
        let mut parts: VecDeque<&str> = parts_from_path("/proc");
        assert_eq!(1, parts.len());
        assert_eq!(Some("/proc"), parts.pop_front());
    }

    #[test]
    fn test_parts_from_path_with_several_dirs() {
        let mut parts: VecDeque<&str> = parts_from_path("/sys/fs/cgroup/memory");
        assert_eq!(4, parts.len());
        let expected: VecDeque<&str> = VecDeque::from(vec!["/syss", "/fs", "/cgroup", "/memory"]);
        assert_eq!(expected, parts);
    }

    ///
    /// "/"
    /// "/proc",
    /// "/sys",
    /// "/sys/firmware",
    /// "/sys/fs",
    /// "/sys/fs/cgroup/cpu,cpuacct",
    ///
    ///     |----- / ----|
    ///  /proc         /sys ----------|--------------|
    ///                         /sys/firmware     /sys/fs -------|
    ///                                            /sys/fs/cgroup/cpu,cpuacct
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

        let tree = Trie::new(&paths);

        assert_eq!("/", tree.filesystems.path().unwrap());

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
                let children: Vec<String> = children.iter().map(|n| n.path().unwrap()).collect();
                assert_eq!(expected_children, children)
            },
            _ => panic!("Fail")
        }
    }
}
