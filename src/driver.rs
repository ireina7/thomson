use std::ops;

use clap::Parser;

use crate::{
    collect::collect_rules,
    context::Context,
    io::{parse_json, parse_toml},
    transform::transform_by_rules,
};

/// The main entry
pub struct Driver {
    pub ctx: Context,
}

/// Pretending we have basic dependency injection...
impl ops::Deref for Driver {
    type Target = Context;

    fn deref(&self) -> &Self::Target {
        &self.ctx
    }
}

impl Driver {
    pub fn new() -> Self {
        let args = Args::parse();
        let conf = args.toml.unwrap_or("settings.toml".to_owned());
        let rule = args.rule.unwrap_or("settings.json".to_owned());
        Self {
            ctx: Context::new(args.path, conf, rule, args.debugging, args.listen),
        }
    }

    /// Do the job!
    pub fn run(&self) -> anyhow::Result<String> {
        std::env::set_current_dir(&self.path)?;
        let json_value = parse_json(std::path::Path::new(&self.json_path))?;
        let toml_value = parse_toml(std::path::Path::new(&self.toml_path))?;

        let rules = collect_rules(json_value);
        if self.debugging {
            let paths = rules.paths();
            for path in paths {
                log::debug!("Path: {}", path);
            }
        }
        let ans = transform_by_rules(toml_value, &rules);
        Ok(ans.to_string())
    }
}

/// Parser of command line
#[derive(clap::Parser, Debug)]
#[command(version, about = "Thomson", long_about = None)]
pub struct Args {
    #[arg(short, long)]
    pub path: String,

    #[arg(short, long)]
    pub toml: Option<String>,

    #[arg(short, long)]
    pub rule: Option<String>,

    #[arg(short, long, action)]
    pub debugging: bool,

    #[arg(short, long, action)]
    pub listen: bool,
}
