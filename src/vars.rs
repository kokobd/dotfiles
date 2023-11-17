use itertools::Itertools;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::PathBuf;

pub struct UserEnvVars {
    vars: HashMap<String, String>,
    home: PathBuf,
}

impl UserEnvVars {
    pub fn add(self, key: String, value: String) -> Self {
        let mut vars = self.vars;
        vars.insert(key, value);
        Self { vars, home: self.home }
    }

    pub fn apply(self) -> anyhow::Result<()> {
        let x = self
            .vars
            .into_iter()
            .map(|(k, v)| format!("export {}='{}'", k, v))
            .join("\n");
        let bashrc_path = {
            let mut path = self.home;
            path.push(".bashrc");
            path
        };
        let mut file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(bashrc_path);
        Ok(())
    }
}
