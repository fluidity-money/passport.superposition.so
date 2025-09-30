use crate::{
    accounts::{Account, Accounts},
    unsolved::RecipeUnsolved,
};

use libpassport::applicative::Applicative;

use ed25519_dalek::SigningKey;

use std::collections::HashMap;

fn _unsolved_to_app(accounts: HashMap<String, (SigningKey, u64)>, r: RecipeUnsolved) -> Applicative {
    todo!()
}

fn unsolved_to_applicative(accounts: Accounts, r: RecipeUnsolved) -> Applicative {
    _unsolved_to_app(
        accounts
            .0
            .into_iter()
            .map(|Account { key, name, place }| (name, (SigningKey::from_bytes(&key), place)))
            .collect::<HashMap<_, _>>(),
        r,
    )
}
