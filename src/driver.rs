use thomson::{collect_rules, parse_json, parse_toml, toml_to_json_by_rules};

use crate::context::Context;

pub struct Driver {
    pub ctx: Context,
}

impl Driver {
    pub fn new<A: ToString, B: ToString, C: ToString>(path: A, toml_path: B, json_path: C) -> Self {
        Self {
            ctx: Context::new(path, toml_path, json_path),
        }
    }

    pub fn run(&self) -> anyhow::Result<String> {
        std::env::set_current_dir(&self.ctx.path)?;
        let conf_json = parse_json(std::path::Path::new(&self.ctx.json_path))?;
        let conf_toml = parse_toml(std::path::Path::new(&self.ctx.toml_path))?;

        let rules_json = collect_rules(conf_json);
        // for rule in &rules_json {
        //     dbg!(format!("{}", &rule));
        // }
        let json_value = toml_to_json_by_rules(conf_toml, &rules_json);
        Ok(json_value.to_string())
    }
}
