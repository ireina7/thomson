use std::{borrow::Cow, collections::HashMap};

use rule::Rules;
use serde_json as json;

use crate::rule;

/// The main logic to transform `TOML` value into `JSON` value by rules
pub fn transform_by_rules(toml_value: toml::Value, rules: &Rules) -> json::Value {
    // let kv = collect_toml_paths_by_rules(toml_value, rules);
    let kv = map_by_rules(&toml_value, rules);
    let kv = kv
        .into_iter()
        .map(|(k, v)| (k.build(), transform(v.clone())))
        .collect();
    toml_to_json_value(kv)
}

/// Directly transform `TOML` value into `JSON` value without rules.
fn transform(value: toml::Value) -> json::Value {
    match value {
        toml::Value::String(s) => json::Value::String(s),
        toml::Value::Integer(i) => json::Value::Number(json::Number::from_i128(i as i128).unwrap()),
        toml::Value::Float(n) => json::Value::Number(json::Number::from_f64(n).unwrap()),
        toml::Value::Boolean(b) => json::Value::Bool(b),
        toml::Value::Datetime(datetime) => json::Value::String(datetime.to_string()),
        toml::Value::Array(vec) => {
            let vs = vec.into_iter().map(|v| transform(v)).collect();
            json::Value::Array(vs)
        }
        toml::Value::Table(map) => {
            let mut kv = HashMap::new();
            for (k, v) in map {
                let v = transform(v);
                kv.insert(k, v);
            }
            json::Value::Object(kv.into_iter().collect())
        }
    }
}

/// Transform `TOML` paths by `JSON` rules.  
/// Keep leaf values.
pub fn map_by_rules<'v>(
    toml_value: &'v toml::Value,
    rules: &Rules,
) -> HashMap<rule::Path<'v>, &'v toml::Value> {
    let mut path = rule::Path::empty();
    let mut collector = HashMap::new();
    match_rule_dfs(&toml_value, rules.root(), false, &mut path, &mut collector);
    collector
}

fn match_rule_dfs<'v>(
    toml_value: &'v toml::Value,
    node: &rule::Node,
    missed: bool,
    path: &mut rule::Path<'v>,
    collector: &mut HashMap<rule::Path<'v>, &'v toml::Value>,
) {
    match toml_value {
        toml::Value::Table(map) => {
            for (k, v) in map {
                // we have missed before
                if missed {
                    path.flattern(); // cancel all previous adherences
                    path.push(Cow::Borrowed(k));
                    match_rule_dfs(v, node, true, path, collector);
                    path.pop();
                    continue;
                }

                // try to match
                if let Some(next) = node.get(k) {
                    match next.edge {
                        rule::Edge::Connected => {
                            path.adhere(Cow::Borrowed(k));
                        }
                        rule::Edge::Restarted => {
                            path.push(Cow::Borrowed(k));
                        }
                    }
                    match_rule_dfs(v, next, false, path, collector);
                } else {
                    path.push(Cow::Borrowed(k));
                    match_rule_dfs(v, node, true, path, collector);
                }
                path.pop();
            }
        }
        v => {
            collector.insert(path.clone(), v);
        }
    }
}

fn toml_to_json_map(path: &[String], v: json::Value) -> json::Map<String, json::Value> {
    if path.is_empty() {
        return vec![].into_iter().collect();
    }
    if path.len() == 1 {
        return vec![(path[0].clone(), v)].into_iter().collect();
    }

    let mut ans = json::Map::new();
    let inner = toml_to_json_map(&path[1..], v);
    ans.insert(path[0].clone(), json::Value::Object(inner));

    ans
}

fn insert_json_value(ans: &mut json::Map<String, json::Value>, k: String, v: json::Value) {
    if !ans.contains_key(&k) {
        ans.insert(k, v);
        return;
    }

    let av = ans.get_mut(&k).unwrap();
    match (av, v) {
        (serde_json::Value::Object(ref mut ans), serde_json::Value::Object(map)) => {
            for (k, v) in map {
                insert_json_value(ans, k, v);
            }
        }
        _ => unreachable!(),
    }
}

fn toml_to_json_value(kv: HashMap<Vec<String>, json::Value>) -> json::Value {
    // dbg!(&kv);
    let mut ans = json::Map::new();
    for (path, v) in kv {
        let line = toml_to_json_map(&path, v);
        for (k, v) in line {
            insert_json_value(&mut ans, k, v);
        }
    }
    // dbg!(&ans);
    json::Value::Object(ans)
}

#[cfg(test)]
mod test {
    use crate::{collect, io};
    use collect::collect_rules;

    #[test]
    fn test_collect_rules() -> anyhow::Result<()> {
        let conf = io::parse_json(std::path::Path::new("./examples/vscode/conf/settings.json"))?;
        dbg!(&conf);
        let rules = collect_rules(conf);
        for rule in rules.paths() {
            println!("{}", rule);
        }
        Ok(())
    }
}
