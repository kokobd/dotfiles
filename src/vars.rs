use itertools::Itertools;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::path::PathBuf;
use std::io::Write;

pub struct UserEnvVars {
    vars: HashMap<String, String>,
    home: PathBuf,
}

impl UserEnvVars {
    pub fn new(home: PathBuf) -> Self {
        Self {
            vars: HashMap::new(),
            home,
        }
    }

    pub fn add(self, key: &str, value: String) -> Self {
        let mut vars = self.vars;
        vars.insert(String::from(key), value);
        Self {
            vars,
            home: self.home,
        }
    }

    pub fn apply(self) -> anyhow::Result<()> {
        let script_to_append = self
            .vars
            .into_iter()
            .map(|(k, v)| format!("export {}='{}'", k, v))
            .join("\n");
        let bashrc_path = {
            let mut path = self.home;
            path.push(".bashrc");
            path
        };
        let mut bashrc_file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(bashrc_path)?;
        bashrc_file.write_all(b"\n")?;
        bashrc_file.write_all(script_to_append.as_bytes())?;
        Ok(())
    }
}
