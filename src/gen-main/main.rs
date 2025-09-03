use clap::Parser;

use libpassport::{ops::Op, OurLzss};

#[derive(Clone)]
struct OurOp {
    op: Op,
}

use lzss::{SliceReader, VecWriter};

use std::str::FromStr;

impl std::str::FromStr for OurOp {
    type Err = String;

    fn from_str(_s: &str) -> Result<Self, Self::Err> {
        Ok(OurOp { op: Op::Dummy })
    }
}

#[derive(Parser)]
#[command(version, about)]
struct Args {
    #[arg(value_parser = OurOp::from_str)]
    op: OurOp,
}

fn main() {
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
