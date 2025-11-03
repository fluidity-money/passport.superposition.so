#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    conversion::validate, entry_non_reentrant, immutables::pick_solver_key, network::Network,
    ops::OpSolver, reentrancy,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
use libpassport::return_data;

use bobcat_sdk::entry::write_result_slice;

use borsh::BorshDeserialize;

#[global_allocator]
static ALLOC: mini_alloc::MiniAlloc = mini_alloc::MiniAlloc::INIT;

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

pub fn entry(len: usize) -> usize {
    entry_non_reentrant(len, |args| match OpSolver::deserialize(args).unwrap() {
        OpSolver::Solve(accounts, args) => {
            let r = match validate(&pick_solver_key(NETWORK), &accounts, &args) {
                Ok(v) => v,
                Err(v) => {
                    write_result_slice(&{
                        #[cfg(feature = "errors-extra-context")]
                        {
                            borsh::to_vec(&v).unwrap()
                        }
                        #[cfg(not(feature = "errors-extra-context"))]
                        {
                            [v.dis_u8().into()]
                        }
                    });
                    return 1;
                }
            };
            let (r, _rd) = reentrancy::begin_apply(r);
            write_result_slice(&_rd);
            r
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len)
}
