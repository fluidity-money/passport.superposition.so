#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    entry_non_reentrant, immutables::pick_solver_key, network::Network, ops::OpSolver, reentrancy,
};

use stylus_sdk::prelude::HostAccess;

use borsh::BorshDeserialize;

extern crate alloc;

cfg_if::cfg_if! {
    if #[cfg(feature = "network-testnet")] {
        pub const NETWORK: Network = Network::TESTNET;
    } else if #[cfg(feature = "network-custom")] {
        pub const NETWORK: Network = Network::CUSTOM;
    } else {
        pub const NETWORK: Network = Network::MAINNET;
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry_non_reentrant(len, |s, args| match OpSolver::deserialize(args).unwrap() {
        OpSolver::Solve(accounts, args) => {
            let m = match s
                .app
                .validation
                .validate(&pick_solver_key(NETWORK), &accounts, &args)
            {
                Ok(v) => v,
                Err(v) => {
                    s.vm().write_result(&[v.dis_u8()]);
                    return 1;
                }
            };
            // It would be better to hand up to the caller the return value here, but
            // for codesize reasons, we shortcircuit here using exit_early. This also
            // lets us implicitly flush for all of the other facets.
            let (r, _rd) = reentrancy::begin_apply(&mut s.app, m);
            s.vm().write_result(&_rd);
            r
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
