mod nix;
pub mod secret;
use std::{env, path::Path};

pub fn bootstrap() -> anyhow::Result<()> {
    let coder_template = env::var("CODER_TEMPLATE").unwrap_or(String::new());

    if Path::new("/etc/nix").exists() {
        const MY_NIX_CACHE_PARAMS: &'static str =
            "compression=zstd&priority=0&trusted=true&want-mass-query=true";
        let nix_substituter: Option<String> = match coder_template.as_str() {
            "docker" => Some(format!(
            "s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&{MY_NIX_CACHE_PARAMS}"
        )),
            "ecs-ec2" => Some(format!(
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

    Ok(())
}
