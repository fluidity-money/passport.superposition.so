use clap::Parser;

use std::{fs::File, io::stdin, str::FromStr};

use super::{
    accounts::{Accounts, Key},
    convert::unsolved_to_solver,
    reader::RecipeReader,
    unsolved::RecipeUnsolved,
};

use libpassport::{
    applicative::EdSig,
    error::Error,
    facet::Facet,
    ops::{OpSetter, OpSolver},
    result::Res,
    sigs::make_onboarding_sig,
    OurLzss,
};

use bobcat_sdk::maths::U;

use borsh::{ser::BorshSerialize, BorshDeserialize};

use lzss::{SliceReader, VecWriter};

use ed25519_dalek::{Signer, SigningKey};

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
        solver: OpSolver,
    },
    CalldataFromSolvedRecipeFile {
        file: Option<String>,
    },
    Lint,
    ExampleApplicative,
    SetterDummy,
    SetterOnboardWithKey {
        signing_key: Key,
        owner: ArgsAddr,
        contract: ArgsAddr,
        nonce: u16,
        chain: u64,
        token: ArgsAddr,
        value: u128,
        deadline: u128,
        permit_v: u8,
        #[arg(value_parser = ArgsAddr::from_str)]
        permit_r: U,
        #[arg(value_parser = ArgsAddr::from_str)]
        permit_s: U,
    },
    SetterOnboardNoKey {
        #[arg(value_parser = ArgsAddr::from_str)]
        verifying_key: U,
        verifying_sig: EdSig,
        contract: ArgsAddr,
        nonce: u16,
        chain: u64,
        token: ArgsAddr,
        value: u128,
        deadline: u128,
        permit_v: u8,
        #[arg(value_parser = ArgsAddr::from_str)]
        permit_r: U,
        #[arg(value_parser = ArgsAddr::from_str)]
        permit_s: U,
    },
    SetterAddLiquidity {
        token: ArgsAddr,
        recipient: ArgsAddr,
        value: u128,
        #[arg(value_parser = ArgsAddr::from_str)]
        deadline: U,
        v: u8,
        #[arg(value_parser = ArgsAddr::from_str)]
        r: U,
        #[arg(value_parser = ArgsAddr::from_str)]
        s: U,
    },
    DecodeRes,
    DecodeErr,
    DecodeErrExtraContext,
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
        Args::SetterDummy
        | Args::SetterOnboardWithKey { .. }
        | Args::SetterOnboardNoKey { .. }
        | Args::SetterAddLiquidity { .. } => Facet::UserSetter,
        _ => unimplemented!(),
    }
}

fn compress<T: BorshSerialize>(op: T) -> String {
    const_hex::encode(
        OurLzss::compress_stack(
            SliceReader::new(&borsh::to_vec(&op).unwrap()),
            VecWriter::with_capacity(1024 * 10),
        )
        .unwrap(),
    )
}

fn simple_calldata(op: Args) {
    let fc: u8 = match_facet(&op).into();
    println!(
        "{}{}",
        const_hex::encode(&[fc]),
        match op {
            Args::SetterDummy => compress(OpSetter::Dummy),
            _ => unimplemented!(),
        }
    );
}

fn solve_unsolved_recipe(accounts: Accounts, signer: SigningKey, recipe: RecipeUnsolved) {
    println!("{}", unsolved_to_solver(accounts, signer, recipe));
}

fn read_file_no_whitespace(f: &str) -> std::io::Result<String> {
    let content = std::fs::read_to_string(f)?;
    Ok(content.chars().filter(|c| !c.is_whitespace()).collect())
}

fn perform_unsolved_solving(op: Args) {
    match op {
        Args::SolveUnsolvedRecipeArgs {
            accounts,
            accounts_file,
            signer,
            recipe,
        } => {
            let accounts = match (accounts, accounts_file) {
                (_, Some(f)) => Accounts::from_str(&read_file_no_whitespace(&f).unwrap()).unwrap(),
                (None, None) => panic!("No accounts given"),
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
                (_, Some(f)) => Accounts::from_str(&read_file_no_whitespace(&f).unwrap()).unwrap(),
                (None, None) => panic!("No accounts given"),
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

fn solve_cd(op: OpSolver) {
    println!(
        "{}{}",
        const_hex::encode(&[Facet::UserSolver.into()]),
        compress(op)
    );
}

fn solved_calldata(op: Args) {
    match op {
        Args::CalldataFromSolvedRecipeArgs { solver } => solve_cd(solver),
        Args::CalldataFromSolvedRecipeFile { file } => solve_cd(
            match file {
                Some(f) => serde_sexpr::from_reader(RecipeReader::new(File::open(f).unwrap())),
                None => serde_sexpr::from_reader(RecipeReader::new(stdin())),
            }
            .unwrap(),
        ),
        _ => unimplemented!(),
    }
}

fn setter_onboard_with_key(
    signing_key: Key,
    owner: ArgsAddr,
    contract: ArgsAddr,
    nonce: u16,
    chain: u64,
    token: ArgsAddr,
    value: u128,
    deadline: u128,
    permit_v: u8,
    permit_r: U,
    permit_s: U,
) {
    let key = SigningKey::from_bytes(&signing_key.0);
    let sig: EdSig = key
        .sign(&make_onboarding_sig(
            &owner.0,
            &contract.0,
            nonce,
            chain,
        ))
        .to_bytes()
        .into();
    setter_onboard_no_key(
        U(key.verifying_key().to_bytes()),
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
    )
}

fn setter_onboard_no_key(
    verifying_key: U,
    verifying_sig: EdSig,
    contract: ArgsAddr,
    nonce: u16,
    chain: u64,
    token: ArgsAddr,
    value: u128,
    deadline: u128,
    permit_v: u8,
    permit_r: U,
    permit_s: U,
) {
    println!(
        "{}{}",
        const_hex::encode(&[Facet::UserSetter.into()]),
        compress(OpSetter::Onboard(
            verifying_key,
            verifying_sig.into(),
            contract.0,
            nonce,
            chain,
            token.0,
            value,
            U::from(deadline),
            permit_v,
            permit_r,
            permit_s
        ))
    );
}

fn decode_res() {
    let mut b = String::new();
    stdin().read_line(&mut b).unwrap();
    let b = const_hex::decode(b.trim()).unwrap();
    println!("{}", Res::deserialize(&mut b.as_slice()).unwrap())
}

fn decode_err() {
    let mut b = String::new();
    stdin().read_line(&mut b).unwrap();
    let s = const_hex::decode(b.trim()).unwrap()[0];
    println!("{:?}", Error::try_from(s).unwrap())
}

fn decode_err_extra_context() {
    let mut b = String::new();
    stdin().read_line(&mut b).unwrap();
    let e: Error = borsh::de::from_slice(&const_hex::decode(b.trim()).unwrap()).unwrap();
    println!("{e}");
}

pub fn entry() {
    let op = Args::parse();
    match op {
        Args::SolverDummy | Args::SetterDummy => simple_calldata(op),
        Args::SolveUnsolvedRecipeArgs { .. } | Args::SolveUnsolvedRecipeFile { .. } => {
            perform_unsolved_solving(op)
        }
        Args::CalldataFromSolvedRecipeArgs { .. } | Args::CalldataFromSolvedRecipeFile { .. } => {
            solved_calldata(op)
        }
        Args::SetterOnboardWithKey {
            signing_key,
            owner,
            contract,
            nonce,
            chain,
            token,
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        } => setter_onboard_with_key(
            signing_key,
            owner,
            contract,
            nonce,
            chain,
            token,
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        ),
        Args::SetterOnboardNoKey {
            verifying_key,
            verifying_sig,
            contract,
            nonce,
            chain,
            token,
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        } => setter_onboard_no_key(
            verifying_key,
            verifying_sig,
            contract,
            nonce,
            chain,
            token,
            value,
            deadline,
            permit_v,
            permit_r,
            permit_s,
        ),
        Args::DecodeRes => decode_res(),
        Args::DecodeErr => decode_err(),
Args::        DecodeErrExtraContext  => decode_err_extra_context(),
        _ => unimplemented!(),
    }
}
