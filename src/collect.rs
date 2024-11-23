use crate::rule::Rule;
use serde_json as json;

pub fn collect_rules(conf: json::Value) -> Vec<Rule> {
    match conf {
        serde_json::Value::Object(map) => collect_rules_from_map(map),
        _ => vec![],
    }
}

fn collect_rules_from_map(map: json::Map<String, json::Value>) -> Vec<Rule> {
    let mut rules = vec![];
    for (k, v) in map {
        let ks: Vec<_> = k.split(".").map(|s| s.to_string()).collect();
        let len = ks.len();
        let mut rule = Rule::from_path(ks);
        rule.group(0, len);

        let inners = collect_rules(v);
        if inners.is_empty() {
            rules.push(rule);
            continue;
        }
        for inner in inners {
            let r = rule.clone().join(inner);
            rules.push(r);
        }
    }
    rules
}
