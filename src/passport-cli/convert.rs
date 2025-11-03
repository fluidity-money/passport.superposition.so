use crate::{
    accounts::{Account, Accounts},
    unsolved::RecipeUnsolved,
};

use libpassport::{
    applicative::{Applicative, ArgsBalance, ArgsOrder, SolverApplicative, UserApplicative},
    solver_context::SolverContext,
    user_context::UserContext,
    ops::OpSolver
};

use ed25519_dalek::SigningKey;

use std::collections::HashMap;

fn ctx(accounts: &HashMap<String, (usize, SigningKey, u64)>, n: String) -> UserContext {
    let (i, k, _) = accounts.get(&n).unwrap();
    let i: u8 = (*i).try_into().unwrap();
    UserContext::new_from_key(k.clone(), i)
}

fn _unsolved_to_app(
    accounts: &HashMap<String, (usize, SigningKey, u64)>,
    solver: &SolverContext,
    r: RecipeUnsolved,
) -> Applicative {
    match r {
        RecipeUnsolved::Balance(
            n,
            ArgsBalance {
                asset,
                chain,
                amount,
                ms_timestamp,
            },
        ) => ctx(accounts, n).balance(Address::from(asset.0), chain, amount, ms_timestamp),
        RecipeUnsolved::Withdraw(n, from) => {
            let from = _unsolved_to_app(accounts, solver, *from);
            let s_sig = solver.withdraw(&from).unwrap();
            ctx(accounts, n).withdraw(s_sig, from, None).unwrap()
        }
        RecipeUnsolved::Order(
            n,
            ArgsOrder {
                from_amt,
                desired_asset,
                desired_chain,
                desired_amt,
            },
            from,
        ) => ctx(accounts, n)
            .order(
                from_amt.0,
                Address::from(desired_asset.0),
                desired_chain.0,
                desired_amt.0,
                _unsolved_to_app(accounts, solver, *from),
            )
            .unwrap(),
        RecipeUnsolved::Cancel(n, from) => {
            let from = _unsolved_to_app(accounts, solver, *from);
            let s_sig = solver.cancel(&from).unwrap();
            ctx(accounts, n).cancel(s_sig, from).unwrap()
        }
        RecipeUnsolved::Commit(args, left, right) => {
            let left = _unsolved_to_app(accounts, solver, *left);
            let right = _unsolved_to_app(accounts, solver, *right);
            let s_sig = solver.commit(args.ms_timestamp.0, &left, &right).unwrap();
            Applicative::Commit(s_sig, args, Box::new(left), Box::new(right))
        }
        RecipeUnsolved::CommitLeftFilledToBalance(n, from) => ctx(accounts, n)
            .commit_left_filled_to_balance(_unsolved_to_app(accounts, solver, *from))
            .unwrap(),
        RecipeUnsolved::CommitRightFilledToBalance(n, from) => ctx(accounts, n)
            .commit_right_filled_to_balance(_unsolved_to_app(accounts, solver, *from))
            .unwrap(),
        RecipeUnsolved::CommitLeftExcessToOrder(n, from) => ctx(accounts, n)
            .commit_left_excess_to_order(_unsolved_to_app(accounts, solver, *from))
            .unwrap(),
        RecipeUnsolved::CommitRightExcessToOrder(n, from) => ctx(accounts, n)
            .commit_right_excess_to_order(_unsolved_to_app(accounts, solver, *from))
            .unwrap(),
        RecipeUnsolved::Join(n, left, right) => {
            let left = _unsolved_to_app(accounts, solver, *left);
            let right = _unsolved_to_app(accounts, solver, *right);
            ctx(accounts, n).join(left, right).unwrap()
        }
    }
}

pub fn unsolved_to_solver(
    accounts: Accounts,
    solver: SigningKey,
    r: RecipeUnsolved,
) -> OpSolver {
    let offsets = accounts.0.iter().map(|Account { offset, .. }| *offset).collect::<Vec<_>>();
    let ap = _unsolved_to_app(
        &accounts
            .0
            .into_iter()
            .enumerate()
            .map(|(i, Account { key, name, offset })| {
                (name, (i, SigningKey::from_bytes(&key.0), offset))
            })
            .collect::<HashMap<_, _>>(),
        &SolverContext::new(solver),
        r,
    );
    OpSolver::Solve(offsets, ap)
}
