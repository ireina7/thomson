use thomson::{collect_rules, parse_json, parse_toml, toml_to_json_by_rules};

pub struct Driver {
    pub json_path: String,
    pub toml_path: String,
    pub debugging: bool,
}

impl Driver {
    pub fn new<A: ToString, B: ToString>(toml_path: A, json_path: B) -> Self {
        Self {
            json_path: json_path.to_string(),
            toml_path: toml_path.to_string(),
            debugging: false,
        }
    }

    pub fn run(&self) -> anyhow::Result<String> {
        let conf_json = parse_json(std::path::Path::new(&self.json_path))?;
        let conf_toml = parse_toml(std::path::Path::new(&self.toml_path))?;

        let rules_json = collect_rules(conf_json);
        // for rule in &rules_json {
        //     dbg!(format!("{}", &rule));
        // }
        let json_value = toml_to_json_by_rules(conf_toml, &rules_json);
        Ok(json_value.to_string())
    }
}
