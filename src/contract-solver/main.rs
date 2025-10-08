#![cfg_attr(target_arch = "wasm32", no_main, no_std)]

use libpassport::{
    entry_non_reentrant, immutables::pick_solver_key, network::Network, ops::OpSolver, reentrancy,
    wasm_vm_harness,
};

#[cfg(not(target_arch = "wasm32"))]
use libpassport::host_vm_harness;

#[cfg(all(not(target_arch = "wasm32"), feature = "std"))]
use libpassport::return_data;

use stylus_sdk::{host::VM, prelude::HostAccess};

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

pub fn entry(vm: VM, len: usize) -> usize {
    entry_non_reentrant(vm, len, |s, args| {
        match OpSolver::deserialize(args).unwrap() {
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
                let (r, _rd) = reentrancy::begin_apply(&mut s.app, m);
                s.vm().write_result(&_rd);
                r
            }
        }
    })
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn user_entrypoint(len: usize) -> usize {
    entry(wasm_vm_harness(), len)
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
