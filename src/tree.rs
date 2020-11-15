use itertools::Itertools;
use std::borrow::{Borrow, BorrowMut};
use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::Hash;
use std::ops::Deref;

#[derive(Debug)]
struct Tree<T: Eq + Ord + Hash + Sized> {
    nodes: Vec<Node<T>>,
}

impl<T: Eq + Ord + Hash + Sized> Tree<T> {
    pub fn from(value: T, initial_capacity: usize) -> Tree<T> {
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

    pub fn add_into(self, value: T) -> Tree<T> {
        let index: usize = self.next_index(&value);
        self.add_into_index(value, index)
    }

    fn next_index(&self, value: &T) -> usize {
        let node: Option<usize> = self
            .nodes
            .iter()
            .enumerate()
            .rev()
            .filter(
                |(i, node): &(usize, &Node<T>)| match node.value.cmp(&value) {
                    Ordering::Greater => true,
                    Ordering::Equal => true,
                    Ordering::Less => false,
                },
            )
            .map(|(i, _)| i)
            .next();

        match node {
            Some(i) => i,
            None => self.nodes.len(),
        }
    }

    fn add_into_index(self, value: T, parent: Option<usize>, index: usize) -> Tree<T> {
        match self.nodes.get_mut(index) {
            Some(n) => {

                self.add_into_index(self, n.)
            },
            None => {
                let node = Node {
                    value,
                    parent,
                    children: None,
                };
                self.nodes.push(node);
                self
            }
        }
    }
}

#[derive(Debug)]
struct Node<T: Eq + Ord + Hash + Sized> {
    value: T,
    parent: Option<usize>,
    children: Option<Vec<usize>>,
}

impl<T: Eq + Ord + Hash + Sized> Node<T> {
    pub fn add_child(&mut self, index: usize) {
        match self.children {
            Some(children) => children.push(index),
            None => {
                let mut children = Vec::with_capacity(2);
                children.push(index);
                self.children = Some(children)
            }
        }
    }
}

//
// //struct Tree<T: Relational<T> + Eq + Hash + Sized> {
// struct Tree<T: Eq + Ord + Hash + Sized> {
//     value: T,
//     tree: Vec<Box<Tree<T>>>,
// }
//
// //impl<T: Relational<T> + Eq + Hash + Sized> Tree<T> {
// impl<T: Eq + Ord + Hash + Sized> Tree<T> {
//     pub fn from(value: T) -> Tree<T> {
//         Tree {
//             value,
//             tree: Vec::new(),
//         }
//     }
//
//     pub fn add(mut self, value: T) -> Tree<T> {
//         match self.value.cmp(&value) {
//             Ordering::Equal => self,
//             Ordering::Less => {
//                 let mut tree = Vec::new();
//                 tree.push(Box::new(self));
//                 Tree { value, tree }
//             }
//             Ordering::Greater => {
//                 let ancestor: Option<&Box<Tree<T>>> = self
//                     .tree
//                     .iter()
//                     .find(|x| value.cmp(&x.value) == Ordering::Less);
//                 match ancestor {
//                     Some(ancestor) => ancestor.add(value),
//                     None => {
//                         let tree = Tree::from(value);
//                         self.tree.push(Box::new(tree));
//                         self
//                     }
//                 }
//             }
//         }
//     }

// fn add(mut self, value: T) -> Tree<T> {
//     match self.value.relation(value) {
//         Relation::Equal(_, _) => self,
//         Relation::Siblings(_, _) => panic!("Should not have sinblings here"),
//         Relation::Inheritance { parent, child } => {
//             if parent == self {
//                 self.children.iter().map(|x| x.relation(child)).filter(|x| x.)
//             }
//         },
//     }
// }
//}

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

trait Relational<T: Relational<T> + Eq + Hash + Sized> {
    fn relation(&self, other: T) -> Relation<T>;
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
enum Relation<T: Eq> {
    /// Two elements that are equal
    Equal(T, T),
    /// Two elements that are not equal but has the same parent
    Siblings(T, T),
    /// One element is an ancestor to another element
    Inheritance { ancestor: T, descendant: T },
    /// The elements have no relation to each other
    None(T, T),
}
