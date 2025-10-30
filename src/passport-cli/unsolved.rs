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
    Commit(ArgsCommit, Box<RecipeUnsolved>, Box<RecipeUnsolved>),
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
#[ignore]
fn test_print_unsolved_recipe() {
    use libpassport::applicative::{Asset, U128};
    let asset_a = Asset([
        138, 193, 199, 165, 65, 110, 126, 3, 66, 181, 50, 169, 220, 157, 116, 201, 152, 224, 231,
        144,
    ]);
    let asset_b = Asset([
        179, 104, 133, 120, 164, 207, 97, 51, 43, 152, 157, 145, 200, 37, 248, 25, 3, 77, 19, 49,
    ]);
    println!(
        "{}",
        RecipeUnsolved::Withdraw(
            "Alex".to_string(),
            Box::new(RecipeUnsolved::Cancel(
                "Alex".to_string(),
                Box::new(RecipeUnsolved::CommitLeftExcessToOrder(
                    "Alex".to_string(),
                    Box::new(RecipeUnsolved::Commit(
                        ArgsCommit {
                            ms_timestamp: U128(0)
                        },
                        Box::new(RecipeUnsolved::Order(
                            "Alex".to_string(),
                            ArgsOrder {
                                from_amt: U128(0),
                                desired_asset: asset_b.clone(),
                                desired_chain: U128(0),
                                desired_amt: U128(707651766525132719317267334528916),
                            },
                            Box::new(RecipeUnsolved::Balance(
                                "Alex".to_string(),
                                ArgsBalance {
                                    asset: asset_a.clone(),
                                    chain: U128(225476647479317694062150620526444369903),
                                    amount: U128(126508738668270503307039462831516350703),
                                    ms_timestamp: U128(88720005195802133222596758088858595407),
                                },
                            )),
                        )),
                        Box::new(RecipeUnsolved::Order(
                            "Eli".to_string(),
                            ArgsOrder {
                                from_amt: U128(6192372688119060292595079300484488726),
                                desired_asset: asset_a,
                                desired_chain: U128(307727618134271405861877130616748334084),
                                desired_amt: U128(19942947448678186698998836289408085284),
                            },
                            Box::new(RecipeUnsolved::Balance(
                                "Eli".to_string(),
                                ArgsBalance {
                                    asset: asset_b,
                                    chain: U128(24713657718869044757078304407331877748),
                                    amount: U128(56961730660561917068169863894177734718),
                                    ms_timestamp: U128(11835655638720700881890814969174336643),
                                },
                            )),
                        )),
                    )),
                )),
            ))
        )
    );
}
