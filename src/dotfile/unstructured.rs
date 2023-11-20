use std::any::Any;

use super::{merge_same_type, ApplyError, Dotfile, MergeError};

#[derive(Debug, Default)]
pub struct Unstructured {
    content: Vec<u8>,
}

impl Unstructured {
    pub fn new(content: Vec<u8>) -> Self {
        Self { content }
    }

    pub fn new_utf8(content: String) -> Self {
        Self {
            content: content.into_bytes(),
        }
    }
}

impl Dotfile for Unstructured {
    fn apply(&self, old_content: &[u8]) -> Result<Option<Vec<u8>>, ApplyError> {
        if self.content == old_content {
            Ok(None)
        } else {
            Ok(Some(self.content.clone()))
        }
    }

    fn merge(&mut self, y: Box<dyn Dotfile>) -> Result<(), MergeError> {
        merge_same_type(self, y, |x, y| {
            if x.content == y.content {
                Ok(x)
            } else {
                Err(MergeError::MergeConflict {
                    reason: "Unstructured files with different content cannot be merged"
                        .to_string(),
                })
            }
        })
    }

    fn as_any(self: Box<Self>) -> Box<dyn Any> {
        Box::new(*self)
    }
}
