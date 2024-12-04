use clap::Parser;

use crate::{
    collect::collect_json_rules,
    context::Context,
    io::{parse_json, parse_toml},
    toml_to_json_by_rules,
};

pub struct Driver {
    pub ctx: Context,
}

impl Driver {
    pub fn new() -> Self {
        let args = Args::parse();
        let conf = args.toml.unwrap_or("settings.toml".to_owned());
        let rule = args.rule.unwrap_or("settings.json".to_owned());
        Self {
            ctx: Context::new(args.path, conf, rule, args.debugging),
        }
    }

    pub fn run(&self) -> anyhow::Result<String> {
        std::env::set_current_dir(&self.ctx.path)?;
        let conf_json = parse_json(std::path::Path::new(&self.ctx.json_path))?;
        let conf_toml = parse_toml(std::path::Path::new(&self.ctx.toml_path))?;

        let rules = collect_json_rules(conf_json);
        if self.ctx.debugging {
            let paths = rules.paths();
            for path in paths {
                println!("{}", path);
            }
        }

        // let rules_json = collect_rules(conf_json);
        // if self.ctx.debugging {
        //     for rule in &rules_json {
        //         let rule = format!("{}", &rule);
        //         dbg!(rule);
        //     }
        // }
        let json_value = toml_to_json_by_rules(conf_toml, &rules);
        Ok(json_value.to_string())
    }
}

#[derive(clap::Parser, Debug)]
#[command(version, about = "[Thomson]", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub toml: Option<String>,

    #[arg(short, long)]
    pub rule: Option<String>,

    #[arg(short, long, action)]
    pub debugging: bool,
}
