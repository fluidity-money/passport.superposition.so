use ed25519_dalek::VerifyingKey;

use crate::error::*;

use alloc::vec::Vec;

use hashbrown::HashMap;

/// Applicative signature map for a signer verifying key and their place
/// in the map.
#[derive(Debug, PartialEq, Eq)]
pub struct Accounts {
    pub solver: VerifyingKey,
    keys: HashMap<VerifyingKey, usize>,
    ids: Vec<VerifyingKey>,
}

impl Accounts {
    pub fn with_solver(self, key: [u8; 32]) -> Accounts {
        Accounts {
            solver: VerifyingKey::from_bytes(&key).unwrap(),
            keys: self.keys,
            ids: self.ids,
        }
    }

    pub fn register(mut self, key: VerifyingKey) -> Accounts {
        let _ = self.keys.insert(key, self.ids.len());
        self.ids.push(key);
        Accounts {
            solver: self.solver,
            keys: self.keys,
            ids: self.ids,
        }
    }

    pub fn find_key(&self, id: usize) -> Result<VerifyingKey, Error> {
        if let Some(k) = self.ids.get(id) {
            Ok(*k)
        } else {
            Err(Error {
                typ: ErrorDiscriminant::SignerNotFoundId(id),
                cd: Vec::new(),
            })
        }
    }

    pub fn find_place(&self, key: &VerifyingKey) -> Result<usize, Error> {
        if let Some(k) = self.keys.get(key) {
            Ok(*k)
        } else {
            Err(Error {
                typ: ErrorDiscriminant::SignerNotFoundKey,
                cd: Vec::new(),
            })
        }
    }
}

impl Default for Accounts {
    fn default() -> Self {
        Accounts {
            solver: VerifyingKey::default(),
            keys: HashMap::new(),
            ids: Vec::new(),
        }
    }
}
