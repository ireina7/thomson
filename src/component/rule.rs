use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fmt,
    hash::Hash,
};

use super::path::Path;

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
