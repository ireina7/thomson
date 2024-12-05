use std::str::FromStr;
use thiserror::Error;

use serde_json::{self as json};

/// Format error
#[derive(Error, Debug)]
pub enum FmtErr<F: FromStr>
where
    F::Err: std::fmt::Debug + std::error::Error + 'static,
{
    #[error("Reading IO error: {0:?}")]
    ReadErr(#[from] std::io::Error),

    #[error("Parsing error: {0:?}")]
    ParseErr(F::Err), // `F::Err` may be Self again, therefore we cannot directly `#[from]` here :(
}

pub type FmtResult<Fmt> = Result<Fmt, FmtErr<Fmt>>;

pub(crate) fn parse<Format: FromStr>(path: &std::path::Path) -> FmtResult<Format>
where
    Format::Err: std::error::Error,
{
    let file = std::fs::read_to_string(path)?;
    let conf: Format = file.parse().map_err(|err| FmtErr::ParseErr(err))?;
    Ok(conf)
}

/// Parse Toml file into [`toml::Value`] whose `Table` is a `BTreeMap<String, toml::Value>`
pub fn parse_toml(path: &std::path::Path) -> FmtResult<toml::Value> {
    let mut tv = parse(path)?;
    if let toml::Value::Table(ref mut table) = &mut tv {
        let includes = if table.contains_key("include") {
            table.remove("include")
        } else {
            None
        };

        if let Some(toml::Value::Array(includes)) = includes {
            for module in includes {
                if let Some(path) = module.as_str() {
                    let path = format!("{}.toml", path);
                    // dbg!(&path);
                    let inner = parse_toml(&std::path::Path::new(&path))?;
                    if let toml::Value::Table(t) = inner {
                        for (k, v) in t {
                            insert_toml_value(table, k, v);
                        }
                    }
                }
            }
        }
    }
    Ok(tv)
}

fn insert_toml_value(ans: &mut toml::map::Map<String, toml::Value>, k: String, v: toml::Value) {
    if !ans.contains_key(&k) {
        ans.insert(k, v);
        return;
    }

    let av = ans.get_mut(&k).unwrap();
    match (av, v) {
        (toml::Value::Table(ref mut ans), toml::Value::Table(map)) => {
            for (k, v) in map {
                insert_toml_value(ans, k, v);
            }
        }
        _ => unreachable!(),
    }
}

/// Parse Json file into [`json::Value`] whose `Object` is a `Map<String, json::Value>`
pub fn parse_json(path: &std::path::Path) -> FmtResult<json::Value> {
    parse(path)
}
