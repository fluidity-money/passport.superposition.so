/*
use clap::Parser;

use libpassport::ops::Op;

#[derive(Clone)]
struct OurOp {
    op: Op,
}

impl std::str::FromStr for OurOp {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
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
        const_hex::encode(match Args::parse().op.op {
            Op::Dummy => borsh::to_vec(&Op::Dummy).unwrap(),
            _ => unimplemented!(),
        })
    );
}
*/

fn main() {}
