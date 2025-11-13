use libpassport::{add_liq::add_liq, applicative::*, call_eip20_extras, error::*};

use bobcat_sdk::{entry::msg_sender, maths::U};

use proptest::prelude::*;

#[derive(Debug, PartialEq, Clone)]
pub struct TestBalanceInside {
    pub args: ArgsBalance,
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestOrderInside {
    pub from: Box<TestBalance>,
    pub args: ArgsOrder,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TestOrder {
    Order(Box<TestOrderInside>),
    CommitLeftExcessToOrder(Box<TestCommit>),
    CommitRightExcessToOrder(Box<TestCommit>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct TestCommitInside {
    pub args: ArgsCommit,
    pub left: Box<TestOrder>,
    pub right: Box<TestOrder>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TestCommit {
    Commit(Box<TestCommitInside>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum TestBalance {
    Balance(TestBalanceInside),
    CommitLeftFilledToBalance(Box<TestCommit>),
    CommitRightFilledToBalance(Box<TestCommit>),
    Cancel(Box<TestOrder>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum Entry {
    Balance(TestBalance),
    Withdraw(TestBalance),
    MakeOrder(TestBalance),
    Order(TestOrder),
    Cancel(TestOrder),
    Commit(TestCommit),
    CommitLeftFilledToBalance(TestCommit),
    CommitRightFilledToBalance(TestCommit),
    CommitLeftExcessToOrder(TestCommit),
    CommitRightExcessToOrder(TestCommit),
}

#[derive(PartialEq)]
enum S {
    Left,
    Right,
}

fn any_balance_no_zero(side: S) -> impl Strategy<Value = ArgsBalance> {
    (1..u64::MAX, 1..u128::MAX, any::<u128>()).prop_map(move |(chain, amount, ms_timestamp)| {
        ArgsBalance {
            asset: Asset(if side == S::Left { [1; 20] } else { [2; 20] }),
            chain,
            amount: U128(amount),
            ms_timestamp: U128(ms_timestamp),
        }
    })
}

fn order_from_bal(ArgsBalance { amount, .. }: ArgsBalance) -> impl Strategy<Value = ArgsOrder> {
    (0..amount.0, any::<[u8; 20]>(), any::<u128>(), 1..u128::MAX).prop_map(
        |(from_amt, desired_asset, desired_chain, desired_amt)| ArgsOrder {
            from_amt: U128(from_amt),
            desired_asset: Asset(desired_asset),
            desired_chain: U128(desired_chain),
            desired_amt: U128(desired_amt),
        },
    )
}

fn commit_leaf_with_matching_orders() -> impl Strategy<Value = TestCommit> {
    any_balance_no_zero(S::Left)
        .prop_flat_map(|left_bal_args| {
            let left_asset = left_bal_args.asset.clone();
            let right_desired_asset = left_asset.clone();
            any_balance_no_zero(S::Right)
                .prop_filter(
                    "right asset must differ from left asset",
                    move |right_bal_args| right_bal_args.asset.0 != left_asset.0,
                )
                .prop_flat_map(move |right_bal_args| {
                    let right_asset = right_bal_args.asset.clone();
                    let left_desired_asset = right_asset.clone();
                    let right_desired_asset = right_desired_asset.clone();
                    let left_bal = left_bal_args.clone();
                    let right_bal = right_bal_args.clone();
                    (
                        0..left_bal.amount.0,
                        any::<u128>(),
                        1..right_bal.amount.0,
                        0..right_bal.amount.0,
                        any::<u128>(),
                        1..left_bal.amount.0,
                        any::<ArgsCommit>(),
                    )
                        .prop_map(
                            move |(
                                left_from_amt,
                                left_desired_chain,
                                left_desired_amt,
                                right_from_amt,
                                right_desired_chain,
                                right_desired_amt,
                                args,
                            )| {
                                let left_order = TestOrder::Order(Box::new(TestOrderInside {
                                    from: Box::new(TestBalance::Balance(TestBalanceInside {
                                        args: left_bal.clone(),
                                    })),
                                    args: ArgsOrder {
                                        from_amt: U128(left_from_amt),
                                        desired_asset: left_desired_asset.clone(),
                                        desired_chain: U128(left_desired_chain),
                                        desired_amt: U128(left_desired_amt),
                                    },
                                }));

                                let right_order = TestOrder::Order(Box::new(TestOrderInside {
                                    from: Box::new(TestBalance::Balance(TestBalanceInside {
                                        args: right_bal.clone(),
                                    })),
                                    args: ArgsOrder {
                                        from_amt: U128(right_from_amt),
                                        desired_asset: right_desired_asset.clone(),
                                        desired_chain: U128(right_desired_chain),
                                        desired_amt: U128(right_desired_amt),
                                    },
                                }));

                                TestCommit::Commit(Box::new(TestCommitInside {
                                    args,
                                    left: Box::new(left_order),
                                    right: Box::new(right_order),
                                }))
                            },
                        )
                })
        })
        .boxed()
}

impl Arbitrary for Entry {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_: Self::Parameters) -> Self::Strategy {
        let bal_leaf = any_balance_no_zero(S::Left)
            .prop_map(|args| TestBalance::Balance(TestBalanceInside { args }))
            .boxed();
        let ord_leaf = any_balance_no_zero(S::Left)
            .prop_flat_map(move |bal_args| {
                order_from_bal(bal_args.clone()).prop_map(move |ord_args| {
                    TestOrder::Order(Box::new(TestOrderInside {
                        from: Box::new(TestBalance::Balance(TestBalanceInside {
                            args: bal_args.clone(),
                        })),
                        args: ord_args,
                    }))
                })
            })
            .boxed();
        let commit_leaf = commit_leaf_with_matching_orders();
        let commit_strat = commit_leaf.prop_recursive(4, 10, 4, |inner| {
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
        let ord_strat = ord_leaf.prop_recursive(4, 10, 4, move |inner| {
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
        let ord_for_cancel = ord_strat.clone();
        let bal_strat = bal_leaf.prop_recursive(4, 10, 4, move |inner| {
            prop_oneof![
                c_bal
                    .clone()
                    .prop_map(|c| TestBalance::CommitLeftFilledToBalance(Box::new(c))),
                c_bal
                    .clone()
                    .prop_map(|c| TestBalance::CommitRightFilledToBalance(Box::new(c))),
                ord_for_cancel
                    .clone()
                    .prop_map(|o| TestBalance::Cancel(Box::new(o))),
                inner,
            ]
            .boxed()
        });
        prop_oneof![
            bal_strat.clone().prop_map(Entry::Balance),
            bal_strat.clone().prop_map(Entry::Withdraw),
            bal_strat.clone().prop_map(Entry::MakeOrder),
            ord_strat.clone().prop_map(Entry::Order),
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

pub fn convert_test_balance<T: UserApplicative, S: SolverApplicative>(
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
        }) => Ok(user_app.balance(Address::from(asset.0), *chain, amount.0, ms_timestamp.0)),
        TestBalance::CommitLeftFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_left_filled_to_balance(converted_commit)
        }
        TestBalance::CommitRightFilledToBalance(test_commit) => {
            let converted_commit = convert_test_commit(user_app, solver_app, test_commit)?;
            user_app.commit_right_filled_to_balance(converted_commit)
        }
        TestBalance::Cancel(test_order) => {
            let converted_order = convert_test_order(user_app, solver_app, test_order)?;
            user_app.cancel(solver_app.cancel(&converted_order)?, converted_order)
        }
    }
}

pub fn convert_test_order<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    test_order: &TestOrder,
) -> Result<Applicative, Error> {
    match test_order {
        TestOrder::Order(order_inside) => {
            let converted_from = convert_test_balance(user_app, solver_app, &order_inside.from)?;
            let args = &order_inside.args;
            user_app.order(
                args.from_amt.0,
                Address::from(args.desired_asset.0),
                args.desired_chain.0,
                args.desired_amt.0,
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
                commit_inside.args.ms_timestamp.0,
                &left_converted,
                &right_converted,
            )?;
            Ok(Applicative::Commit(
                solver_sig,
                ArgsCommit {
                    ms_timestamp: commit_inside.args.ms_timestamp.clone(),
                },
                Box::new(left_converted),
                Box::new(right_converted),
            ))
        }
    }
}

pub fn convert<T: UserApplicative, S: SolverApplicative>(
    user_app: &T,
    solver_app: &S,
    entry: &Entry,
) -> Result<Applicative, Error> {
    match entry {
        Entry::Balance(test_balance) => convert_test_balance(user_app, solver_app, test_balance),
        Entry::Withdraw(test_balance) => {
            let converted_balance = convert_test_balance(user_app, solver_app, test_balance)?;
            let solver_sig = solver_app.withdraw(&converted_balance)?;
            user_app.withdraw(solver_sig, converted_balance, None)
        }
        Entry::MakeOrder(test_balance) => user_app.order(
            0,
            Address::default(),
            0,
            0,
            convert_test_balance(user_app, solver_app, test_balance)?,
        ),
        Entry::Order(test_order) => convert_test_order(user_app, solver_app, test_order),
        Entry::Cancel(test_order) => {
            let converted_order = convert_test_order(user_app, solver_app, test_order)?;
            let solver_sig = solver_app.cancel(&converted_order)?;
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

fn starting_amts_balance(v: &mut Vec<(Address, u128)>, b: &TestBalance) {
    match b {
        TestBalance::Balance(TestBalanceInside {
            args: ArgsBalance { asset, amount, .. },
        }) => v.push((Address::from(asset.0), amount.0)),
        TestBalance::CommitLeftFilledToBalance(c) | TestBalance::CommitRightFilledToBalance(c) => {
            starting_amts_commit(v, &*c)
        }
        TestBalance::Cancel(o) => starting_amts_order(v, &*o),
    }
}

fn starting_amts_order(v: &mut Vec<(Address, u128)>, o: &TestOrder) {
    match o {
        TestOrder::Order(o) => starting_amts_balance(v, &*o.from),
        TestOrder::CommitLeftExcessToOrder(c) | TestOrder::CommitRightExcessToOrder(c) => {
            starting_amts_commit(v, c)
        }
    }
}

fn starting_amts_commit(v: &mut Vec<(Address, u128)>, TestCommit::Commit(c): &TestCommit) {
    starting_amts_order(v, &*c.left);
    starting_amts_order(v, &*c.right);
}

fn _starting_amts(v: &mut Vec<(Address, u128)>, e: &Entry) {
    match e {
        Entry::Balance(b) | Entry::Withdraw(b) | Entry::MakeOrder(b) => starting_amts_balance(v, b),
        Entry::Order(o) | Entry::Cancel(o) => starting_amts_order(v, o),
        Entry::Commit(c)
        | Entry::CommitLeftFilledToBalance(c)
        | Entry::CommitRightFilledToBalance(c)
        | Entry::CommitLeftExcessToOrder(c)
        | Entry::CommitRightExcessToOrder(c) => starting_amts_commit(v, c),
    }
}

pub fn starting_amts(e: &Entry) -> Vec<(Address, u128)> {
    let mut v = Vec::new();
    _starting_amts(&mut v, e);
    v
}

pub fn apply_balances(v: Vec<(Address, u128)>) -> Result<(), Error> {
    for (asset, amt) in v {
        let sender = msg_sender();
        call_eip20_extras::give(sender.into(), amt.into());
        add_liq(asset, sender, amt, U::ZERO, 0, U::ZERO, U::ZERO)?;
    }
    Ok(())
}
