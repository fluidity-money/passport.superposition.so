#[cfg(target_arch = "x86_64")]

mod host {
    use clap::Parser;

    use stylus_sdk::alloy_primitives::{Address, FixedBytes};

    use libpassport::{
        applicative::{Applicative, EdSig},
        facet::Facet,
        ops::{OpSetter, OpSolver},
        OurLzss,
    };

    use std::str::FromStr;

    use borsh::ser::BorshSerialize;

    use lzss::{SliceReader, VecWriter};

    #[derive(Clone, Parser, Debug)]
    #[command(version, about)]
    enum Args {
        SolverDummy,
        SolveFromArgs {
            accounts: Vec<u64>,
            #[arg(value_parser = Applicative::from_str)]
            applicative: Applicative,
        },
        SolveFromFile,
        Lint,
        ExampleApplicative,
        SetterDummy,
        SetterOnboard {
            verifying_key: FixedBytes<32>,
            verifying_sig: EdSig,
            nonce: u16,
            token: Address,
            value: u128,
            deadline: FixedBytes<32>,
            permit_v: u8,
            permit_r: FixedBytes<32>,
            permit_s: FixedBytes<32>,
        },
        SetterAddLiquidity {
            token: Address,
            recipient: Address,
            value: u128,
            deadline: FixedBytes<32>,
            v: u8,
            r: FixedBytes<32>,
            s: FixedBytes<32>,
        },
    }

    fn match_facet(a: &Args) -> Facet {
        match a {
            Args::SolverDummy | Args::Solve { .. } | Args::Lint | Args::ExampleApplicative => {
                Facet::UserSolver
            }
            Args::SetterDummy | Args::SetterOnboard { .. } | Args::SetterAddLiquidity { .. } => {
                Facet::UserSetter
            }
        }
    }

    pub fn entry() {
        let mut b = Vec::new();
        let op = Args::parse();
        let fc: u8 = match_facet(&op).into();
        match op {
            Args::SolverDummy => OpSolver::Dummy.serialize(&mut b).unwrap(),
            Args::SetterDummy => OpSetter::Dummy.serialize(&mut b).unwrap(),
            _ => (),
        };
        println!(
            "{}{}",
            const_hex::encode(&[fc]),
            const_hex::encode(
                OurLzss::compress_stack(SliceReader::new(&b), VecWriter::with_capacity(1024 * 10))
                    .unwrap(),
            )
        );
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    host::entry()
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {}
