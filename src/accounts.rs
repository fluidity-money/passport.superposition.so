use ed25519_dalek::VerifyingKey;

use crate::{error::*, immutables::*};

use alloc::vec::Vec;

use hashbrown::HashMap;

/// Simple list that's sent with every Applicative form to the contract
/// with the list of the unique addresses involved with the calldata.
pub struct AccountsList {
    pub solver: VerifyingKey,
    keys: Vec<VerifyingKey>,
}

/// AccountId is the first 4 bytes of the verifying key created from the
/// expanded list that's sent to the program.
pub type AccountId = [u8; 4];

#[derive(Debug, PartialEq, Eq)]
pub struct AccountsExpanded {
    pub solver: VerifyingKey,
    keys: HashMap<VerifyingKey, AccountId>,
    ids: HashMap<AccountId, VerifyingKey>,
}

fn err_bad_verifying_key() -> Error {
    Error {
        typ: ErrorDiscriminant::BadVerifyingKey,
        cd: Vec::new(),
    }
}

impl AccountsList {
    pub fn from(solver: &[u8; 32], keys: Vec<[u8; 32]>) -> Result<Self, Error> {
        Ok(Self {
            solver: VerifyingKey::from_bytes(solver).unwrap(),
            keys: keys
                .iter()
                .map(|x| VerifyingKey::from_bytes(x).map_err(|_| err_bad_verifying_key()))
                .collect::<Result<Vec<_>, _>>()?,
        })
    }

    pub fn from_testnet(keys: Vec<[u8; 32]>) -> Result<Self, Error> {
        Self::from(&SOLVER_KEY_TESTNET, keys)
    }
}

impl From<AccountsList> for AccountsExpanded {
    fn from(v: AccountsList) -> Self {
        let mut keys = HashMap::with_capacity(v.keys.len());
        let mut ids = HashMap::with_capacity(v.keys.len());
        for k in v.keys {
            let id: [u8; 4] = k.as_bytes()[..4].try_into().unwrap();
            let _ = keys.insert(k, id);
            let _ = ids.insert(id, k);
        }
        Self {
            solver: v.solver,
            keys,
            ids,
        }
    }
}

/// Expanded form from the AccountsList, with a simple hashmap for simple
/// lookup of account info.
impl AccountsExpanded {
    pub fn with_solver(self, key: [u8; 32]) -> Result<Self, Error> {
        Ok(Self {
            solver: VerifyingKey::from_bytes(&key).map_err(|_| err_bad_verifying_key())?,
            keys: self.keys,
            ids: self.ids,
        })
    }

    pub fn register(mut self, key: VerifyingKey) -> Self {
        let id: [u8; 4] = key.as_bytes()[..4].try_into().unwrap();
        let _ = self.keys.insert(key, id);
        let _ = self.ids.insert(id, key);
        Self {
            solver: self.solver,
            keys: self.keys,
            ids: self.ids,
        }
    }

    pub fn find_key(&self, id: [u8; 4]) -> Result<VerifyingKey, Error> {
        if let Some(k) = self.ids.get(&id) {
            Ok(*k)
        } else {
            Err(Error {
                typ: ErrorDiscriminant::SignerNotFoundId(id),
                cd: Vec::new(),
            })
        }
    }

    pub fn find_place(&self, key: &VerifyingKey) -> Result<[u8; 4], Error> {
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

impl Default for AccountsExpanded {
    fn default() -> Self {
        Self {
            solver: VerifyingKey::default(),
            keys: HashMap::new(),
            ids: HashMap::new(),
        }
    }
}
