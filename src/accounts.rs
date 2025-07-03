use ed25519_dalek::VerifyingKey;

use crate::error::*;

use alloc::vec::Vec;

/// Applicative signature map for a signer verifying key and their place
/// in the map.
#[derive(Debug, PartialEq, Eq)]
pub struct Accounts {
    pub solver: VerifyingKey,
}

impl Accounts {
    pub fn find<'a>(&self, _id: usize) -> Result<&'a VerifyingKey, Error> {
        Err(Error {
            typ: ErrorDiscriminant::SignerNotFound,
            cd: Vec::new(),
        })
    }
}

impl Default for Accounts {
    fn default() -> Self {
        Accounts {
            solver: VerifyingKey::default(),
        }
    }
}
