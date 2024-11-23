mod context;
pub mod driver;
mod io;
mod rule;

use std::collections::HashMap;

use rule::Rule;
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

fn toml_to_json(value: toml::Value) -> json::Value {
    match value {
        toml::Value::String(s) => json::Value::String(s),
        toml::Value::Integer(i) => json::Value::Number(json::Number::from_i128(i as i128).unwrap()),
        toml::Value::Float(n) => json::Value::Number(json::Number::from_f64(n).unwrap()),
        toml::Value::Boolean(b) => json::Value::Bool(b),
        toml::Value::Datetime(datetime) => json::Value::String(datetime.to_string()),
        toml::Value::Array(vec) => {
            let vs = vec.into_iter().map(|v| toml_to_json(v)).collect();
            json::Value::Array(vs)
        }
        toml::Value::Table(map) => {
            let mut kv = HashMap::new();
            for (k, v) in map {
                let v = toml_to_json(v);
                kv.insert(k, v);
            }
            json::Value::Object(kv.into_iter().collect())
        }
    }
}

#[allow(dead_code)]
fn collect_toml_paths(
    prefix: Vec<String>,
    value: toml::Value,
    collector: &mut HashMap<Vec<String>, json::Value>,
) {
    match value {
        toml::Value::String(s) => {
            collector.insert(prefix, json::Value::String(s));
        }
        toml::Value::Integer(i) => {
            collector.insert(
                prefix,
                json::Value::Number(json::Number::from_i128(i as i128).unwrap()),
            );
        }
        toml::Value::Float(n) => {
            collector.insert(
                prefix,
                json::Value::Number(json::Number::from_f64(n).unwrap()),
            );
        }
        toml::Value::Boolean(b) => {
            collector.insert(prefix, json::Value::Bool(b));
        }
        toml::Value::Datetime(datetime) => {
            collector.insert(prefix, json::Value::String(datetime.to_string()));
        }
        toml::Value::Array(_) => {
            collector.insert(prefix, toml_to_json(value));
        }
        toml::Value::Table(map) => {
            for (k, v) in map {
                let mut prefix = prefix.clone();
                prefix.push(k);
                collect_toml_paths(prefix, v, collector);
            }
        }
    }
}

fn transform_path(
    mut collector: HashMap<Vec<String>, json::Value>,
    json_rules: &[Rule],
) -> HashMap<Vec<String>, json::Value> {
    let mut ans = HashMap::new();
    // dbg!(&collector);
    for rule in json_rules {
        let path = &rule.path;
        // dbg!(&path, collector.get(path));
        if let Some(v) = collector.get(path) {
            ans.insert(rule.flatten(), v.clone());
            // dbg!("insert", rule.flatten(), &v);
            collector.remove(path);
        }
    }

    // Found no corresponding rules, output as nested path
    for (path, v) in collector {
        ans.insert(path, v);
    }
    // dbg!(&ans);
    ans
}

fn collect_toml_paths_by_rules(
    toml_value: toml::Value,
    json_rules: &[Rule],
) -> HashMap<Vec<String>, json::Value> {
    let mut collector = HashMap::new();
    collect_toml_paths(vec![], toml_value, &mut collector);

    transform_path(collector, json_rules)
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

pub fn toml_to_json_by_rules(toml_value: toml::Value, rules: &[Rule]) -> json::Value {
    let kv = collect_toml_paths_by_rules(toml_value, rules);
    toml_to_json_value(kv)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_collect_rules() -> anyhow::Result<()> {
        let conf = io::parse_json(std::path::Path::new("./settings.json"))?;
        dbg!(&conf);
        let rules = collect_rules(conf);
        for rule in rules {
            println!("{}", rule);
        }
        Ok(())
    }

    #[test]
    fn test_toml_kv() -> anyhow::Result<()> {
        let conf = io::parse_toml(std::path::Path::new("./Cargo.toml"))?;
        dbg!(&conf);
        let mut collector = HashMap::new();
        collect_toml_paths(vec![], conf, &mut collector);
        for (k, v) in collector {
            println!("{:?}: {:?}", k, v);
        }
        Ok(())
    }

    #[test]
    fn test_toml_rules() -> anyhow::Result<()> {
        let conf_json = io::parse_json(std::path::Path::new("./conf/settings.json"))?;
        let conf_toml = io::parse_toml(std::path::Path::new("./conf/settings.toml"))?;

        let rules_json = collect_rules(conf_json);

        let rules_toml = collect_toml_paths_by_rules(conf_toml, &rules_json);
        dbg!(rules_toml);
        Ok(())
    }
}
