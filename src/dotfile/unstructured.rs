use super::{Dotfile, MergeError, ApplyError};

pub struct Unstructured {
    content: Vec<u8>,
}

impl Unstructured {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }
}

impl Dotfile for Unstructured {
    type ApplyError = ();

    fn apply(self, old_content: &[u8]) -> Result<Option<Vec<u8>>, ApplyError<Self::ApplyError>> {
        if self.content == old_content {
            Ok(None)
        } else {
            Ok(Some(self.content))
        }
    }

    fn merge(x: Self, y: Self) -> Result<Box<Self>, MergeError> {
        if x.content == y.content {
            Ok(Box::new(x))
        } else {
            Err(MergeError::MergeConflict {
                reason: "Unstructured files with different content cannot be merged".to_string(),
            })
        }
    }
}
