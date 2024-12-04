use crate::rule::{self, Edge, Rules};
use serde_json as json;

// pub fn collect_rules(conf: json::Value) -> Vec<Rule> {
//     match conf {
//         serde_json::Value::Object(map) => collect_rules_from_map(map),
//         _ => vec![],
//     }
// }

// fn collect_rules_from_map(map: json::Map<String, json::Value>) -> Vec<Rule> {
//     let mut rules = vec![];
//     for (k, v) in map {
//         let ks: Vec<_> = k.split(".").map(|s| s.to_string()).collect();
//         let len = ks.len();
//         let mut rule = Rule::from_path(ks);
//         rule.group(0, len);

//         let inners = collect_rules(v);
//         if inners.is_empty() {
//             rules.push(rule);
//             continue;
//         }
//         for inner in inners {
//             let r = rule.clone().join(inner);
//             rules.push(r);
//         }
//     }
//     rules
// }

pub fn collect_json_rules(json_value: json::Value) -> Rules {
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
