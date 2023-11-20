mod dotfile;
mod nix;
pub mod secret;
mod vars;
use pathbuf::pathbuf;

use anyhow::anyhow;
use dotfile::{nix_conf::NixConf, Dotfile, merge_dotfiles};
use std::{
    collections::HashMap,
    env,
    path::{Path, PathBuf},
    str::FromStr,
};
use vars::UserEnvVars;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Target {
    PersonalNixCache,
}

fn all_targets() -> Vec<Target> {
    use Target::*;
    vec![PersonalNixCache]
}

#[derive(Debug)]
pub enum Region {
    Home,
    AWS { region: String },
}

#[derive(Debug)]
pub struct Config {
    region: Region,
    ssh_private_key_base64: String,
}

pub fn bootstrap(config: Config, targets: Vec<Target>) -> anyhow::Result<()> {
    let targets = if targets.is_empty() {
        all_targets()
    } else {
        targets
    };
    let home_dir: PathBuf = dirs::home_dir().ok_or(anyhow!("Could not find home directory"))?;

    let dotfiles = merge_dotfiles(vec![
        personal_nix_cache(&config)
    ])?;

    Ok(())
}

fn personal_nix_cache(config: &Config) -> HashMap<PathBuf, Box<dyn Dotfile>> {
    const MY_NIX_CACHE_PARAMS: &'static str =
        "compression=zstd&priority=0&trusted=true&want-mass-query=true";
    let nix_substituter: Option<String> = match &config.region {
        Region::Home => Some(format!(
            "s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&{MY_NIX_CACHE_PARAMS}"
        )),
        Region::AWS { region } => {
            if region == "us-east-2" {
                Some(format!("s3://kokobd-nix-cache-ohio?profile=default&region=us-east-2&{MY_NIX_CACHE_PARAMS}"))
            } else {
                None
            }
        }
    };
    let nix_substituters = match nix_substituter {
        Some(s) => vec![s],
        None => vec![],
    };
    let nix_conf: Box<dyn Dotfile> = Box::new(
        NixConf::new()
            .with_secret_key_files(vec!["/etc/nix/secret-key".to_string()])
            .with_post_build_hook("/etc/nix/post-build-hook".to_string())
            .with_substituters(nix_substituters)
            .with_trusted_public_keys(vec![
                "cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=".to_string(),
            ]),
    );
    HashMap::from([(pathbuf!["/etc", "nix", "nix.conf"], nix_conf)])
}
