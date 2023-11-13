mod nix;
use std::env;

pub fn bootstrap() -> anyhow::Result<()> {
    let coder_template = env::var("CODER_TEMPLATE").unwrap_or(String::new());
    let nix_substituter: Option<String> = match coder_template.as_str() {
        "docker" => {
            Some(String::from("s3://nix-cache?endpoint=http://192.168.31.2:9090&compression=zstd&priority=0&profile=minio&trusted=true&want-mass-query=true"))
        }
        "ecs-ec2" => {
            Some(String::from("TODO"))
        }
        _ => {
            None
        }
    };
    if let Some(nix_substituter) = nix_substituter {
        let mut nix_config = nix::Configuration::read()?;
        nix_config.add_cache(
            "cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=",
            &nix_substituter,
        )?;
        nix_config.write()?;
    }
    Ok(())
}
