use age::Identity;
use std::error::Error;
use std::io::{Cursor, Read};
use std::iter;
use thiserror::Error;

pub trait Decrpytor {
    fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>>;
}

pub struct AgeDecryptor {
    identity: age::ssh::Identity,
}

#[derive(Debug, Error)]
pub enum AgeIdentityParseError {
    #[error("failed to parse identity")]
    IOError(#[from] std::io::Error),
}

impl AgeDecryptor {
    pub fn new(identity_str: String) -> Result<Self, AgeIdentityParseError> {
        let identity = age::ssh::Identity::from_buffer(Cursor::new(identity_str), None)
            .map_err(AgeIdentityParseError::IOError)?;
        Ok(Self { identity })
    }
}

#[derive(Debug, Error)]
enum AgeDecryptionError {
    #[error("unexpected io error")]
    IOError(#[from] std::io::Error),
    #[error("failed to decrypt")]
    AgeError(#[from] age::DecryptError),
}

impl Decrpytor for AgeDecryptor {
    fn decrypt(&self, bytes: &[u8]) -> Result<Vec<u8>, Box<dyn Error>> {
        let encrypted_data = Cursor::new(bytes);
        let decryptor = match age::Decryptor::new_buffered(encrypted_data)
            .map_err(AgeDecryptionError::AgeError)?
        {
            age::Decryptor::Recipients(d) => d,
            _ => unreachable!(),
        };

        let mut decrypted = vec![];
        let identity: &dyn Identity = &self.identity;
        let mut reader = decryptor
            .decrypt(iter::once(identity))
            .map_err(AgeDecryptionError::AgeError)?;
        reader
            .read_to_end(&mut decrypted)
            .map_err(AgeDecryptionError::IOError)?;
        Ok(decrypted)
    }
}
