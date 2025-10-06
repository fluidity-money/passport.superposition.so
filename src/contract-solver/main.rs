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

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
unsafe extern "C" {
    fn exit_early(code: i32);
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry_non_reentrant(len, |s, args| match OpSolver::deserialize(args).unwrap() {
        OpSolver::Solve(accounts, args) => {
            let m = s
                .app
                .validate(&pick_solver_key(NETWORK), &accounts, &args)?;
            // It would be better to hand up to the caller the return value here, but
            // fro codesize reasons, we shortcircuit here using exit_early. This also
            // lets us implicitly flush for all of the other facets.
            let (r, rd) = reentrancy::begin_apply(&mut s.app, m);
            s.vm().write_result(&rd);
            unsafe { exit_early(r) }
            unreachable!()
        }
    })
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {}
