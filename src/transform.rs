use std::{borrow::Cow, collections::HashMap, mem};

use rule::Rules;
use serde_json as json;

use crate::component::{
    path::Path,
    rule::{self, Key},
};

/// The main logic to transform `TOML` value into `JSON` value by rules
pub fn transform_by_rules(toml_value: toml::Value, rules: &Rules) -> json::Value {
    let kv: HashMap<_, _> = map_by_rules(toml_value, rules)
        .into_iter()
        .map(|(k, v)| (k, transform(v)))
        .collect();

    // TODO: add context to debug
    // for (k, _) in &kv {
    //     log::debug!("Transformed path: {}", format!("{}", k));
    // }

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
pub fn map_by_rules<'v>(toml_value: toml::Value, rules: &Rules) -> HashMap<Path<'v>, toml::Value> {
    let mut path = Path::empty();
    let mut collector = HashMap::new();
    match_rule_dfs(toml_value, rules.root(), &mut path, &mut collector);
    collector
}

fn match_rule_dfs<'v>(
    toml_value: toml::Value,
    node: &rule::Node,
    path: &mut Path<'v>,
    collector: &mut HashMap<Path<'v>, toml::Value>,
) {
    match toml_value {
        toml::Value::Table(map) => {
            for (k, v) in map {
                // try to match
                let key = Key::field(k);
                if let Some(next) = node.get(&key) {
                    path.link(next.edge, Cow::Owned(key));
                    match_rule_dfs(v, next, path, collector);
                } else {
                    path.flattern(); // cancel all previous adherences
                    path.push(Cow::Owned(key));
                    collector.insert(path.clone(), v);
                }
                path.pop();
            }
        }
        toml::Value::Array(vs) => {
            let len = vs.len();
            for (i, v) in vs.into_iter().enumerate() {
                let key = Key::index(i, len).unwrap();
                if let Some(next) = node.get(&Key::pseudo_index()) {
                    path.link(next.edge, Cow::Owned(key));
                    match_rule_dfs(v, next, path, collector);
                } else {
                    path.flattern(); // cancel all previous adherences
                    path.push(Cow::Owned(key));
                    collector.insert(path.clone(), v);
                }
                path.pop();
            }
        }
        v => {
            collector.insert(path.clone(), v);
        }
    }
}

fn insert_json_array(vec: &mut Vec<json::Value>, k: Key, v: json::Value) {
    match k {
        Key::Field(_) => panic!(""),
        Key::Index { of, total } => {
            if vec.len() < total {
                panic!("");
            }

            match (&mut vec[of], v) {
                (json::Value::Null, v) => vec[of] = v,
                (slot, json::Value::Object(map)) => {
                    for (k, v) in map {
                        insert_json_value(slot, Key::field(k), v);
                    }
                }
                (_, json::Value::Null) => {
                    // we have inserted a null value
                    return;
                }
                _ => {
                    unreachable!()
                }
            }
        }
    }
}

fn insert_json_map(map: &mut json::Map<String, json::Value>, k: Key, v: json::Value) {
    match k {
        Key::Field(k) => {
            if !map.contains_key(&k) {
                map.insert(k, v);
                return;
            }

            let av = map.get_mut(&k).unwrap();
            match (av, v) {
                (av, serde_json::Value::Object(map)) => {
                    for (k, v) in map {
                        insert_json_value(av, Key::field(k), v);
                    }
                }
                (_, json::Value::Null) => {
                    // we have inserted a null value
                    return;
                }
                _ => {
                    unreachable!()
                }
            }
        }
        Key::Index { .. } => unreachable!(),
    }
}

fn replace_json_value(slot: &mut json::Value, k: Key, v: json::Value) {
    match k {
        Key::Field(k) => {
            let v = json::Value::Object(json::Map::from_iter([(k.into(), v)]));
            let _ = mem::replace(slot, v);
        }
        Key::Index { of, total } => {
            if of >= total {
                panic!("of >= total");
            }
            let mut vs = vec![json::Value::Null; total];
            vs[of] = v;
            let v = json::Value::Array(vs);
            let _ = mem::replace(slot, v);
        }
    }
}

/// Never touch existed values.  
///
/// # PANIC
/// If conflicts.
fn insert_json_value(ans: &mut json::Value, k: Key, v: json::Value) {
    match ans {
        serde_json::Value::Array(vec) => insert_json_array(vec, k, v),
        serde_json::Value::Object(map) => insert_json_map(map, k, v),
        serde_json::Value::Null => replace_json_value(ans, k, v),
        _ => unreachable!(),
    }
}

fn toml_to_json_value(kv: HashMap<Path<'_>, json::Value>) -> json::Value {
    let mut ans = json::Value::Null;
    for (path, v) in kv {
        let mut cur = &mut ans;
        for key in path.keys() {
            insert_json_value(cur, key.clone(), json::Value::Null);
            cur = match key {
                Key::Field(s) => cur.get_mut(s).unwrap(),
                Key::Index { of, .. } => cur.get_mut(of).unwrap(),
            }
        }
        let _ = mem::replace(cur, v);
    }
    ans
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
