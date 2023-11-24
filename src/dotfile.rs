use std::{
    any::Any,
    collections::{HashMap, HashSet},
    error::Error,
    fs, io,
    path::{Path, PathBuf},
};
use thiserror::Error;

pub mod nix_conf;
pub mod unstructured;

pub trait Dotfile: Any {
    fn apply(&self, old_content: &[u8]) -> Result<Option<Vec<u8>>, ApplyError>;
    fn file_permission(&self) -> fs::Permissions;
    fn merge(&mut self, y: Box<dyn Dotfile>) -> Result<(), MergeError>;
    fn as_any(self: Box<Self>) -> Box<dyn Any>;
}

#[derive(Debug, PartialEq, Eq, Error)]
pub enum MergeError {
    #[error("merge conflict, reason: {reason:?}")]
    MergeConflict { reason: String },
}

#[derive(Debug, Error)]
pub enum ApplyError {
    #[error("text encoding error, expected {expected_encoding:?}, {message:?}")]
    TextEncoding {
        expected_encoding: &'static str,
        message: String,
    },
    #[error("other error: {0}")]
    Other(Box<dyn Error + Send + Sync>),
    #[error("path: {path:?}, operation: {operation:?}, io error: {err:?}")]
    IO {
        path: PathBuf,
        operation: String,
        err: std::io::Error,
    },
}

fn apply_utf8<D: Dotfile, F>(old_content: &[u8], consume: F) -> Result<Option<Vec<u8>>, ApplyError>
where
    F: FnOnce(String) -> Result<Option<String>, Box<dyn Error + Send + Sync>>,
{
    let old_content =
        String::from_utf8(old_content.into()).map_err(|err| ApplyError::TextEncoding {
            expected_encoding: "utf8",
            message: err.to_string(),
        })?;
    let new_content = consume(old_content).map_err(|err| ApplyError::Other(err))?;
    Ok(new_content.map(|new_content| new_content.into_bytes()))
}

fn merge_same_type<T: Dotfile + Default, F>(
    x: &mut T,
    y: Box<dyn Dotfile>,
    f: F,
) -> Result<(), MergeError>
where
    F: FnOnce(T, T) -> Result<T, MergeError>,
{
    match y.as_any().downcast::<T>() {
        Ok(y) => {
            let z = f(std::mem::take(x), *y)?;
            *x = z;
            Ok(())
        }
        Err(_) => Err(MergeError::MergeConflict {
            reason: "can't merge dotfiles with different types".to_string(),
        }),
    }
}

pub fn merge_dotfiles(
    dotfiles: Vec<HashMap<PathBuf, Box<dyn Dotfile>>>,
) -> Result<HashMap<PathBuf, Box<dyn Dotfile>>, MergeError> {
    let mut result: HashMap<PathBuf, Box<dyn Dotfile>> = HashMap::new();
    for dotfile_map in dotfiles {
        for (path, dotfile) in dotfile_map {
            match result.remove(&path) {
                None => {
                    result.insert(path, dotfile);
                }
                Some(mut existing_dotfile) => {
                    existing_dotfile.merge(dotfile)?;
                    result.insert(path, existing_dotfile);
                }
            }
        }
    }
    Ok(result)
}

pub fn apply_dotfiles(
    dotfiles: HashMap<PathBuf, Box<dyn Dotfile>>,
) -> Result<HashSet<PathBuf>, ApplyError> {
    let mut changed_files = HashSet::<PathBuf>::new();
    for (path, dotfile) in dotfiles {
        if apply_dotfile(path.as_path(), dotfile)? {
            changed_files.insert(path);
        }
    }
    Ok(changed_files)
}

fn apply_dotfile(path: &Path, dotfile: Box<dyn Dotfile>) -> Result<bool, ApplyError> {
    let lift_err = |operation: &str| {
        let operation = operation.to_string();
        |err: io::Error| ApplyError::IO {
            path: path.to_path_buf(),
            operation,
            err,
        }
    };
    let existing_content = if path.exists() {
        std::fs::read(path).map_err(lift_err("fs::read"))?
    } else {
        vec![]
    };
    let new_content = dotfile.apply(&existing_content)?;
    let file_permissions = dotfile.file_permission();
    if let Some(new_content) = new_content {
        fs::write(path, new_content).map_err(lift_err("fs::write"))?;
        fs::set_permissions(path, file_permissions).map_err(lift_err("fs::set_permissions"))?;
        Ok(true)
    } else {
        Ok(false)
    }
}
