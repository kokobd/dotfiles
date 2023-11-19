pub mod nix_conf;
pub mod unstructured;

pub trait Dotfile {
    type ApplyError;
    fn apply(self, old_content: &[u8]) -> Result<Option<Vec<u8>>, ApplyError<Self::ApplyError>>;
    fn merge(x: Self, y: Self) -> Result<Box<Self>, MergeError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum MergeError {
    MergeConflict { reason: String },
}

#[derive(Debug, PartialEq, Eq)]
pub enum ApplyError<T> {
    TextEncoding {
        expected_encoding: &'static str,
        message: String,
    },
    Other(T),
}

fn apply_utf8<D: Dotfile, F>(old_content: &[u8], consume: F) -> Result<Option<Vec<u8>>, ApplyError<D::ApplyError>>
where
    F: FnOnce(String) -> Result<Option<String>, D::ApplyError>,
{
    let old_content = String::from_utf8(old_content.into()).map_err(|err| {
        ApplyError::<D::ApplyError>::TextEncoding {
            expected_encoding: "utf8",
            message: err.to_string(),
        }
    })?;
    let new_content = consume(old_content).map_err(|err| ApplyError::Other(err))?;
    Ok(new_content.map(|new_content| new_content.into_bytes()))
}
