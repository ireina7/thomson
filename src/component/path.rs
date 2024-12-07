use crate::component::rule::{Edge, Key};
use itertools::Itertools;
use std::{borrow::Cow, collections::VecDeque, fmt, hash::Hash};

type Unit<'s> = Vec<Cow<'s, Key>>;

/// Representation of a key path, may be owned or not.  
/// If a unit is an *Index*, one should ensure that the index is the **ONLY** key inside Unit's Vec
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Path<'s>(VecDeque<Unit<'s>>);

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
    type Item = Unit<'s>;
    type IntoIter = <VecDeque<Unit<'s>> as IntoIterator>::IntoIter;

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

    /// Get real keys
    pub fn keys(self) -> impl Iterator<Item = Key> + use<'s> {
        self.0.into_iter().filter(|s| !s.is_empty()).map(|part| {
            if part.len() > 1 {
                Key::field(part.into_iter().map(|s| s.into_owned()).join("."))
            } else {
                part.into_iter().next().unwrap().into_owned()
            }
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
