#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    entry,
    ops::OpSolver,
    {DONE_UNIT, NOOP},
    network::Network
};

use borsh::BorshDeserialize;

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len, |s, args| match OpSolver::deserialize(args).unwrap() {
        OpSolver::Dummy => NOOP,
        OpSolver::Solve(accounts, args) => s
            .app
            .validate(Network::MAINNET, &accounts, args)
            .and_then(|x| s.app.apply(x))
            .and_then(|_| DONE_UNIT),
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
