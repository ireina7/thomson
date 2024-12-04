#![allow(dead_code)]
use std::{
    borrow::{Borrow, Cow},
    collections::HashMap,
    fmt,
    hash::Hash,
};

use itertools::Itertools;

/// A single rule to combine paths  
/// `span.len() >= path.len()`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub path: Vec<String>,
    pub span: Vec<usize>,
}

impl Rule {
    pub fn empty() -> Self {
        Self {
            path: Vec::new(),
            span: Vec::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.path.len()
    }

    pub fn from_path(path: Vec<String>) -> Self {
        let len = path.len();
        Self {
            path,
            span: vec![0; len],
        }
    }

    pub fn group(&mut self, i: usize, j: usize) {
        if i > self.span.len() || j > self.span.len() || j < i {
            return;
        }
        self.span[i] = j - i;
    }

    pub fn extend(&mut self, path: Vec<String>) {
        let len = path.len();
        self.path.extend(path);
        self.span.extend(vec![0; len]);
    }

    pub fn push_front<S: ToString>(&mut self, s: S) {
        self.extend(vec![s.to_string()]);
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut ans: Vec<String> = vec![];
        let mut i = 0;
        while i < self.path.len() {
            if self.span[i] > 0 {
                let mut s = String::new();
                let j = i + self.span[i];
                while i < self.path.len() && i < j {
                    if !s.is_empty() {
                        s.push_str(".");
                    }
                    s.push_str(&self.path[i]);
                    i += 1;
                }
                ans.push(s);
                continue;
            }
            ans.push(self.path[i].clone());
            i += 1;
        }
        ans
    }

    /// Join two rules like joining two paths
    pub fn join(self, that: Rule) -> Rule {
        let offset = self.path.len();

        let mut rule = Self::from_path(self.path);
        rule.extend(that.path);

        for i in 0..self.span.len() {
            let j = self.span[i];
            if j == 0 {
                continue;
            }
            rule.group(i, j);
        }

        for i in 0..that.span.len() {
            let j = that.span[i];
            if j == 0 {
                continue;
            }
            rule.group(i + offset, j + offset);
        }
        rule
    }

    pub fn tail(&self) -> Rule {
        let mut rule = Self::from_path(self.path[1..].into_iter().map(|s| s.clone()).collect());
        for i in 1..self.path.len() {
            if self.span[i] > 0 {
                rule.group(i - 1, self.span[i] - 1);
            }
        }
        // TODO
        rule
    }
}

impl fmt::Display for Rule {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let ans = self.flatten();
        write!(f, "[{}]", ans.join(", "))
    }
}

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

    pub fn flattern(&mut self) {
        let ss = self.units();
        self.clear();
        for s in ss {
            self.push(s);
        }
    }

    pub fn build(&self) -> Vec<String> {
        let mut ans = Vec::new();
        for ss in &self.0 {
            let s: String = ss.iter().map(|s| s.to_owned()).join(".");
            ans.push(s);
        }
        ans
    }

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
        Self::collect_rules(&self.root, &mut collector, &mut path);
        collector
    }

    /// DFS
    fn collect_rules<'s>(node: &'s Node, collector: &mut Vec<Path<'s>>, path: &mut Path<'s>) {
        if node.is_leaf() {
            collector.push(path.clone());
            return;
        }
        for (key, next) in &node.nexts {
            match next.edge {
                Edge::Connected => path.adhere(Cow::Borrowed(key)),
                Edge::Restarted => path.push(Cow::Borrowed(key)),
            }
            Self::collect_rules(next, collector, path);
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

impl Edge {
    pub fn is_connected(&self) -> bool {
        matches!(self, Self::Connected)
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
    fn display_rule() {
        let path = vec!["a", "b", "c", "d", "e", "f", "g"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let mut rule = Rule::from_path(path);
        rule.group(1, 5);

        let path = vec!["a", "b", "c", "d", "e", "f", "g"]
            .into_iter()
            .map(|s| s.to_string())
            .collect();
        let mut that = Rule::from_path(path);
        that.group(1, 3);

        let rule = rule.join(that);

        dbg!(format!("{}", rule));
    }

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
