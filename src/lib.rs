mod dotfile;
mod nix;
pub mod secret;
mod vars;

use anyhow::anyhow;
use std::{env, path::Path};
use vars::UserEnvVars;

pub fn bootstrap() -> anyhow::Result<()> {
    let coder_region: String = env::var("CODER_REGION").unwrap_or(String::new());
    let home_dir = dirs::home_dir().ok_or(anyhow!("Could not find home directory"))?;
    let mut user_env_vars = UserEnvVars::new(home_dir.clone());

    if Path::new("/etc/nix").exists() {
        const MY_NIX_CACHE_PARAMS: &'static str =
            "compression=zstd&priority=0&trusted=true&want-mass-query=true";
        let nix_substituter: Option<String> = match coder_region.as_str() {
            "home" => Some(format!(
            "s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&{MY_NIX_CACHE_PARAMS}"
        )),
            "aws-us-east-2" => Some(format!(
                "s3://kokobd-nix-cache-ohio?profile=default&region=us-east-2&{MY_NIX_CACHE_PARAMS}"
            )),
            _ => None,
        };
        if let Some(nix_substituter) = nix_substituter {
            let mut nix_config = nix::Configuration::read()?;
            nix_config.add_cache(
                "cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=",
                &nix_substituter,
            )?;
            nix_config.write()?;
        }
    }

    {
        let gh_token_encrypted = include_bytes!("../config/github_token.rage");
        let gh_token = String::from_utf8(secret::decrypt(gh_token_encrypted)?)?;
        user_env_vars = user_env_vars.add("GITHUB_TOKEN", gh_token);
    }

    user_env_vars.apply()?;

    Ok(())
}
