use std::{collections::HashMap, path::PathBuf};

use crate::{
    dotfile::{unstructured::Unstructured, Dotfile},
    secret::Decrpytor,
    Config, DecryptError,
};

pub fn dotfiles(config: &Config) -> Result<HashMap<PathBuf, Box<dyn Dotfile>>, DecryptError> {
    let aws_config_path = config.home_dir.join(".aws/config");
    let aws_config: Box<dyn Dotfile> = Box::new(Unstructured::new(
        include_bytes!("../../config/aws/config").to_vec(),
    ));
    let aws_credentials_path = config.home_dir.join(".aws/credentials");
    let aws_credentials: Box<dyn Dotfile> = Box::new(Unstructured::new(
        config
            .decryptor
            .decrypt(include_bytes!("../../config/aws/credentials.rage"))
            .map_err(|err| DecryptError {
                path: "config/aws/credentials.rage".to_string(),
                error: err,
            })?,
    ));
    Ok(HashMap::from([
        (aws_config_path, aws_config),
        (aws_credentials_path, aws_credentials),
    ]))
}
