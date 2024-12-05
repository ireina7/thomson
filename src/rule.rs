use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fmt,
    hash::Hash,
};

use itertools::Itertools;

/// Representation of a key path, may be owned or not.
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path<'s>(Vec<Vec<Cow<'s, str>>>);

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

#[allow(dead_code)]
impl<'s> Path<'s> {
    pub fn empty() -> Self {
        Self(Vec::new())
    }

    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub(crate) fn push(&mut self, s: Cow<'s, str>) {
        self.0.push(vec![s]);
    }

    pub(crate) fn adhere(&mut self, s: Cow<'s, str>) {
        match self.0.last_mut() {
            None => self.push(s),
            Some(unit) => unit.push(s),
        }
    }

    /// Pop only 1 unit
    pub(crate) fn pop(&mut self) -> Option<Cow<'s, str>> {
        match self.0.last_mut() {
            Some(ss) => match ss.pop() {
                Some(s) => {
                    if ss.is_empty() {
                        self.0.pop();
                    }
                    Some(s)
                }
                None => {
                    self.0.pop();
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
    pub fn units(&self) -> Vec<Cow<'s, str>> {
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

/// Trie tree node
/// TODO compact prefix path
#[derive(Debug, Clone)]
pub(crate) struct Node {
    pub edge: Edge,
    nexts: HashMap<String, Node>,
}

#[allow(dead_code)]
impl Node {
    pub(crate) fn link(edge: Edge) -> Self {
        Self {
            edge,
            nexts: HashMap::new(),
        }
    }

    pub(crate) fn insert<S: ?Sized + ToString>(&mut self, edge: Edge, key: &S) -> Option<Node> {
        self.nexts.insert(key.to_string(), Node::link(edge))
    }

    pub(crate) fn contains_key<S: ?Sized>(&self, s: &S) -> bool
    where
        String: Borrow<S>,
        S: Hash + Eq,
    {
        self.nexts.contains_key(s)
    }

    pub(crate) fn next<S: ToString>(&mut self, edge: Edge, key: S) -> &mut Node
    where
        String: Borrow<S>,
        S: Hash + Eq,
    {
        if self.contains_key(&key) {
            return self.get_mut(&key).unwrap(); // Sad that Rust currently cannot infer None case's lifetime ;(
        }

        self.insert(edge, &key);
        self.get_mut(&key).unwrap() // we know we can safely unwrap here
    }

    pub(crate) fn compact<S, Iter>(&mut self, path: Iter) -> &mut Node
    where
        S: ToString + Hash + Eq,
        String: Borrow<S>,
        Iter: IntoIterator<Item = (Edge, S)>,
    {
        let mut this = self;
        for (edge, s) in path {
            this = this.next(edge, s);
        }
        this
    }

    pub(crate) fn get<S: ?Sized>(&self, s: &S) -> Option<&Node>
    where
        String: Borrow<S>,
        S: Hash + Eq,
    {
        self.nexts.get(s)
    }

    pub(crate) fn get_mut<S: ?Sized>(&mut self, s: &S) -> Option<&mut Node>
    where
        String: Borrow<S>,
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
        rules.root.insert(Edge::Restarted, "editor");
        rules.root.insert(Edge::Restarted, "window");

        let editor = rules.root.get_mut("editor").unwrap();
        editor.insert(Edge::Connected, "font");
        editor.insert(Edge::Connected, "size");
        editor.insert(Edge::Restarted, "restarted");

        let paths = rules.paths();
        for path in paths {
            dbg!(path.build());
        }
    }
}
