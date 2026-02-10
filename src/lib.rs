#![cfg_attr(not(feature = "std"), no_std)]

extern crate alloc;

pub mod error;
pub mod result;

pub mod storage;

pub mod network;

pub mod apply;
pub mod conversion;

pub mod solver_context;
pub mod user_context;

pub mod sigs;

pub mod immutables;

pub mod reentrancy;

pub mod applicative;
pub mod state_machine;

pub mod facet;
pub mod ops;

pub mod add_liq;
pub mod onboard;
pub mod view;

pub mod call_eip20_extras;

pub type OurLzss = lzss::Lzss<12, 11, 0, { 1 << 12 }, { 2 << 12 }>;
pub use lzss::{SliceReader, VecWriter};

use bobcat_sdk::{
    entry::{read_args_vec, write_result_slice},
    storage::{flush_cache, reentrancy_guard_const_keccak},
};

use borsh::BorshDeserialize;

#[cfg(feature = "std")]
use clap::Parser as ClapParser;

use core::str::FromStr;

pub use crate::{
    error::{DONE_UNIT, NOOP, R, done_u64},
    network::Network,
    ops::{OpAdmin, OpSetter, OpSolver, OpVault},
};
use crate::{
    ops::CompressedOpSolver,
    view::{view_hash_owner_l, view_hash_owner_r, view_owner, view_withdrawable},
};

use immutables::pick_solver_key;

use state_machine::StateMachine;

#[allow(unused_imports)]
use alloc::boxed::Box;

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "vm_hooks")]
#[allow(unused)]
unsafe extern "C" {
    pub(crate) fn pay_for_memory_grow(pages: u16);
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgsAddr(pub [u8; 20]);

#[derive(Debug, Clone, PartialEq)]
pub struct FromStrErr;

impl core::fmt::Display for FromStrErr {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{self:?}")
    }
}

impl core::error::Error for FromStrErr {}

impl FromStr for ArgsAddr {
    type Err = FromStrErr;

    fn from_str(x: &str) -> Result<Self, Self::Err> {
        const_hex::decode_to_array::<_, 20>(x)
            .map(|x| ArgsAddr(x))
            .map_err(|_| FromStrErr)
    }
}

cfg_if::cfg_if! {
    if #[cfg(feature = "network-testnet")] {
        pub const NETWORK: Network = Network::TESTNET;
    } else if #[cfg(feature = "network-custom")] {
        pub const NETWORK: Network = Network::CUSTOM;
    } else {
        pub const NETWORK: Network = Network::MAINNET;
    }
}

#[cfg(feature = "std")]
#[derive(Debug, Clone, PartialEq, ClapParser)]
#[command(version, about)]
pub struct VmArgs {
    #[arg(
        short,
        long,
        default_value = "0xfeb6034fc7df27df18a3a6bad5fb94c0d3dcb6d5",
        value_parser = ArgsAddr::from_str
    )]
    pub sender: ArgsAddr,
    #[arg(short, long, default_value = "98985")]
    pub chain_id: u64,
    #[arg(
        short,
        long,
        default_value = "0x0000000000000000000000000000000000000000",
        value_parser = ArgsAddr::from_str
    )]
    pub addr: ArgsAddr,
}

pub fn entry_non_reentrant(len: usize, entry: impl FnOnce(&mut &[u8]) -> usize) -> usize {
    let args = read_args_vec(len);
    reentrancy_guard_const_keccak(b"superposition.passport.reentrancy-canary", || {
        let c = entry(
            &mut OurLzss::decompress_stack(
                lzss::SliceReader::new(&args[1..]),
                lzss::VecWriter::with_capacity(1024 * 10),
            )
            .unwrap()
            .as_slice(),
        );
        c
    })
}

pub fn entry_vault(len: usize) -> usize {
    entry_non_reentrant(len, |args| match OpVault::deserialize(args).unwrap() {
        OpVault::MoveLiquidity => todo!(),
    })
}

pub fn entry_setter(len: usize) -> usize {
    entry_non_reentrant(len, |args| {
        let r = match OpSetter::deserialize(args).unwrap() {
            OpSetter::Dummy => DONE_UNIT,
            OpSetter::ViewWithdrawable(owner, asset) => view_withdrawable(&owner, &asset),
            OpSetter::ViewOwner(id) => view_owner(&id),
            OpSetter::ViewHashOwnerLeft(hash) => view_hash_owner_l(&hash),
            OpSetter::ViewHashOwnerRight(hash) => view_hash_owner_r(&hash),
            OpSetter::Onboard(
                key,
                sig,
                contract,
                nonce,
                chain,
                token,
                value,
                deadline,
                permit_v,
                permit_r,
                permit_s,
            ) => onboard::onboard(
                *key, sig, contract, nonce, chain, token, value, deadline, permit_v, permit_r,
                permit_s,
            ),
            OpSetter::AddLiquidity(token, recipient, value, deadline, v, r, s_) => {
                add_liq::add_liq(token, recipient, value, deadline, v, r, s_)
            }
        };
        let rd = match r {
            Ok(_) => 0,
            Err(ref _reason) => {
                #[cfg(feature = "harness-stylus-interpreter")]
                panic!("reverted: {_reason:?}");
                #[allow(unreachable_code)]
                1
            }
        };
        match r {
            Ok(v) => write_result_slice(&borsh::to_vec(&v).unwrap()),
            Err(v) => write_result_slice(&{
                #[cfg(feature = "errors-extra-context")]
                {
                    borsh::to_vec(&v).unwrap()
                }
                #[cfg(not(feature = "errors-extra-context"))]
                {
                    [v.dis_u8().into()]
                }
            }),
        }
        flush_cache();
        rd
    })
}

pub fn entry_admin(len: usize) -> usize {
    entry_non_reentrant(len, |args| match OpAdmin::deserialize(args).unwrap() {
        OpAdmin::Upgrade(_, _, _, _) => todo!(),
    })
}

pub fn entry_solver(len: usize) -> usize {
    entry_non_reentrant(len, |args| {
        let c = CompressedOpSolver::deserialize(args).unwrap();
        match OpSolver::try_from(c).unwrap() {
            OpSolver::Solve(accounts, args) => {
                let r = match conversion::validate(&pick_solver_key(NETWORK), &accounts, &args) {
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
        }
    })
}

pub fn entry_apply(len: usize) -> usize {
    // entry_apply is a unique entrypoint in that it's not designed for
    // external consumption. It's used in a delegatecall chain.
    let args = read_args_vec(len);
    let r = apply::apply(StateMachine::deserialize(&mut args.as_slice()).unwrap());
    match r {
        Ok(v) => write_result_slice(&borsh::to_vec(&v).unwrap()),
        Err(v) => write_result_slice(&[v.dis_u8()]),
    }
    flush_cache();
    0
}
