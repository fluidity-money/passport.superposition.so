#![cfg(not(target_arch = "wasm32"))]

use libpassport::{
    applicative::*, error::*, immutables::solver_key_offline, network::Network, solver_context::*,
    user_context::*, utils::strat_address_not_empty, Storage,
};

use ed25519_dalek::SigningKey;

use stylus_sdk::alloy_primitives::{Address, FixedBytes};

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
        }) => Ok(user_app.balance(Address::from(asset), *chain, *amount, *ms_timestamp)),
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
                args.from_amt,
                Address::from(args.desired_asset),
                args.desired_chain,
                args.desired_amt,
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
            user_app.withdraw(solver_sig, converted_balance, None)
        }
        Entry::Order(test_balance) => user_app.order(
            0,
            Address::default(),
            0,
            0,
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
        signer_priv_key in any::<[u8; 32]>(),
        signer_addr in strat_address_not_empty(),
        e: Entry
    ) {
        let signer_priv = SigningKey::from_bytes(&signer_priv_key);
        let signer_pub = signer_priv.verifying_key();
        let solver_ctx = SolverContext::new(solver_key_offline());
        // Implicitly registers the solver key.
        let user_ctx = UserContext::new_from_bytes(
            signer_priv.as_bytes(),
            0
        );
        let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
        let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
        s.app.ed25519_keys.setter(0).set(FixedBytes::from_slice(signer_pub.as_bytes()));
        s.app.ed25519_owners.setter(0).set(signer_addr);
        s.app.validate(Network::OFFLINE, &user_ctx.accounts, converted).unwrap();
    }
}

#[test]
fn test_simple_into_cancel() {
    let signer_priv = SigningKey::from_bytes(&[1u8; 32]);
    let signer_key = signer_priv.verifying_key();
    let solver_addr = Address::from([1u8; 20]);
    let solver_ctx = SolverContext::new(solver_key_offline());
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
    s.app
        .ed25519_keys
        .setter(0)
        .set(FixedBytes::from_slice(signer_key.as_bytes()));
    s.app.ed25519_owners.setter(0).set(solver_addr);
    let e = Entry::Withdraw(TestBalance::CommitRightFilledToBalance(Box::new(TestCommit::Commit(Box::new(
        TestCommitInside {
            args: ArgsCommit { ms_timestamp: 0 },
            left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(TestCommit::Commit(Box::new(TestCommitInside {
                args: ArgsCommit { ms_timestamp: 0 },
                left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                    TestCommit::Commit(Box::new(TestCommitInside {
                        args: ArgsCommit { ms_timestamp: 0 },
                        left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                            from: Box::new(TestBalance::Balance(TestBalanceInside {
                                args: ArgsBalance {
                                    asset: [
                                        0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                    ],
                                    chain: 3,
                                    amount: 56099561609010545350793644506327707195,
                                    ms_timestamp: 277864496551148979307559563409430510480,
                                },
                            })),
                            args: ArgsOrder {
                                from_amt: 313644024035310137011647848486540656511,
                                desired_asset: [
                                    63, 32, 183, 126, 101, 115, 116, 118, 191, 83, 13, 66, 6, 0,
                                    153, 92, 122, 252, 202, 117,
                                ],
                                desired_chain: 229972955726412453383369364413433881050,
                                desired_amt: 163073534072517870893627572716641426599,
                            },
                        }))),
                        right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                            from: Box::new(TestBalance::Balance(TestBalanceInside {
                                args: ArgsBalance {
                                    asset: [
                                        76, 128, 24, 212, 37, 83, 85, 202, 29, 116, 94, 242, 14,
                                        153, 180, 214, 190, 115, 28, 45,
                                    ],
                                    chain: 93221576065995587210658910886752529348,
                                    amount: 150667883398757509576232755303900354347,
                                    ms_timestamp: 92547440453636988145516562835028583712,
                                },
                            })),
                            args: ArgsOrder {
                                from_amt: 52771575092819886020368852503125150543,
                                desired_asset: [
                                    8, 175, 87, 31, 35, 117, 195, 180, 225, 96, 65, 186, 193, 120,
                                    12, 128, 219, 116, 198, 32,
                                ],
                                desired_chain: 67840090979626373720015446883107598266,
                                desired_amt: 74591063512175575758367155717642126831,
                            },
                        }))),
                    })),
                ))),
                right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit {
                        ms_timestamp: 298943278997994472306405989066889612216,
                    },
                    left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 56679203155017875216960605614936424478,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            198, 114, 248, 170, 10, 100, 108, 141, 208, 224, 170,
                                            129, 191, 104, 228, 73, 73, 107, 113, 203,
                                        ],
                                        chain: 285894907003591006624805008666230249480,
                                        amount: 295050182633368717450572971045374472457,
                                        ms_timestamp: 138524646641477801459810675919868526970,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 45665535137314636259641212414815619780,
                                    desired_asset: [
                                        37, 149, 49, 80, 254, 122, 156, 14, 218, 154, 175, 232,
                                        190, 69, 126, 90, 225, 83, 24, 54,
                                    ],
                                    desired_chain: 91741467474242771652303089891485113467,
                                    desired_amt: 11812184116545681051048387825638432150,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            97, 18, 0, 107, 75, 48, 120, 18, 113, 35, 89, 119, 226,
                                            144, 21, 150, 151, 178, 193, 155,
                                        ],
                                        chain: 108948759720332955213591205156039371050,
                                        amount: 289732289167904837500822541061079168945,
                                        ms_timestamp: 160853681434325203175398261372484528044,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 274210021637817839691350491535962196136,
                                    desired_asset: [
                                        5, 48, 4, 90, 254, 219, 68, 153, 191, 28, 70, 195, 255,
                                        182, 167, 38, 55, 214, 106, 206,
                                    ],
                                    desired_chain: 9297064855705640555239292457837243329,
                                    desired_amt: 295827604457062504430233688562374743902,
                                },
                            }))),
                        })),
                    ))),
                    right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(TestCommit::Commit(Box::new(
                        TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 224033194381108177167279098097532277905,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            147, 84, 48, 100, 28, 185, 177, 179, 95, 139, 206, 38,
                                            6, 145, 177, 164, 150, 25, 40, 245,
                                        ],
                                        chain: 37935263769605188108472890252097641959,
                                        amount: 99666384312193290243273089229848683897,
                                        ms_timestamp: 332974194148005712131728006819330384141,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 227209094930023438915160194220483293805,
                                    desired_asset: [
                                        15, 29, 18, 137, 68, 179, 171, 239, 170, 46, 39, 34, 138,
                                        142, 236, 142, 205, 80, 85, 160,
                                    ],
                                    desired_chain: 227916533332138251298398819263529137381,
                                    desired_amt: 24135230482437571766481853429502339615,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            171, 1, 106, 131, 244, 163, 205, 176, 52, 10, 16, 130,
                                            214, 58, 222, 149, 179, 131, 222, 19,
                                        ],
                                        chain: 41171617379700007756069422475978723944,
                                        amount: 47464982280537112470979582028851086779,
                                        ms_timestamp: 124834729320259741457341361920995960841,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 242915847824954606944206070175649504030,
                                    desired_asset: [
                                        147, 131, 241, 112, 177, 7, 228, 122, 15, 192, 87, 32, 143,
                                        159, 72, 208, 51, 181, 21, 224,
                                    ],
                                    desired_chain: 193635475714814465909756788737031969173,
                                    desired_amt: 3397057957679265337858768051559819064,
                                },
                            }))),
                        },
                    ))))),
                }))))),
            }))))),
            right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(
                TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit {
                        ms_timestamp: 205970582820571560698833572663188116214,
                    },
                    left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(TestCommit::Commit(Box::new(
                        TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 142934945070407282040972571848114438390,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            198, 60, 24, 172, 131, 31, 92, 176, 235, 84, 223, 136,
                                            159, 102, 130, 196, 128, 31, 143, 150,
                                        ],
                                        chain: 271088813697535733237502269757520906425,
                                        amount: 291172529558064780801545730511315028066,
                                        ms_timestamp: 84132574555635401792924625200530665756,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 282732367442205267405064926541767520415,
                                    desired_asset: [
                                        74, 199, 80, 77, 254, 81, 47, 229, 189, 231, 8, 230, 236,
                                        150, 185, 205, 194, 114, 231, 49,
                                    ],
                                    desired_chain: 15016146245335547351144627065390591306,
                                    desired_amt: 246429829945578611334824001470348397504,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            132, 246, 134, 41, 56, 249, 45, 135, 50, 8, 31, 184,
                                            99, 177, 111, 207, 180, 229, 159, 243,
                                        ],
                                        chain: 10314491203841371823307255725960138183,
                                        amount: 336766399449933691453414408708953894055,
                                        ms_timestamp: 12159456991374480043824443967866311992,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 262529716789746856390333622526159343321,
                                    desired_asset: [
                                        68, 147, 188, 15, 106, 90, 101, 184, 219, 49, 14, 58, 62,
                                        229, 30, 42, 7, 84, 92, 252,
                                    ],
                                    desired_chain: 323991108465066420407432942062628676781,
                                    desired_amt: 334553695995085800251547839365459450215,
                                },
                            }))),
                        },
                    ))))),
                    right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(TestCommit::Commit(Box::new(
                        TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 172930386038617774597231045475702462158,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            218, 108, 38, 191, 113, 66, 241, 207, 160, 24, 232,
                                            126, 181, 51, 103, 161, 228, 82, 195, 56,
                                        ],
                                        chain: 40773823639095896322834466688466507213,
                                        amount: 19998560296122432562798285975630284997,
                                        ms_timestamp: 297299550054585626701062700872093392266,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 339443693314682773939158904090126693454,
                                    desired_asset: [
                                        86, 239, 47, 223, 83, 100, 160, 188, 153, 70, 134, 215, 17,
                                        149, 246, 38, 110, 48, 225, 172,
                                    ],
                                    desired_chain: 283910555082266181913248244114355406993,
                                    desired_amt: 52313530921438222139533753535819220470,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            166, 112, 143, 55, 218, 85, 110, 209, 238, 34, 229,
                                            227, 81, 227, 181, 90, 81, 190, 235, 224,
                                        ],
                                        chain: 129153239340515351491855288715416959299,
                                        amount: 98934769413624370404144584159307478856,
                                        ms_timestamp: 59335138315736120537097364767639106003,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 290328656653875770163931412673702682846,
                                    desired_asset: [
                                        142, 98, 174, 119, 122, 91, 36, 218, 230, 121, 106, 132,
                                        142, 44, 60, 213, 208, 228, 238, 142,
                                    ],
                                    desired_chain: 183186548353566861183140435540183413357,
                                    desired_amt: 76758871697356278506192206755847555742,
                                },
                            }))),
                        },
                    ))))),
                })),
            ))),
        },
    )))));
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    s.app
        .validate(Network::OFFLINE, &user_ctx.accounts, converted)
        .unwrap();
}
