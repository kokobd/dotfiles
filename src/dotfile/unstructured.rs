use super::{merge_same_type, ApplyError, Dotfile, MergeError};
use std::any::Any;
use std::os::unix::fs::PermissionsExt;

#[derive(Debug)]
pub struct Unstructured {
    content: Vec<u8>,
    permissions: std::fs::Permissions,
}

impl Default for Unstructured {
    fn default() -> Self {
        Self {
            content: Vec::new(),
            permissions: std::fs::Permissions::from_mode(0o644),
        }
    }
}

impl Unstructured {
    pub fn new(content: Vec<u8>) -> Self {
        Self {
            content,
            ..Default::default()
        }
    }

    pub fn new_utf8(content: String) -> Self {
        Self {
            content: content.into_bytes(),
            ..Default::default()
        }
    }

    pub fn with_permissions(self, permissions: std::fs::Permissions) -> Self {
        Self {
            permissions,
            ..self
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

    fn file_permission(&self) -> std::fs::Permissions {
        self.permissions.clone()
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
