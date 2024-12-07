use std::{
    borrow::{Borrow, Cow},
    collections::{HashMap, VecDeque},
    fmt,
    hash::Hash,
};

use itertools::Itertools;

/// Representation of a key path, may be owned or not.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path<'s>(VecDeque<Vec<Cow<'s, Key>>>);

impl Default for Path<'_> {
    fn default() -> Self {
        Self::empty()
    }
}

impl fmt::Display for Path<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = self.0.iter().map(|ss| ss.into_iter().join(".")).join(", ");
        write!(f, "[{}]", s)
    }
}

impl<'s> IntoIterator for Path<'s> {
    type Item = Vec<Cow<'s, Key>>;

    type IntoIter = <VecDeque<Vec<Cow<'s, Key>>> as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[allow(dead_code)]
impl<'s> Path<'s> {
    pub fn empty() -> Self {
        Self(VecDeque::new())
    }

    pub fn from<Iter>(iter: Iter) -> Self
    where
        Iter: IntoIterator<Item = Vec<Cow<'s, Key>>>,
    {
        Self(iter.into_iter().collect())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn split_head(&mut self) -> Option<Path<'s>> {
        if self.is_empty() {
            return None;
        }
        let rest = self.0.split_off(1);
        Some(Path(rest))
    }

    pub(crate) fn push(&mut self, s: Cow<'s, Key>) {
        self.0.push_back(vec![s]);
    }

    pub(crate) fn adhere(&mut self, s: Cow<'s, Key>) {
        match self.0.back_mut() {
            None => self.push(s),
            Some(unit) => unit.push(s),
        }
    }

    pub(crate) fn link(&mut self, edge: Edge, s: Cow<'s, Key>) {
        match edge {
            Edge::Connected => {
                self.adhere(s);
            }
            Edge::Restarted => {
                self.push(s);
            }
        }
    }

    /// Pop only 1 unit
    pub(crate) fn pop(&mut self) -> Option<Cow<'s, Key>> {
        match self.0.back_mut() {
            Some(ss) => match ss.pop() {
                Some(s) => {
                    if ss.is_empty() {
                        self.0.pop_back();
                    }
                    Some(s)
                }
                None => {
                    self.0.pop_back();
                    self.pop()
                }
            },
            None => None,
        }
    }

    /// Flattern self
    pub fn flattern(&mut self) {
        let ss = self.units();
        self.clear();
        for s in ss {
            self.push(s);
        }
    }

    pub fn keys(self) -> impl Iterator<Item = Key> + use<'s> {
        self.0.into_iter().map(|part| {
            let x = if part.len() > 1 {
                Key::field(part.into_iter().map(|s| s.into_owned()).join("."))
            } else if part.is_empty() {
                unreachable!()
            } else {
                part.into_iter().next().unwrap().into_owned()
            };
            // dbg!(&x);
            x
        })
    }

    /// Build the path into `Vec<String>`, all borrows are forced to be *owned*.  
    /// If you want to build without consuming ownership, clone `self` before `build`.
    pub fn build(self) -> Vec<String> {
        let mut ans = Vec::new();
        for ss in self.0 {
            let s = ss.into_iter().map(|s| s.into_owned()).join(".");
            ans.push(s);
        }
        ans
    }

    /// Flattern and return new Vec
    pub fn units(&self) -> Vec<Cow<'s, Key>> {
        self.0.iter().flatten().map(|s| s.clone()).collect()
    }
}

/// Rule set as trie tree
#[derive(Debug, Clone)]
pub struct Rules {
    root: Box<Node>,
}

impl Default for Rules {
    fn default() -> Self {
        Self {
            root: Box::new(Node::link(Edge::Restarted)),
        }
    }
}

impl Rules {
    pub fn new() -> Self {
        Self::default()
    }

    pub(crate) fn root_mut(&mut self) -> &mut Node {
        &mut self.root
    }

    pub(crate) fn root(&self) -> &Node {
        &self.root
    }

    pub fn paths(&self) -> Vec<Path<'_>> {
        let mut collector = Vec::new();
        let mut path = Path::empty();
        Self::collect_paths(&self.root, &mut collector, &mut path);
        collector
    }

    /// DFS
    fn collect_paths<'s>(node: &'s Node, collector: &mut Vec<Path<'s>>, path: &mut Path<'s>) {
        if node.is_leaf() {
            collector.push(path.clone());
            return;
        }
        for (key, next) in &node.nexts {
            match next.edge {
                Edge::Connected => path.adhere(Cow::Borrowed(key)),
                Edge::Restarted => path.push(Cow::Borrowed(key)),
            }
            Self::collect_paths(next, collector, path);
            path.pop();
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Edge {
    Connected,
    Restarted,
}

impl Default for Edge {
    fn default() -> Self {
        Edge::Restarted
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Key {
    Field(String),
    Index { of: usize, total: usize },
}

impl Key {
    pub fn field<S: ToString>(name: S) -> Self {
        Self::Field(name.to_string())
    }

    pub fn index(of: usize, total: usize) -> Self {
        Self::Index { of, total }
    }

    pub fn pseudo_index() -> Self {
        Self::index(0, 0)
    }
}

impl Into<String> for Key {
    fn into(self) -> String {
        match self {
            Key::Field(s) => s,
            Key::Index { of, .. } => format!("{}", of),
        }
    }
}

impl fmt::Display for Key {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Key::Field(s) => write!(f, "{}", s),
            Key::Index { of, .. } => write!(f, "{}", of),
        }
    }
}

/// Trie tree node
/// TODO compact prefix path
#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub edge: Edge,
    nexts: HashMap<Key, Node>,
}

#[allow(dead_code)]
impl Node {
    pub(crate) fn link(edge: Edge) -> Self {
        Self {
            edge,
            nexts: HashMap::new(),
        }
    }

    pub(crate) fn insert(&mut self, edge: Edge, key: Key) -> Option<Node> {
        self.nexts.insert(key, Node::link(edge))
    }

    pub(crate) fn contains_key<S: ?Sized>(&self, s: &S) -> bool
    where
        Key: Borrow<S>,
        S: Hash + Eq,
    {
        self.nexts.contains_key(s)
    }

    pub(crate) fn next(&mut self, edge: Edge, key: Key) -> &mut Node {
        if self.contains_key(&key) {
            return self.get_mut(&key).unwrap(); // Sad that Rust currently cannot infer None case's lifetime ;(
        }

        self.insert(edge, key.clone());
        self.get_mut(&key).unwrap() // we know we can safely unwrap here
    }

    pub(crate) fn compact<Iter>(&mut self, path: Iter) -> &mut Node
    where
        Iter: IntoIterator<Item = (Edge, Key)>,
    {
        let mut this = self;
        for (edge, s) in path {
            this = this.next(edge, s);
        }
        this
    }

    pub(crate) fn get<S: ?Sized>(&self, s: &S) -> Option<&Node>
    where
        Key: Borrow<S>,
        S: Hash + Eq,
    {
        self.nexts.get(s)
    }

    pub(crate) fn get_mut<S: ?Sized>(&mut self, s: &S) -> Option<&mut Node>
    where
        Key: Borrow<S>,
        S: Hash + Eq,
    {
        self.nexts.get_mut(s)
    }

    pub(crate) fn is_leaf(&self) -> bool {
        self.nexts.is_empty()
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_rules_set() {
        let mut rules = Rules::new();
        rules.root.insert(Edge::Restarted, Key::field("editor"));
        rules.root.insert(Edge::Restarted, Key::field("window"));

        let editor = rules.root.get_mut(&Key::field("editor")).unwrap();
        editor.insert(Edge::Connected, Key::field("font"));
        editor.insert(Edge::Connected, Key::field("size"));
        editor.insert(Edge::Restarted, Key::field("restarted"));

        let paths = rules.paths();
        for path in paths {
            dbg!(path.build());
        }
    }
}
