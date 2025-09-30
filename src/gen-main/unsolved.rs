use libpassport::applicative::{ArgsBalance, ArgsCommit, ArgsOrder};

use serde::{Deserialize as SerdeDeserialize, Serialize as SerdeSerialize};

#[derive(Clone, Debug, Copy)]
pub enum ErrFromRecipe {
    Unknown,
}

impl std::fmt::Display for ErrFromRecipe {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Clone, PartialEq, Debug, SerdeDeserialize, SerdeSerialize)]
pub enum RecipeUnsolved {
    Balance(String, ArgsBalance),
    Withdraw(String, Box<RecipeUnsolved>),
    Order(String, ArgsOrder, Box<RecipeUnsolved>),
    Cancel(String, Box<RecipeUnsolved>),
    Commit(String, ArgsCommit, Box<RecipeUnsolved>, Box<RecipeUnsolved>),
    CommitLeftFilledToBalance(String, Box<RecipeUnsolved>),
    CommitRightFilledToBalance(String, Box<RecipeUnsolved>),
    CommitLeftExcessToOrder(String, Box<RecipeUnsolved>),
    CommitRightExcessToOrder(String, Box<RecipeUnsolved>),
    Join(String, Box<RecipeUnsolved>, Box<RecipeUnsolved>),
}

impl std::fmt::Display for RecipeUnsolved {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", serde_sexpr::to_string(self).unwrap())
    }
}

impl serde::ser::StdError for ErrFromRecipe {}

impl std::str::FromStr for RecipeUnsolved {
    type Err = ErrFromRecipe;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        serde_sexpr::from_str(s).map_err(|_| ErrFromRecipe::Unknown)
    }
}

#[test]
fn test_print_unsolved_recipe() {
    use libpassport::applicative::{Asset, U128};
    println!(
        "{}",
        RecipeUnsolved::Balance(
            "Alex".to_owned(),
            ArgsBalance {
                asset: Asset([
                    0xae, 0xff, 0x36, 0x1b, 0xab, 0x37, 0x08, 0xfa, 0x59, 0xf2, 0x97, 0x2f, 0xd4,
                    0xa3, 0x90, 0x0b, 0xa3, 0x3c, 0xe8, 0x21
                ]),
                chain: U128(0),
                amount: U128(123),
                ms_timestamp: U128(1759221563622),
            }
        )
    );
}
