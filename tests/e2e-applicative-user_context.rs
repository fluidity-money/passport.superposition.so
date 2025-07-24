#![cfg(not(target_arch = "wasm32"))]

use libpassport::{
    accounts::*, applicative::*, crypto::validate, error::*, solver_context::*, user_context::*,
};

use stylus_sdk::alloy_primitives::{Address, U256};

use proptest::prelude::*;

#[derive(Debug, PartialEq)]
struct TestBalanceInside {
    args: ArgsBalance,
}

#[derive(Debug, PartialEq)]
struct TestOrderInside {
    from: Box<TestBalance>,
    args: ArgsOrder,
}

#[derive(Debug, PartialEq)]
enum TestOrder {
    Order(Box<TestOrderInside>),
    CommitLeftExcessToOrder(Box<TestCommit>),
    CommitRightExcessToOrder(Box<TestCommit>),
}

#[derive(Debug, PartialEq)]
struct TestCommitInside {
    args: ArgsCommit,
    left: Box<TestOrder>,
    right: Box<TestOrder>,
}

#[derive(Debug, PartialEq)]
enum TestCommit {
    Commit(Box<TestCommitInside>),
}

#[derive(Debug, PartialEq)]
enum TestBalance {
    Balance(TestBalanceInside),
    CommitLeftFilledToBalance(Box<TestCommit>),
    CommitRightFilledToBalance(Box<TestCommit>),
}

#[derive(Debug, PartialEq)]
enum Entry {
    Balance(TestBalance),
    Withdraw(TestBalance),
    Order(TestBalance),
    Cancel(TestOrder),
    Commit(TestCommit),
    CommitLeftFilledToBalance(TestCommit),
    CommitRightFilledToBalance(TestCommit),
    CommitLeftExcessToOrder(TestCommit),
    CommitRightExcessToOrder(TestCommit),
}

impl Arbitrary for Entry {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        let bal_leaf = any::<ArgsBalance>()
            .prop_map(|args| TestBalance::Balance(TestBalanceInside { args }))
            .boxed();
        let ord_leaf = (any::<ArgsBalance>(), any::<ArgsOrder>())
            .prop_map(|(bal_args, ord_args)| {
                TestOrder::Order(Box::new(TestOrderInside {
                    from: Box::new(TestBalance::Balance(TestBalanceInside { args: bal_args })),
                    args: ord_args,
                }))
            })
            .boxed();
        let commit_leaf = (any::<ArgsCommit>(), ord_leaf.clone(), ord_leaf.clone())
            .prop_map(|(args, left, right)| {
                TestCommit::Commit(Box::new(TestCommitInside {
                    args,
                    left: Box::new(left),
                    right: Box::new(right),
                }))
            })
            .boxed();
        let commit_strat = commit_leaf.prop_recursive(4, 64, 4, |inner| {
            (any::<ArgsCommit>(), inner.clone(), inner)
                .prop_map(|(args, left_c, right_c)| {
                    TestCommit::Commit(Box::new(TestCommitInside {
                        args,
                        left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(left_c))),
                        right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(right_c))),
                    }))
                })
                .boxed()
        });
        let c_ord_l = commit_strat.clone();
        let c_ord_r = commit_strat.clone();
        let bal_strat_for_ord = bal_leaf.clone();
        let ord_strat = ord_leaf.prop_recursive(4, 64, 4, move |inner| {
            prop_oneof![
                (bal_strat_for_ord.clone(), any::<ArgsOrder>()).prop_map(
                    |(test_balance, ord_args)| {
                        TestOrder::Order(Box::new(TestOrderInside {
                            from: Box::new(test_balance),
                            args: ord_args,
                        }))
                    }
                ),
                c_ord_l
                    .clone()
                    .prop_map(|c| TestOrder::CommitLeftExcessToOrder(Box::new(c))),
                c_ord_r
                    .clone()
                    .prop_map(|c| TestOrder::CommitRightExcessToOrder(Box::new(c))),
                inner,
            ]
            .boxed()
        });
        let c_bal = commit_strat.clone();
        let bal_strat = bal_leaf.prop_recursive(4, 64, 4, move |inner| {
            prop_oneof![
                c_bal
                    .clone()
                    .prop_map(|c| TestBalance::CommitLeftFilledToBalance(Box::new(c))),
                c_bal
                    .clone()
                    .prop_map(|c| TestBalance::CommitRightFilledToBalance(Box::new(c))),
                inner,
            ]
            .boxed()
        });
        prop_oneof![
            bal_strat.clone().prop_map(Entry::Balance),
            bal_strat.clone().prop_map(Entry::Withdraw),
            bal_strat.clone().prop_map(Entry::Order),
            ord_strat.clone().prop_map(Entry::Cancel),
            commit_strat.clone().prop_map(Entry::Commit),
            commit_strat
                .clone()
                .prop_map(Entry::CommitLeftFilledToBalance),
            commit_strat
                .clone()
                .prop_map(Entry::CommitRightFilledToBalance),
            commit_strat
                .clone()
                .prop_map(Entry::CommitLeftExcessToOrder),
            commit_strat.prop_map(Entry::CommitRightExcessToOrder),
        ]
        .boxed()
    }
}

fn convert_test_balance<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    test_balance: &TestBalance,
) -> Result<Applicative, Error> {
    match test_balance {
        TestBalance::Balance(TestBalanceInside {
            args:
                ArgsBalance {
                    asset,
                    chain,
                    amount,
                    ms_timestamp,
                },
        }) => Ok(user_app.balance(asset.x, *chain, amount.x, *ms_timestamp)),
        TestBalance::CommitLeftFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_left_filled_to_balance(converted_commit)
        }
        TestBalance::CommitRightFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_right_filled_to_balance(converted_commit)
        }
    }
}

fn convert_test_order<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    test_order: &TestOrder,
) -> Result<Applicative, Error> {
    match test_order {
        TestOrder::Order(order_inside) => {
            let converted_from = convert_test_balance(user_app, solver_app, &order_inside.from)?;
            let args = &order_inside.args;
            user_app.order(
                args.from_amt.x,
                args.desired_asset.x,
                args.desired_chain,
                args.desired_amt.x,
                converted_from,
            )
        }
        TestOrder::CommitLeftExcessToOrder(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_left_excess_to_order(converted_commit)
        }
        TestOrder::CommitRightExcessToOrder(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_right_excess_to_order(converted_commit)
        }
    }
}

fn convert_test_commit<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    test_commit: &TestCommit,
) -> Result<Applicative, Error> {
    match test_commit {
        TestCommit::Commit(commit_inside) => {
            let left_converted = convert_test_order(user_app, solver_app, &commit_inside.left)?;
            let right_converted = convert_test_order(user_app, solver_app, &commit_inside.right)?;
            let solver_sig = solver_app.commit(
                commit_inside.args.ms_timestamp,
                left_converted.clone(),
                right_converted.clone(),
            )?;
            user_app.commit(
                solver_sig,
                commit_inside.args.ms_timestamp,
                left_converted,
                right_converted,
            )
        }
    }
}

fn convert<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    entry: &Entry,
) -> Result<Applicative, Error> {
    match entry {
        Entry::Balance(test_balance) => convert_test_balance(user_app, solver_app, test_balance),
        Entry::Withdraw(test_balance) => {
            let converted_balance = convert_test_balance(user_app, solver_app, test_balance)?;
            let solver_sig = solver_app.withdraw(converted_balance.clone())?;
            user_app.withdraw(solver_sig, converted_balance)
        }
        Entry::Order(test_balance) => user_app.order(
            U256::from(0),
            Address::default(),
            0,
            U256::from(0),
            convert_test_balance(user_app, solver_app, test_balance)?,
        ),
        Entry::Cancel(test_order) => {
            let converted_order = convert_test_order(user_app, solver_app, test_order)?;
            let solver_sig = solver_app.cancel(converted_order.clone())?;
            user_app.cancel(solver_sig, converted_order)
        }
        Entry::Commit(test_commit) => convert_test_commit(user_app, solver_app, test_commit),
        Entry::CommitLeftFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_left_filled_to_balance(converted_commit)
        }
        Entry::CommitRightFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_right_filled_to_balance(converted_commit)
        }
        Entry::CommitLeftExcessToOrder(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_left_excess_to_order(converted_commit)
        }
        Entry::CommitRightExcessToOrder(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_right_excess_to_order(converted_commit)
        }
    }
}

proptest! {
    #[test]
    fn test_conversions(
        solver_key in any::<[u8; 32]>(),
        signer_key in any::<[u8; 32]>(),
        e: Entry
    ) {
        let solver_ctx = SolverContext::new_from_bytes(solver_key);
        let user_ctx = UserContext::new_from_bytes(
            Accounts::default().with_solver(solver_key),
            signer_key
        );
        let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
        dbg!(&converted);
        validate(&user_ctx.accounts, &converted).unwrap();
    }
}
