/// Dependency injection
#[allow(dead_code)]
pub struct Context {
    pub path: String,
    pub json_path: String,
    pub toml_path: String,
    pub debugging: bool,
    pub listen: bool,
}

impl Context {
    pub fn new<A: ToString, B: ToString, C: ToString>(
        path: A,
        toml_path: B,
        json_path: C,
        debugging: bool,
        listen: bool,
    ) -> Self {
        Self {
            path: path.to_string(),
            json_path: json_path.to_string(),
            toml_path: toml_path.to_string(),
            debugging,
            listen,
        }
    }
}
