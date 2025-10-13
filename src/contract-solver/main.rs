#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    conversion::validate, entry_non_reentrant, immutables::pick_solver_key, network::Network,
    ops::OpSolver, reentrancy,
};

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
use libpassport::return_data;

use bobcat_sdk::entry::write_result;

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

pub fn entry(len: usize) -> usize {
    entry_non_reentrant(len, |args| match OpSolver::deserialize(args).unwrap() {
        OpSolver::Solve(accounts, args) => {
            let r = match validate(&pick_solver_key(NETWORK), &accounts, &args) {
                Ok(v) => v,
                Err(v) => {
                    write_result(&{
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
            write_result(&_rd);
            r
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(len)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() {
    let (vm, len) = host_vm_harness();
    let c = entry(vm, len).try_into().unwrap();
    #[cfg(feature = "std")]
    {
        let d = const_hex::encode(&return_data());
        if c > 0 {
            eprintln!("0x{d}");
        } else {
            println!("0x{d}");
        }
    }
    std::process::exit(c)
}
