mod dotfile;
mod secret;
mod targets;
use base64::Engine;
use dotfile::apply_dotfiles;
use dotfile::{merge_dotfiles, Dotfile};
use secret::{AgeDecryptor, AgeIdentityParseError};
use std::{collections::HashMap, path::PathBuf};
use thiserror::Error;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, clap::ValueEnum)]
pub enum Target {
    PersonalNixCache,
    Git,
}

pub fn all_targets() -> Vec<Target> {
    use Target::*;
    vec![PersonalNixCache, Git]
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum Region {
    Home,
    AWS { region: String },
}

pub struct Config {
    region: Region,
    decryptor: AgeDecryptor,
    home_dir: PathBuf,
}

impl Config {
    pub fn new(region: Region, ssh_private_key_base64: String) -> Result<Self, InitConfigError> {
        let home_dir: PathBuf = dirs::home_dir().ok_or(InitConfigError::NoHomeDir)?;
        let ssh_private_key =
            base64::engine::general_purpose::STANDARD.decode(ssh_private_key_base64.as_bytes())?;
        Ok(Self {
            region,
            decryptor: AgeDecryptor::new(ssh_private_key)?,
            home_dir,
        })
    }
}

#[derive(Debug, Error)]
pub enum InitConfigError {
    #[error("Could not find home directory")]
    NoHomeDir,
    #[error("Could not parse ssh private key")]
    AgeIdentityParseError(#[from] AgeIdentityParseError),
    #[error("Provided ssh private key is not valid base64")]
    Base64DecodeError(#[from] base64::DecodeError),
}

#[derive(Debug, Error)]
#[error("Failed to bootstrap")]
pub enum BootStrapError {
    DecryptError(#[from] DecryptError),
}

pub fn bootstrap(config: Config, targets: Vec<Target>) -> anyhow::Result<()> {
    let dotfiles = merge_dotfiles({
        let mut dotfiles_vec = Vec::new();
        for target in targets {
            let dotfiles = target.bootstrap_dotfiles(&config)?;
            dotfiles_vec.push(dotfiles);
        }
        dotfiles_vec
    })?;
    let changed_files = apply_dotfiles(dotfiles)?;
    if changed_files.is_empty() {
        println!("no files changed");
    } else {
        println!("changed files: ");
        for file in changed_files {
            println!("- {}", file.display());
        }
    }

    Ok(())
}

impl Target {
    fn bootstrap_dotfiles(
        self,
        config: &Config,
    ) -> Result<HashMap<PathBuf, Box<dyn Dotfile>>, DecryptError> {
        match self {
            Target::PersonalNixCache => targets::personal_nix_cache::dotfiles(config),
            Target::Git => targets::git::dotfiles(config),
        }
    }
}

#[derive(Debug, Error)]
#[error("Failed to decrypt {path:?}: {error}")]
pub struct DecryptError {
    path: String,
    error: Box<dyn std::error::Error + Send + Sync>,
}
