// Tests that go the full way of asking questions about the applicative
// form before comparing the outcome in the state machine to the final
// outcome.

use libpassport::{
    applicative::{ArgsBalance, ArgsCommit, ArgsOrder},
    immutables::solver_key_offline,
    network::Network,
    solver_context::*,
    user_context::*,
    Storage,
};

mod experimentation;

use experimentation::*;

use proptest::prelude::*;

use ed25519_dalek::SigningKey;

use stylus_sdk::{alloy_primitives::FixedBytes, prelude::HostAccess};

proptest! {
    #[test]
    #[ignore]
    fn test_apply(signer_priv_key in any::<[u8; 32]>(), e in any::<Entry>()) {
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
        let msg_sender = s.vm().msg_sender();
        s.app.ed25519_keys.setter(0).set(FixedBytes::from_slice(signer_pub.as_bytes()));
        s.app.ed25519_owners.setter(0).set(msg_sender);
        apply_balances(&mut s, starting_amts(&e));
        s.app.apply(
            s.app.validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap()
        )
            .unwrap();
    }
}

#[test]
fn test_apply_2() {
    let signer_priv_key = [0u8; 32];
    /* let e = Entry::Cancel(TestOrder::CommitRightExcessToOrder(Box::new(TestCommit::Commit(Box::new(
        TestCommitInside {
            args: ArgsCommit { ms_timestamp: 0 },
            left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit { ms_timestamp: 0 },
                    left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args: ArgsCommit { ms_timestamp: 0 },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0,
                                            0, 0,
                                        ],
                                        chain: 0,
                                        amount: 0,
                                        ms_timestamp: 54605753107711057641113418769989,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 111713659194848969324448206362625575306,
                                    desired_asset: [
                                        65, 110, 126, 3, 66, 181, 50, 169, 220, 157, 116, 201, 152,
                                        224, 231, 144, 239, 100, 246, 3,
                                    ],
                                    desired_chain: 126508738668270503307039462831516350703,
                                    desired_amt: 88720005195802133222596758088858595407,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            104, 229, 111, 239, 239, 75, 102, 164, 86, 204, 178,
                                            137, 167, 104, 27, 102, 97, 181, 142, 40,
                                        ],
                                        chain: 188259381966339623680013711456576128638,
                                        amount: 158056353123159321273596219701523934535,
                                        ms_timestamp: 23188333085495548946588216017843527887,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 328885601818301762029628764417363529061,
                                    desired_asset: [
                                        204, 64, 87, 230, 60, 56, 9, 127, 105, 206, 159, 25, 128,
                                        209, 105, 236, 199, 107, 9, 249,
                                    ],
                                    desired_chain: 274486929356778341899652319785989165804,
                                    desired_amt: 320569236902476478523558249204735344365,
                                },
                            }))),
                        })),
                    ))),
                    right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 302306682539350410484312740961886832548,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            43, 152, 157, 145, 200, 37, 248, 25, 3, 77, 19, 49,
                                            116, 176, 1, 102, 62, 151, 45, 38,
                                        ],
                                        chain: 11835655638720700881890814969174336643,
                                        amount: 36992472151634260284500250019252995923,
                                        ms_timestamp: 140960259466510338919973252939009975594,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 253459334905483865556895105409239578934,
                                    desired_asset: [
                                        112, 175, 13, 139, 188, 145, 69, 61, 205, 216, 11, 198, 4,
                                        108, 15, 246, 36, 95, 130, 57,
                                    ],
                                    desired_chain: 94636658802915117892383761895483730296,
                                    desired_amt: 270008884643397102601701986178909600765,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            227, 192, 167, 250, 154, 184, 55, 128, 124, 99, 250,
                                            106, 80, 243, 188, 94, 152, 18, 159, 207,
                                        ],
                                        chain: 241858455840355572507648962270524222707,
                                        amount: 135983393360971126081861298618175388721,
                                        ms_timestamp: 162641873254200223477243764816810253625,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 259332073197853050513467805778409171588,
                                    desired_asset: [
                                        236, 151, 36, 139, 201, 127, 148, 161, 246, 140, 162, 87,
                                        226, 255, 0, 134, 230, 228, 138, 252,
                                    ],
                                    desired_chain: 26661426631872375138865868971784225969,
                                    desired_amt: 52833633084629656597473247770008838517,
                                },
                            }))),
                        })),
                    ))),
                })),
            ))),
            right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(
                TestCommit::Commit(Box::new(TestCommitInside {
                    args: ArgsCommit {
                        ms_timestamp: 322275798272514434649132080745486903332,
                    },
                    left: Box::new(TestOrder::CommitLeftExcessToOrder(Box::new(
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 237773410097257951985028996295701106258,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            239, 84, 20, 214, 136, 88, 237, 11, 109, 197, 99, 206,
                                            64, 216, 174, 153, 37, 145, 177, 68,
                                        ],
                                        chain: 275679835223001621297153760439211506552,
                                        amount: 235125459858618703476883255876790280815,
                                        ms_timestamp: 122158141337202828505968782238606359820,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 10860313813723825617815491556834763255,
                                    desired_asset: [
                                        223, 252, 47, 207, 129, 141, 31, 159, 5, 75, 151, 212, 47,
                                        46, 217, 81, 36, 127, 171, 229,
                                    ],
                                    desired_chain: 250522765677056535127712465207031493160,
                                    desired_amt: 19748150831852849840918620083115085073,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            15, 251, 84, 147, 176, 229, 224, 106, 54, 198, 70, 44,
                                            250, 18, 207, 83, 28, 238, 131, 191,
                                        ],
                                        chain: 143035875179039418904207703635995364307,
                                        amount: 31561208669121740151554875519127865862,
                                        ms_timestamp: 12359151042727274431200864338205984579,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 108137052259204888881520239054270012565,
                                    desired_asset: [
                                        218, 41, 127, 183, 32, 41, 241, 141, 145, 182, 171, 86,
                                        218, 59, 65, 178, 196, 141, 4, 84,
                                    ],
                                    desired_chain: 267037543424443300284103081784761282522,
                                    desired_amt: 7133211868824227930336396130661228018,
                                },
                            }))),
                        })),
                    ))),
                    right: Box::new(TestOrder::CommitRightExcessToOrder(Box::new(
                        TestCommit::Commit(Box::new(TestCommitInside {
                            args: ArgsCommit {
                                ms_timestamp: 20196192324841025228276527611184027117,
                            },
                            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            188, 191, 251, 110, 247, 46, 135, 158, 85, 214, 32, 74,
                                            51, 24, 125, 80, 53, 231, 144, 3,
                                        ],
                                        chain: 127184834038886112123923467662707202951,
                                        amount: 46886208906261310993848354447067929752,
                                        ms_timestamp: 134270720251684625311143498939783535068,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 117011833649155003137922487252252258566,
                                    desired_asset: [
                                        92, 50, 103, 251, 156, 255, 137, 39, 79, 154, 137, 110,
                                        188, 67, 182, 146, 177, 125, 130, 234,
                                    ],
                                    desired_chain: 120934643301613249609636081478290748702,
                                    desired_amt: 262946928659167323773204388444455331266,
                                },
                            }))),
                            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                                from: Box::new(TestBalance::Balance(TestBalanceInside {
                                    args: ArgsBalance {
                                        asset: [
                                            11, 43, 113, 103, 31, 6, 179, 171, 165, 79, 154, 255,
                                            159, 72, 94, 169, 40, 137, 219, 56,
                                        ],
                                        chain: 144615449047607788339713840890578972969,
                                        amount: 246626070654408523742081292568828153298,
                                        ms_timestamp: 231881543574203499859776996343687960962,
                                    },
                                })),
                                args: ArgsOrder {
                                    from_amt: 19815731280301412915059320694907134262,
                                    desired_asset: [
                                        248, 240, 238, 91, 76, 45, 68, 225, 151, 151, 12, 234, 31,
                                        208, 1, 13, 255, 194, 152, 76,
                                    ],
                                    desired_chain: 107229507878582853837350764374091027055,
                                    desired_amt: 239460754668301716588558370630034747208,
                                },
                            }))),
                        })),
                    ))),
                })),
            ))),
        },
    )))));
    */
    let e = Entry::Cancel(TestOrder::Order(Box::new(TestOrderInside {
        from: Box::new(TestBalance::Balance(TestBalanceInside {
            args: ArgsBalance {
                asset: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
                chain: 0,
                amount: 1000,
                ms_timestamp: 54605753107711057641113418769989,
            },
        })),
        args: ArgsOrder {
            from_amt: 100,
            desired_asset: [
                65, 110, 126, 3, 66, 181, 50, 169, 220, 157, 116, 201, 152, 224, 231, 144, 239,
                100, 246, 3,
            ],
            desired_chain: 126508738668270503307039462831516350703,
            desired_amt: 88720005195802133222596758088858595407,
        },
    })));
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    // Implicitly registers the solver key.
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
    let msg_sender = s.vm().msg_sender();
    s.app
        .ed25519_keys
        .setter(0)
        .set(FixedBytes::from_slice(signer_pub.as_bytes()));
    s.app.ed25519_owners.setter(0).set(msg_sender);
    apply_balances(&mut s, starting_amts(&e));
    s.app
        .apply(
            s.app
                .validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap(),
        )
        .unwrap();
}
