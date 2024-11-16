use std::fmt;

/// A single rule to combine paths  
/// `span.len() >= path.len()`
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct Rule {
    pub path: Vec<String>,
    pub span: Vec<usize>,
}

impl Rule {
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
        if i > self.span.len() || j > self.span.len() {
            return;
        }
        self.span[i] = j;
    }

    pub fn extend(&mut self, path: Vec<String>) {
        let len = path.len();
        self.path.extend(path);
        self.span.extend(vec![0; len]);
    }

    pub fn flatten(&self) -> Vec<String> {
        let mut ans: Vec<String> = vec![];
        let mut i = 0;
        while i < self.path.len() {
            if self.span[i] > 0 {
                let mut s = String::new();
                let j = self.span[i];
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
}
