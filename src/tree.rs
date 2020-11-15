use itertools::Itertools;
use std::borrow::{Borrow, BorrowMut};
use std::cmp::{max, Ordering};
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;
use std::path::PathBuf;

#[derive(Debug)]
struct Tree {
    nodes: Vec<Node>,
}

impl Tree {
    pub fn from(value: PathBuf, initial_capacity: usize) -> Tree {
        let mut tree = Tree {
            nodes: Vec::with_capacity(initial_capacity),
        };

        let node = Node {
            value,
            parent: None,
            children: None,
        };
        tree.nodes.push(node);
        tree
    }

    pub fn add_into(self, value: PathBuf) -> Tree {
        let index: usize = self.last_index();
        self.add_into_index(index, value)
    }

    fn last_index(&self) -> usize {
        self.nodes.len() - 1
    }

    fn add_into_index(self, index: usize, value: PathBuf) -> Tree {
        let node: &Node = match self.get_node(index) {
            Some(node) => node,
            None => panic!("Not possible?"),
        };

        let relation: Relation = node.value.relation(&value);
        self.on_relation(index, value, relation)
    }

    fn get_node(&self, index: usize) -> Option<&Node> {
        self.nodes.get(index)
    }

    fn get_mut_node(&mut self, index: usize) -> Option<&mut Node> {
        self.nodes.get_mut(index)
    }

    fn on_relation(self, index: usize, value: PathBuf, relation: Relation) -> Tree {
        match relation {
            Relation::Equal => self,
            Relation::Siblings => self.handle_siblings(index, value),
            Relation::Ancestor => self.handle_ancestor(index, value),
            _ => panic!("Not supported yet: {:?}", relation),
        }
    }

    fn handle_ancestor(self, index: usize, value: PathBuf) -> Tree {}

    fn handle_siblings(self, index: usize, value: PathBuf) -> Tree {
        let parent_index: usize = self
            .get_node(index)
            .unwrap()
            .parent
            .expect("Must have parent");
        let new_node = Node {
            value,
            parent: Some(parent_index),
            children: None,
        };
        self.add_node(parent_index, new_node)
    }

    fn add_node(mut self, parent_index: usize, node: Node) -> Tree {
        self.nodes.push(node);
        let last_index: usize = self.last_index();
        self.get_mut_node(parent_index)
            .unwrap()
            .add_child(last_index);
        self
    }
}

#[derive(Debug)]
struct Node {
    value: PathBuf,
    parent: Option<usize>,
    children: Option<Vec<usize>>,
}

impl Node {
    pub fn add_child(&mut self, index: usize) {
        match &mut self.children {
            Some(children) => children.push(index),
            None => {
                let mut children = Vec::with_capacity(2);
                children.push(index);
                self.children = Some(children)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::tree::Tree;
    use std::path::PathBuf;

    #[test]
    ///
    /// ```text
    /// PATH                        | Index Parent  Children
    /// /home/arthur                │ 0             [1, 2]
    ///     /home/arthur/foo        | 1     0
    ///     /home/arthur/bar        | 2     0       [3]
    ///     /home/arthur/bar/docs   | 3     2
    /// /home/trillian              | 4
    /// ```
    /// Inserting `/home/arthur/bar/files`;
    /// 1. Take a random index from the indices of the vector: 3
    /// 2. Compare value at index 3 with inserting element;
    /// `/home/arthur/bar/docs` cmp `/home/arthur/bar/files` => Greater
    /// 3. Compare
    fn test_add_to_tree() {
        let arthur = PathBuf::from("/home/arthur");
        let arthur_foo = PathBuf::from("/home/arthur/foo");
        let arthur_bar = PathBuf::from("/home/arthur/bar");
        let arthur_bar_docs = PathBuf::from("/home/arthur/bar/docs");
        let arthur_bar_files = PathBuf::from("/home/arthur/bar/files");
        let trillian = PathBuf::from("/home/trillian");

        let tree = Tree::from(arthur, 8)
            .add_into(arthur_foo)
            .add_into(arthur_bar)
            .add_into(arthur_bar_docs)
            .add_into(arthur_bar_files)
            .add_into(trillian);

        println!("{:?}", tree);
    }
}

impl Relational<PathBuf> for PathBuf {
    fn relation(&self, other: &PathBuf) -> Relation {
        if self == other {
            Relation::Equal
        } else if self.parent() == other.parent() {
            Relation::Siblings
        } else if self.starts_with(other) {
            Relation::Descendant
        } else if other.starts_with(self) {
            Relation::Ancestor
        } else {
            Relation::None
        }
    }
}

trait Relational<T: Relational<T> + Eq + Hash + Sized> {
    fn relation(&self, other: &T) -> Relation;
}
/// ```text
///         a           a > b (a parent of b)
///        / \
///       /   \
///      b     c        b ~= c (b and c are siblings
///     / \     \
///    /   \     \
///   d    e     f      d ~= e (d and e are siblings)
///
///                     e != f (e and f have no close relation)
/// ```
///
/// **Siblings**
///  - /home/arthur
///  - /home/trillian
///
/// **Ancestor / Descendant**
///  - /home
///  - /home/trillian
///
/// **Equal**
///  - /home
///  - /home
///
/// _Structure_
/// ```text
/// /home
///     /home/trillian
///         /home/trillian/docs
///         /home/trillian/audio
///         /home/trillian/secrets
///    /home/arthur
///        /home/arthur/files
/// ```
/// Insert `/home/arthur/docs`;
/// 1. `/home/arthur/foo/bar` vs `/home` => `[/home/arthur/foo/bar]` => `[Inheritance]`
/// 2. `/home/arthur/foo/bar` vs `[/home/arthur, /home/trillian]` => `[Inheritance, None]` (create `foo`)
/// 3. `/home/arthur/foo/bar` vs `[/home/arthur/foo]` => `[Inheritance]` (create `bar`)
///
#[derive(Debug)]
enum Relation {
    /// Two elements that are equal
    Equal,
    /// Two elements that are not equal but has the same parent
    Siblings,
    /// The first element is an ancestor to the second element
    Ancestor,
    /// The second element ia an ancestor to the first element
    Descendant,
    /// The elements have no relation to each other
    None,
}
