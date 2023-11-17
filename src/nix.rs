use anyhow::{bail, Result};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::os::unix::prelude::PermissionsExt;
use std::path::Path;

#[derive(PartialEq, Eq, Debug)]
pub struct Configuration(HashMap<String, String>);

const NIX_CONF_PATH: &'static str = "/etc/nix/nix.conf";

const KEY_TRUSTED_PUBLIC_KEYS: &'static str = "trusted-public-keys";
const KEY_SUBSTITUTERS: &'static str = "substituters";

const KEY_POST_BUILD_HOOK: &'static str = "post-build-hook";
const KEY_SECRET_FILES: &'static str = "secret-key-files";

impl Configuration {
    pub fn read() -> Result<Configuration> {
        let nix_conf_exists = Path::new(NIX_CONF_PATH).exists();
        let file_content = if !nix_conf_exists {
            String::new()
        } else {
            fs::read_to_string(NIX_CONF_PATH)?
        };
        Configuration::parse(&file_content)
    }

    fn parse(file_content: &str) -> Result<Configuration> {
        let mut map = HashMap::new();
        for line in file_content.lines() {
            let line = line.trim();
            if line.starts_with("#") || line.is_empty() {
                continue;
            }
            let parts: Vec<&str> = line.splitn(2, '=').collect();
            if parts.len() != 2 {
                bail!("No '=' in line: {line}");
            }
            let key = parts[0].trim();
            let value = parts[1].trim();
            map.insert(key.to_string(), value.to_string());
        }
        Ok(Configuration(map))
    }

    pub fn write(&self) -> Result<()> {
        fs::write(NIX_CONF_PATH, self.render())?;
        Ok(())
    }

    fn render(&self) -> String {
        self.0
            .iter()
            .map(|(k, v)| format!("{} = {}", k, v))
            .collect::<Vec<String>>()
            .join("\n")
    }

    pub fn add_cache(&mut self, public_key: &str, substituter: &str) -> Result<()> {
        self.append_to_list(KEY_TRUSTED_PUBLIC_KEYS, public_key);
        self.append_to_list(KEY_SUBSTITUTERS, substituter);
        self.replace_value(KEY_POST_BUILD_HOOK, "/etc/nix/post-build-hook");
        fs::write(
            "/etc/nix/post-build-hook",
            format!(
                r#"#!/usr/bin/env bash
set -eu
set -f # disable globbing
export IFS=' '

echo "Signing paths" $OUT_PATHS
nix store sign --recursive --key-file /etc/nix/secret-key $OUT_PATHS
echo "Uploading paths" $OUT_PATHS
exec nix copy --to '{}' $OUT_PATHS
        "#,
                substituter
            ),
        )?;
        fs::set_permissions("/etc/nix/post-build-hook", PermissionsExt::from_mode(0o777))?;
        self.append_to_list(KEY_SECRET_FILES, "/etc/nix/secret-key");
        Ok(())
    }

    fn append_to_list(&mut self, conf_key: &str, value: &str) {
        let mut existing_entries: HashSet<String> = self
            .0
            .remove(conf_key)
            .unwrap_or(String::new())
            .split(" ")
            .filter(|&s| !s.trim().is_empty())
            .map(String::from)
            .collect();
        existing_entries.insert(String::from(value));
        let new_value = itertools::join(existing_entries.iter(), " ");
        self.0.insert(conf_key.to_string(), new_value);
    }

    fn replace_value(&mut self, conf_key: &str, value: &str) {
        self.0.insert(conf_key.to_string(), value.to_string());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_nix_conf() {
        assert_eq!(
            Configuration::parse("sandbox = false\nmax-jobs=auto\n").unwrap(),
            Configuration(
                [
                    ("sandbox".to_string(), "false".to_string()),
                    ("max-jobs".to_string(), "auto".to_string())
                ]
                .iter()
                .cloned()
                .collect()
            )
        );
        assert_eq!(
            Configuration::parse("").unwrap(),
            Configuration([].iter().cloned().collect())
        );
    }
}
