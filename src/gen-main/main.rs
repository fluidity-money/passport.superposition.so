mod accounts;
mod convert;
mod reader;
mod unsolved;

mod host {
    use clap::Parser;

    use std::{fs::File, io::stdin};

    use super::{
        accounts::{Accounts, Key},
        convert::unsolved_to_applicative,
        reader::RecipeReader,
        unsolved::RecipeUnsolved,
    };

    use stylus_sdk::alloy_primitives::{Address, FixedBytes};

    use libpassport::{
        applicative::{Applicative, EdSig},
        facet::Facet,
        ops::{OpSetter, OpSolver},
        OurLzss,
    };

    use borsh::ser::BorshSerialize;

    use lzss::{SliceReader, VecWriter};

    use ed25519_dalek::SigningKey;

    #[derive(Clone, Parser, Debug)]
    #[command(version, about)]
    enum Args {
        SolverDummy,
        SolveUnsolvedRecipeArgs {
            #[arg(short = 'a', long = "accounts")]
            accounts: Option<Accounts>,
            #[arg(short = 'f', long = "accounts-file")]
            accounts_file: Option<String>,
            signer: Key,
            recipe: RecipeUnsolved,
        },
        SolveUnsolvedRecipeFile {
            #[arg(short = 'a', long = "accounts")]
            accounts: Option<Accounts>,
            #[arg(short = 'f', long = "accounts-file")]
            accounts_file: Option<String>,
            signer: Key,
            file: Option<String>,
        },
        CalldataFromSolvedRecipeArgs {
            applicative: Applicative,
        },
        CalldataFromSolvedRecipeFile {
            file: Option<String>,
        },
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
            Args::SolverDummy
            | Args::SolveUnsolvedRecipeArgs { .. }
            | Args::SolveUnsolvedRecipeFile { .. }
            | Args::CalldataFromSolvedRecipeArgs { .. }
            | Args::CalldataFromSolvedRecipeFile { .. }
            | Args::Lint
            | Args::ExampleApplicative => Facet::UserSolver,
            Args::SetterDummy | Args::SetterOnboard { .. } | Args::SetterAddLiquidity { .. } => {
                Facet::UserSetter
            }
        }
    }

    fn simple_calldata(op: Args) {
        let mut b = Vec::new();
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

    fn solve_unsolved_recipe(accounts: Accounts, signer: SigningKey, recipe: RecipeUnsolved) {
        println!("{}", unsolved_to_applicative(accounts, signer, recipe));
    }

    pub fn entry() {
        let op = Args::parse();
        match op {
            Args::SolverDummy | Args::SetterDummy => simple_calldata(op),
            Args::SolveUnsolvedRecipeArgs {
                accounts,
                accounts_file,
                signer,
                recipe,
            } => {
                let accounts = match (accounts, accounts_file) {
                    (_, Some(_)) | (None, None) => {
                        unimplemented!("Only accounts arguments for now")
                    }
                    (Some(accounts), _) => accounts,
                };
                solve_unsolved_recipe(accounts, SigningKey::from_bytes(&signer.0), recipe)
            }
            Args::SolveUnsolvedRecipeFile {
                accounts,
                accounts_file,
                signer,
                file,
            } => {
                let accounts = match (accounts, accounts_file) {
                    (_, Some(_)) | (None, None) => {
                        unimplemented!("Only accounts arguments for now")
                    }
                    (Some(accounts), _) => accounts,
                };
                let recipe = match file {
                    Some(f) => serde_sexpr::from_reader(RecipeReader::new(File::open(f).unwrap())),
                    None => serde_sexpr::from_reader(RecipeReader::new(stdin())),
                }
                .unwrap();
                solve_unsolved_recipe(accounts, SigningKey::from_bytes(&signer.0), recipe)
            }
            _ => unimplemented!(),
        }
    }
}

#[cfg(target_arch = "x86_64")]
fn main() {
    host::entry()
}

#[cfg(not(target_arch = "x86_64"))]
fn main() {}
