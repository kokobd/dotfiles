use std::{collections::HashMap, path::PathBuf};

use crate::{
    dotfile::{unstructured::Unstructured, Dotfile},
    secret::Decrpytor,
    Config, DecryptError,
};

pub fn dotfiles(config: &Config) -> Result<HashMap<PathBuf, Box<dyn Dotfile>>, DecryptError> {
    let git_config_path = config.home_dir.join(".gitconfig");
    let git_config: Box<dyn Dotfile> = Box::new(Unstructured::new(
        include_bytes!("../../config/.gitconfig").to_vec(),
    ));
    let gpg_key_path = config.home_dir.join(".gpg/private.gpg");
    let gpg_key: Box<dyn Dotfile> = Box::new(Unstructured::new(
        config
            .decryptor
            .decrypt(include_bytes!("../../config/private.gpg.rage"))
            .map_err(|err| DecryptError {
                path: "config/private.gpg.rage".to_string(),
                error: err,
            })?,
    ));
    Ok(HashMap::from([
        (git_config_path, git_config),
        (gpg_key_path, gpg_key),
    ]))
}
