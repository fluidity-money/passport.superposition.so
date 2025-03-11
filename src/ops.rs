use crate::encoding::*;

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

/// The hash to use alongside the signature is created from the concenation of the allowlist
/// and spendable goals.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub struct SecpSig {
    r: [u8; 32],
    s: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum Op {
    /// Dummy operation.
    Dummy,
    /// Signature-free Spend function. Always reverts.
    SimSpend {
        allowlist: Vec<BAddress>,
        spendable: Vec<(BAddress, BU256)>,
        cds: Vec<Vec<u8>>,
    },
    /// Hash the allowlist and spendable goals, then use that to verify the signature matches
    /// the owner of this smart account.
    Spend {
        allowlist: Vec<BAddress>,
        spendable: Vec<(BAddress, BU256)>,
        sig: SecpSig,
        cds: Vec<Vec<u8>>,
    },
}
