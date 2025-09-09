use ed25519_dalek::VerifyingKey;

use crate::{error::*};

use stylus_sdk::alloy_primitives::FixedBytes;

use borsh::BorshDeserialize;

#[cfg(not(target_arch = "wasm32"))]
use borsh::BorshSerialize;

use arrayvec::ArrayVec;

pub const MAX_ACCOUNT_LIST_SIZE: usize = 1000;

/// Simple list that's sent with every Applicative form to the contract
/// with the list of the unique addresses involved with the calldata.
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug, BorshSerialize))]
#[derive(BorshDeserialize, PartialEq, Eq, Clone)]
pub struct AccountsList {
    keys: ArrayVec<[u8; 32], MAX_ACCOUNT_LIST_SIZE>
}

/// AccountId is the first 4 bytes of the verifying key created from the
/// expanded list that's sent to the program.
pub type AccountId = [u8; 4];

/// Expanded form from the AccountsList, with a simple hashmap for simple
/// lookup of account info.
impl AccountsList {
    pub fn with_solver(self, key: &[u8; 32]) -> Result<Self, Error> {
        todo!()
        /*
        Ok(Self {
            solver: VerifyingKey::from_bytes(key).map_err(|_| err_bad_verifying_key())?,
            keys: self.keys,
            ids: self.ids,
        }) */
    }

    pub fn register(mut self, key: VerifyingKey) -> Self {
        /*
        let id: [u8; 4] = key.as_bytes()[..4].try_into().unwrap();
        let _ = self.keys.insert(key, id);
        let _ = self.ids.insert(id, key);
        Self {
            solver: self.solver,
            keys: self.keys,
            ids: self.ids,
        } */
        todo!()
    }

    pub fn find_key(&self, id: [u8; 4]) -> Result<VerifyingKey, Error> {
        /*if let Some(k) = self.ids.get(&id) {
            Ok(*k)
        } else {
            Err(Error {
                typ: ErrorDiscriminant::SignerNotFoundId(id),
            })
        } */
        todo!()
    }

    pub fn find_key_bytes(&self, id: [u8; 4]) -> Result<FixedBytes<32>, Error> {
        Ok(FixedBytes(self.find_key(id)?.to_bytes()))
    }

    pub fn find_place(&self, key: &VerifyingKey) -> Result<[u8; 4], Error> {
        /*if let Some(k) = self.keys.get(key) {
            Ok(*k)
        } else {
            Err(Error {
                typ: ErrorDiscriminant::SignerNotFoundKey,
            })
        } */
        todo!()
    }
}
