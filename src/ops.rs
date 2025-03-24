use crate::encoding::*;

use alloc::vec::Vec;

use borsh::{BorshDeserialize, BorshSerialize};

/// The hash to use alongside the signature is created from the concenation
/// of the take and goal.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct EdSig {
    pub r: [u8; 32],
    pub s: [u8; 32],
}

/// EIP signed transaction that sets up the special account that one of the users will use.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct Eip712SetAddrSig {}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct MatchReq {
    pub sender: [u8; 32],
    pub spend_token: BAddress,
    pub spend_amt: BU256,
    pub goal_token: BAddress,
    pub goal_amt: BU256,
    pub nonce: BU256,
    pub deadline: u64,
}

/// Blob sent when a permit takes place when someone funds the smart
/// account using the degraded experience.
#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq, Debug)]
pub struct PermitReq {
    pub token: BAddress,
    pub value: BU256,
    pub deadline: BU256,
    pub v: u8,
    pub r: [u8; 32],
    pub s: [u8; 32],
}

#[derive(BorshSerialize, BorshDeserialize, Clone, PartialEq)]
#[cfg_attr(not(target_arch = "wasm32"), derive(Debug))]
pub enum Op {
    /// Dummy operation.
    Dummy,
    /// Set the associated address after validating the signature given. Uses the message sender
    /// as the source of truth for this.
    SetAddress(([u8; 32], BAddress)),
    /// A pair of requests to match by taking amounts from both sides.
    Match(Vec<(MatchReq, EdSig)>, Vec<PermitReq>),
}

#[cfg(not(target_arch = "wasm32"))]
pub fn strat_match_req() -> impl proptest::prelude::Strategy<Value = MatchReq> {
    use crate::utils::{strat_address, strat_large_u256};
    use proptest::prelude::*;
    (
        any::<[u8; 32]>(),
        strat_address(),
        strat_large_u256(),
        strat_address(),
        strat_large_u256(),
        strat_large_u256(),
        any::<u64>(),
    )
        .prop_map(
            |(sender, spend_token, spend_amt, goal_token, goal_amt, nonce, deadline)| MatchReq {
                sender,
                spend_token: BAddress { x: spend_token },
                spend_amt: BU256 { x: spend_amt },
                goal_token: BAddress { x: goal_token },
                goal_amt: BU256 { x: goal_amt },
                nonce: BU256 { x: nonce },
                deadline,
            },
        )
}
