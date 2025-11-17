// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    applicative::*,
    apply::apply,
    call_eip20_extras,
    conversion::validate,
    immutables::{pick_solver_key, solver_key_offline},
    network::Network,
    solver_context::*,
    storage::{self},
    user_context::*,
};

use bobcat_sdk::{
    entry::{host::set_msg_sender, msg_sender},
    maths::U,
    storage::host as storage_host,
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

#[test]
fn test_simple_apply() {
    let signer_priv_key = [
        38, 37, 197, 255, 30, 234, 252, 184, 30, 168, 152, 209, 36, 146, 88, 141, 213, 220, 61,
        191, 121, 200, 145, 223, 34, 75, 129, 113, 104, 12, 124, 202,
    ];
    storage_host::storage_clear();
    call_eip20_extras::clear();
    set_msg_sender([1u8; 20]);
    let e = Entry::Withdraw(TestBalance::CommitLeftFilledToBalance(Box::new(
        TestCommit::Commit(Box::new(TestCommitInside {
            args: ArgsCommit {
                ms_timestamp: U128(48517478656298620596289092692691332662),
            },
            left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit {
                        ms_timestamp: U128(47244592515662955545810822164048522122),
                    },
                    left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                        from: Box::new(TestBalance::Balance(TestBalanceInside {
                            args: ArgsBalance {
                                asset: Asset([
                                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                ]),
                                chain: 6124949163344909421,
                                amount: U128(230617854711066214364563614156323414009),
                                ms_timestamp: U128(318884888395207043350450011416698249459),
                            },
                        })),
                        args: ArgsOrder {
                            from_amt: U128(102648944053531729878261564810486063075),
                            desired_asset: Asset([
                                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                            ]),
                            desired_chain: U128(321172992780827823820708556427625344034),
                            desired_amt: U128(104059865613456211645288051521014921287),
                        },
                    }))),
                    right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                        from: Box::new(TestBalance::Balance(TestBalanceInside {
                            args: ArgsBalance {
                                asset: Asset([
                                    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                                ]),
                                chain: 10068993298131360742,
                                amount: U128(190240882065656212104208782526279707042),
                                ms_timestamp: U128(67638908455992037033526463012485255346),
                            },
                        })),
                        args: ArgsOrder {
                            from_amt: U128(16980427457911253408130308255975656210),
                            desired_asset: Asset([
                                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                            ]),
                            desired_chain: U128(301365458976364606252530000704705766671),
                            desired_amt: U128(144938153433307934226648289810762347459),
                        },
                    }))),
                })),
            ))),
            right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(
                TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit {
                        ms_timestamp: U128(211209237290234442204095328473054709708),
                    },
                    left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                        from: Box::new(TestBalance::Balance(TestBalanceInside {
                            args: ArgsBalance {
                                asset: Asset([
                                    1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                                ]),
                                chain: 17420485723122345274,
                                amount: U128(43900651520522288652505963059371614446),
                                ms_timestamp: U128(23474412624821262660873398785383198074),
                            },
                        })),
                        args: ArgsOrder {
                            from_amt: U128(27637156947976727073450424372322851727),
                            desired_asset: Asset([
                                2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                            ]),
                            desired_chain: U128(251666315623928646887861828149679920400),
                            desired_amt: U128(204673331374160494232336421946736248189),
                        },
                    }))),
                    right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                        from: Box::new(TestBalance::Balance(TestBalanceInside {
                            args: ArgsBalance {
                                asset: Asset([
                                    2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2, 2,
                                ]),
                                chain: 7834365052195421917,
                                amount: U128(316876851883914389279521734592222508658),
                                ms_timestamp: U128(207079860372034684638330597959763137296),
                            },
                        })),
                        args: ArgsOrder {
                            from_amt: U128(180474346798746020442141794423716313899),
                            desired_asset: Asset([
                                1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1, 1,
                            ]),
                            desired_chain: U128(85414412918790587994630930653958887136),
                            desired_amt: U128(20314643742326833355712690244047259593),
                        },
                    }))),
                })),
            ))),
        })),
    )));
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    // Implicitly registers the solver key.
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    storage::ed25519_keys::set(&U::ZERO, signer_pub.as_bytes().into());
    storage::ed25519_owners::set(&U::ZERO, &msg_sender().into());
    apply_balances(starting_amts(&e)).unwrap();
    apply(validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted).unwrap()).unwrap();
}

proptest! {
    #[test]
    fn test_apply(signer_priv_key in any::<[u8; 32]>(), e in any::<Entry>()) {
        storage_host::storage_clear();
        call_eip20_extras::clear();
        set_msg_sender([1u8; 20]);
        let signer_priv = SigningKey::from_bytes(&signer_priv_key);
        let signer_pub = signer_priv.verifying_key();
        let solver_ctx = SolverContext::new(solver_key_offline());
        // Implicitly registers the solver key.
        let user_ctx = UserContext::new_from_bytes(
            signer_priv.as_bytes(),
            0
        );
        let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
        storage::ed25519_keys::set(&U::ZERO, signer_pub.as_bytes().into());
        storage::ed25519_owners::set(&U::ZERO, &msg_sender().into());
        apply_balances(starting_amts(&e)).unwrap();
        apply(
            validate(&pick_solver_key(Network::CUSTOM), &vec![0], &converted)
                .unwrap()
        )
            .unwrap();
    }
}
