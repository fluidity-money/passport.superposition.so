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

use stylus_sdk::{
    alloy_primitives::{Address, FixedBytes},
    prelude::HostAccess,
};

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
        apply_balances(&mut s, starting_amts(&e)).unwrap();
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
    let asset_a = [
        138, 193, 199, 165, 65, 110, 126, 3, 66, 181, 50, 169, 220, 157, 116, 201, 152, 224, 231,
        144,
    ];
    let asset_b = [
        179, 104, 133, 120, 164, 207, 97, 51, 43, 152, 157, 145, 200, 37, 248, 25, 3, 77, 19, 49,
    ];
    let e = Entry::Balance(TestBalance::Cancel(Box::new(
        TestOrder::CommitLeftExcessToOrder(Box::new(TestCommit::Commit(Box::new(TestCommitInside {
            args: ArgsCommit { ms_timestamp: 0 },
            left: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                from: Box::new(TestBalance::Balance(TestBalanceInside {
                    args: ArgsBalance {
                        asset: asset_a,
                        chain: 225476647479317694062150620526444369903,
                        amount: 126508738668270503307039462831516350703,
                        ms_timestamp: 88720005195802133222596758088858595407,
                    },
                })),
                args: ArgsOrder {
                    from_amt: 0,
                    desired_asset: asset_b,
                    desired_chain: 0,
                    desired_amt: 707651766525132719317267334528916,
                },
            }))),
            right: Box::new(TestOrder::Order(Box::new(TestOrderInside {
                from: Box::new(TestBalance::Balance(TestBalanceInside {
                    args: ArgsBalance {
                        asset: asset_b,
                        chain: 24713657718869044757078304407331877748,
                        amount: 56961730660561917068169863894177734718,
                        ms_timestamp: 11835655638720700881890814969174336643,
                    },
                })),
                args: ArgsOrder {
                    from_amt: 6192372688119060292595079300484488726,
                    desired_asset: asset_a,
                    desired_chain: 307727618134271405861877130616748334084,
                    desired_amt: 19942947448678186698998836289408085284,
                },
            }))),
        })))),
    )));
    let signer_priv = SigningKey::from_bytes(&signer_priv_key);
    let signer_pub = signer_priv.verifying_key();
    let solver_ctx = SolverContext::new(solver_key_offline());
    let user_ctx = UserContext::new_from_bytes(signer_priv.as_bytes(), 0);
    let converted = convert(&user_ctx, &solver_ctx, &e).unwrap();
    let mut s = Storage::from(&stylus_sdk::testing::vm::TestVM::new());
    let msg_sender = s.vm().msg_sender();
    s.app
        .ed25519_keys
        .setter(0)
        .set(FixedBytes::from_slice(signer_pub.as_bytes()));
    s.app.ed25519_owners.setter(0).set(msg_sender);
    apply_balances(&mut s, starting_amts(&e)).unwrap();
    s.app
        .apply(
            s.app
                .validate(Network::OFFLINE, &user_ctx.accounts, converted)
                .unwrap(),
        )
        .unwrap();
    dbg!(s
        .app
        .test_eip20
        .balances
        .getter(Address::from(asset_a))
        .get(s.vm().contract_address()));
    panic!()
}
