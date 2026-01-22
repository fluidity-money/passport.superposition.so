use borsh::{BorshDeserialize, BorshSerialize};

use alloc::boxed::Box;

pub type Hash = [u8; 64];

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct BalanceArgs {
    pub ms_ts: u32,
    pub owner: [u8; 20],
    pub asset: [u8; 20],
    pub amt: u128,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum Balance {
    Inline(BalanceArgs, Hash),
    Onchain(Hash),
    CommitLeftFilledToBal(Box<Commit>, Hash),
    CommitRightFilledToBal(Box<Commit>, Hash),
    Cancel(Box<Order>, Hash),
    Join(Box<Balance>, Box<Balance>, Hash),
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub struct CommitArgs {
    pub ms_ts: u32,
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum Commit {
    Inline(CommitArgs, Box<Order>, Box<Order>, Hash),
    Onchain(Hash),
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
pub enum Order {
    Inline(OrderArgs, Box<Balance>, Hash),
    Onchain(Hash),
    CommitLeftExcessToOrder(Box<Commit>, Hash),
    CommitRightExcessToOrder(Box<Commit>, Hash),
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum Withdraw {
    Inline(Box<Balance>, Hash),
}

#[derive(Clone, Debug, PartialEq, BorshSerialize, BorshDeserialize)]
pub enum StateMachine {
    Balance(Balance),
    Commit(Commit),
    Order(Order),
    Withdraw(Withdraw),
}
