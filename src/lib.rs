mod nix;
use std::env;

pub fn bootstrap() -> anyhow::Result<()> {
    let coder_template = env::var("CODER_TEMPLATE").unwrap_or(String::new());
    const MY_NIX_CACHE_PARAMS: &'static str =
        "compression=zstd&priority=0&trusted=true&want-mass-query=true";
    let nix_substituter: Option<String> = match coder_template.as_str() {
        "docker" => Some(format!(
            "s3://nix-cache?endpoint=http://192.168.31.2:9091&profile=minio&{MY_NIX_CACHE_PARAMS}"
        )),
        "ecs-ec2" => Some(String::from("TODO")),
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
    Ok(())
}
