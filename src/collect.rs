use crate::rule::{self, Edge, Rules};
use serde_json as json;

/// Collect `JSON` format rules.  
/// returns a trie tree as rule set.
pub fn collect_rules(json_value: json::Value) -> Rules {
    let mut rules = Rules::new();
    collect_dfs(json_value, rules.root_mut());
    rules
}

fn collect_dfs(json_value: json::Value, node: &mut rule::Node) {
    match json_value {
        serde_json::Value::Object(map) => {
            for (s, v) in map {
                let ss: Vec<_> = s.split(".").map(|s| s.to_string()).collect();
                let path = ss.into_iter().enumerate().map(|(i, s)| {
                    (
                        if i == 0 {
                            Edge::Restarted
                        } else {
                            Edge::Connected
                        },
                        s,
                    )
                });
                let next = node.compact(path);
                collect_dfs(v, next);
            }
        }
        serde_json::Value::Array(_) => {}
        _ => {}
    }
}
