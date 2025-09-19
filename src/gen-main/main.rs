#[cfg(target_arch = "x86_64")]

mod host {
    use clap::Parser;

    use libpassport::{ops::OpSolver, OurLzss};

    #[derive(Clone)]
    struct OurOp {
        op: OpSolver,
    }

    use lzss::{SliceReader, VecWriter};

    use std::str::FromStr;

    impl std::str::FromStr for OurOp {
        type Err = String;

        fn from_str(_s: &str) -> Result<Self, Self::Err> {
            Ok(OurOp { op: OpSolver::Dummy })
        }
    }

    #[derive(Parser)]
    #[command(version, about)]
    struct Args {
        #[arg(value_parser = OurOp::from_str)]
        op: OurOp,
    }

    pub fn entry() {
        println!(
            "{}",
            const_hex::encode(
                OurLzss::compress_stack(
                    SliceReader::new(&borsh::to_vec(&Args::parse().op.op).unwrap()),
                    VecWriter::with_capacity(1024 * 10),
                )
                .unwrap()
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
