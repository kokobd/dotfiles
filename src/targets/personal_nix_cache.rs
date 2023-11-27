use crate::{
    dotfile::{nix_conf::NixConf, unstructured::Unstructured, Dotfile},
    secret::Decrpytor,
    Config, DecryptError, Region,
};
use pathbuf::pathbuf;
use std::{collections::HashMap, os::unix::fs::PermissionsExt, path::PathBuf};

pub fn dotfiles(config: &Config) -> Result<HashMap<PathBuf, Box<dyn Dotfile>>, DecryptError> {
    const MY_NIX_CACHE_PARAMS: &'static str =
        "compression=zstd&priority=0&trusted=true";
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
    if let Some(nix_substituter) = nix_substituter {
        let nix_conf: Box<dyn Dotfile> = Box::new(
            NixConf::new()
                .with_secret_key_files(vec!["/etc/nix/secret-key".to_string()])
                .with_post_build_hook("/etc/nix/post-build-hook".to_string())
                .with_substituters(vec![nix_substituter.clone()])
                .with_trusted_public_keys(vec![
                    "cache.zelinf.net:poESahuRAXqYC2QPevSId+pTtoq0P1XfTxaSHRgfvVI=".to_string(),
                ]),
        );
        let nix_post_build_hook: Box<dyn Dotfile> = Box::new(
            Unstructured::new_utf8(format!(
                r#"#!/usr/bin/env bash
set -eu
set -f # disable globbing
export IFS=' '

echo "Signing paths" $OUT_PATHS
nix store sign --recursive --key-file /etc/nix/secret-key $OUT_PATHS
echo "Uploading paths" $OUT_PATHS
exec nix copy --to '{}' $OUT_PATHS
"#,
                nix_substituter
            ))
            .with_permissions(std::fs::Permissions::from_mode(0o777)),
        );
        let secret_key_file: Box<dyn Dotfile> = Box::new(
            Unstructured::new(
                config
                    .decryptor
                    .decrypt(include_bytes!("../../config/nix-secret-key.rage"))
                    .map_err(|err| DecryptError {
                        path: "config/nix-secret-key.rage".to_string(),
                        error: err,
                    })?,
            )
            .with_permissions(std::fs::Permissions::from_mode(0o600)),
        );
        Ok(HashMap::from([
            (pathbuf!["/etc", "nix", "nix.conf"], nix_conf),
            (
                pathbuf!["/etc", "nix", "post-build-hook"],
                nix_post_build_hook,
            ),
            (pathbuf!["/etc", "nix", "secret-key"], secret_key_file),
        ]))
    } else {
        Ok(HashMap::new())
    }
}
