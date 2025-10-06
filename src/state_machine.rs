use borsh::{BorshSerialize, BorshDeserialize};

pub type Hash = [u8; 64];

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct BalanceArgs {
    pub ms_ts: u128,
    pub owner: [u8; 20],
    pub asset: [u8; 20],
    pub amt: u128,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct OrderArgs {
    pub desired_asset: [u8; 20],
    pub desired_amt: u128,
    pub from_amt: u128,
    pub max_pol_fee: u16,
    pub ord_partial_fill_okay: bool,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum StateMachine {
    WithdrawFromCommitLeftFilled(CommitDetails),
    WithdrawFromCommitRightFilled(CommitDetails),
    WithdrawFromCommitLeftUnfilledOrderCancel(CommitDetails),
    WithdrawFromCommitRightUnfilledOrderCancel(CommitDetails),
}
